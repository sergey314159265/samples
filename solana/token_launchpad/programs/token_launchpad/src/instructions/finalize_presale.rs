use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn,
        Burn,
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use crate::{
    constants::{
        PRESALE_SEED,
        VAULT_SEED,
        WRAPPED_SOL_MINT_ADDRESS,
    },
    error::PresaleError,
    state::{
        presale::{
            PresaleState,
            PresaleType,
            RefundType,
        },
        vault::Vault,
    },
    utils::{
        is_authorized_to_finalize_presale,
        tranfer_sol_from_vault,
    },
};

#[event]
pub struct PresaleFinalized {
    pub success: bool,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct FinalizePresale<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK
    #[account(mut, constraint = owner.key() == presale.owner @ PresaleError::Unauthorized)]
    pub owner: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub fee_collector: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = presale,
        associated_token::token_program = token_program
    )]
    pub token_vault_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated in code
    pub pool_program: UncheckedAccount<'info>,

    /// CHECK: validated in code
    pub amm_config: UncheckedAccount<'info>,

    /// CHECK: validated in code
    pub pool_state: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn finalize_presale(ctx: Context<FinalizePresale>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let token_vault = &mut ctx.accounts.token_vault_account;
    let current_time = Clock::get().unwrap().unix_timestamp;

    require!(
        presale.token == ctx.accounts.token_mint.key(),
        PresaleError::InvalidTokenMint
    );

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?,
        PresaleError::Unauthorized
    );

    require!(
        current_time > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    require!(
        ctx.accounts.fee_collector.key() == presale.fee_collector,
        PresaleError::InvalidFeeCollector
    );

    require!(
        !presale.presale_canceled && !presale.presale_ended && !presale.presale_refund,
        PresaleError::PresaleFinalizationPreconditionsNotMet
    );

    let (expected_pool_state, _) = if ctx.accounts.pool_program.key() == raydium_cp_swap::ID {
        let (token_0_mint, token_1_mint) = match WRAPPED_SOL_MINT_ADDRESS <= presale.token {
            true => (WRAPPED_SOL_MINT_ADDRESS, presale.token),
            false => (presale.token, WRAPPED_SOL_MINT_ADDRESS),
        };
        Pubkey::find_program_address(
            &[
                raydium_cp_swap::states::POOL_SEED.as_bytes(),
                ctx.accounts.amm_config.key().as_ref(),
                token_0_mint.key().as_ref(),
                token_1_mint.key().as_ref(),
            ],
            &ctx.accounts.pool_program.key(),
        )
    } else {
        let (token_0_mint, token_1_mint) = match WRAPPED_SOL_MINT_ADDRESS >= presale.token {
            true => (WRAPPED_SOL_MINT_ADDRESS, presale.token),
            false => (presale.token, WRAPPED_SOL_MINT_ADDRESS),
        };
        Pubkey::find_program_address(
            &[
                token_0_mint.key().as_ref(),
                token_1_mint.key().as_ref(),
                ctx.accounts.amm_config.key().as_ref(),
            ],
            &ctx.accounts.pool_program.key(),
        )
    };

    require!(
        ctx.accounts.pool_state.key() == expected_pool_state,
        PresaleError::InvalidRaydiumPoolState
    );

    require!(
        ctx.accounts.pool_state.get_lamports() > 0,
        PresaleError::InvalidRaydiumAmmConfig
    );

    presale.presale_ended = true;

    if presale.total_raised >= presale.soft_cap {
        let service_fee_reserve = (presale.total_raised as u128)
            .checked_mul(presale.service_fee as u128)
            .and_then(|f| f.checked_div(10000))
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let total_raised_net = presale
            .total_raised
            .checked_sub(service_fee_reserve)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let affiliate_reward_pool_reserve = if presale.affiliate_enabled {
            (total_raised_net as u128)
                .checked_mul(presale.commission_rate as u128)
                .and_then(|f| f.checked_div(10000))
                .and_then(|f| u64::try_from(f).ok())
                .ok_or(PresaleError::ArithmeticOverflow)?
        } else {
            0u64
        };

        let liquidity_pool_reserve = (total_raised_net as u128)
            .checked_mul(presale.liquidity_bp as u128)
            .and_then(|f| f.checked_div(10000))
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let presale_owner_reward: u64 = total_raised_net
            .checked_sub(affiliate_reward_pool_reserve)
            .and_then(|f| f.checked_sub(liquidity_pool_reserve))
            .and_then(|f| f.checked_sub(presale.tokens_claimed_by_owner))
            .ok_or(PresaleError::ArithmeticOverflow)?;

        tranfer_sol_from_vault(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.fee_collector.to_account_info(),
            service_fee_reserve,
        )?;

        tranfer_sol_from_vault(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            presale_owner_reward,
        )?;

        if presale.presale_type == PresaleType::HardCapped
            && presale.refund_type == RefundType::Burn
        {
            // burn spl token
            let bump: u8 = ctx.bumps.presale;
            let token_key = ctx.accounts.token_mint.key();
            let signer: &[&[&[u8]]] = &[&[
                PRESALE_SEED,
                token_key.as_ref(),
                presale.identifier.as_ref(),
                &[bump],
            ]];

            let unsold_tokens = token_vault
                .amount
                .checked_sub(presale.total_tokens_sold)
                .ok_or(PresaleError::ArithmeticOverflow)?;

            burn(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        from: token_vault.to_account_info().clone(),
                        mint: ctx.accounts.token_mint.to_account_info(),
                        authority: presale.to_account_info().clone(),
                    },
                    signer,
                ),
                unsold_tokens,
            )?;
        }
    } else {
        presale.presale_refund = true;
    }

    emit!(PresaleFinalized {
        success: true,
        timestamp: current_time
    });

    Ok(())
}
