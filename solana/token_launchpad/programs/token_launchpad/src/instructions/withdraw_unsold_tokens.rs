use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        transfer_checked,
        Mint,
        TokenAccount,
        TokenInterface,
        TransferChecked,
    },
};

use crate::{
    constants::PRESALE_SEED,
    error::PresaleError,
    state::presale::{
        LaunchpadType,
        PresaleState,
        PresaleType,
    },
    utils::{
        calculate_presale_data,
        calculate_presale_data_degen,
        validate_presale_pda,
    },
};

#[derive(Accounts)]
pub struct WithdrawUnsoldTokens<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = presale,
        associated_token::token_program = token_program
    )]
    pub token_vault_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_unsold_tokens(ctx: Context<WithdrawUnsoldTokens>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.owner,
        &ctx.accounts.system_program,
    )?;
    let token_mint = &mut ctx.accounts.token_mint;

    let presale_bump = validate_presale_pda(presale, ctx.accounts.presale.key(), token_mint.key())?;

    require!(presale.token == token_mint.key(), PresaleError::Invalid);

    require!(
        presale.owner == *ctx.accounts.owner.key,
        PresaleError::Unauthorized
    );

    if presale.presale_type == PresaleType::HardCapped {
        require!(
            presale.presale_ended || presale.presale_canceled,
            PresaleError::PresaleEndedOrCanceled
        );

        let decimal_result = 10u64
            .checked_pow(token_mint.decimals as u32)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let (actual_total_tokens_presale, hc_presale_tokens) = match presale.launchpad_type {
            LaunchpadType::Pro => {
                let (_, _, _, actual_total_tokens_presale) = calculate_presale_data(
                    u128::from(presale.total_raised),
                    u128::from(presale.service_fee),
                    u128::from(presale.liquidity_bp),
                    u128::from(decimal_result),
                    u128::from(presale.token_price),
                    u128::from(presale.listing_rate),
                )?;
                let (_, _, _, hc_presale_tokens) = calculate_presale_data(
                    u128::from(presale.hard_cap),
                    u128::from(presale.service_fee),
                    u128::from(presale.liquidity_bp),
                    u128::from(decimal_result),
                    u128::from(presale.token_price),
                    u128::from(presale.listing_rate),
                )?;

                (actual_total_tokens_presale, hc_presale_tokens)
            }
            LaunchpadType::Degen => {
                let (_, _, _, actual_total_tokens_presale) = calculate_presale_data_degen(
                    u128::from(presale.total_raised),
                    u128::from(presale.service_fee),
                    u128::from(presale.liquidity_bp),
                    u128::from(decimal_result),
                    u128::from(presale.token_price),
                    u128::from(presale.listing_rate),
                )?;
                let (_, _, _, hc_presale_tokens) = calculate_presale_data_degen(
                    u128::from(presale.hard_cap),
                    u128::from(presale.service_fee),
                    u128::from(presale.liquidity_bp),
                    u128::from(decimal_result),
                    u128::from(presale.token_price),
                    u128::from(presale.listing_rate),
                )?;

                (actual_total_tokens_presale, hc_presale_tokens)
            }
        };

        let token_vault_balance = ctx.accounts.token_vault_account.amount;

        let token_to_transfer = if presale.presale_ended {
            hc_presale_tokens
                .checked_sub(actual_total_tokens_presale)
                .ok_or(PresaleError::ArithmeticOverflow)?
        } else {
            token_vault_balance
        };

        let token_key = token_mint.key();

        let signer: &[&[&[u8]]] = &[&[
            PRESALE_SEED,
            token_key.as_ref(),
            presale.identifier.as_ref(),
            &[presale_bump],
        ]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    mint: token_mint.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer,
            ),
            token_to_transfer,
            ctx.accounts.token_mint.decimals,
        )?;
    } else if presale.presale_type == PresaleType::FairLaunch {
        require!(presale.presale_canceled, PresaleError::PresaleNotCancelled);

        let token_key = token_mint.key();
        let signer: &[&[&[u8]]] = &[&[
            PRESALE_SEED,
            token_key.as_ref(),
            presale.identifier.as_ref(),
            &[presale_bump],
        ]];

        let net_rate_bp = 10000u16
            .checked_sub(presale.service_fee)
            .ok_or(PresaleError::ArithmeticOverflow)?;
        let net_tokens_allocated = (presale.total_tokens_sold as u128)
            .checked_mul(net_rate_bp as u128)
            .and_then(|f| f.checked_div(10000))
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let lp_pool_reserve = net_tokens_allocated
            .checked_mul(presale.liquidity_bp as u128)
            .and_then(|f| f.checked_div(10000))
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(PresaleError::ArithmeticOverflow)?;
        let tokens_to_withdraw = presale
            .total_tokens_sold
            .checked_add(lp_pool_reserve)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    mint: token_mint.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer,
            ),
            tokens_to_withdraw,
            ctx.accounts.token_mint.decimals,
        )?;
    }

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
