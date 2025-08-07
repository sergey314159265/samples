use anchor_lang::{
    prelude::*,
    system_program::Transfer,
};
use anchor_spl::{
    token::{
        self,
        transfer_checked,
        CloseAccount,
        TransferChecked,
    },
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use anchor_lang::system_program::transfer;

use crate::{
    constants::{
        METEORA_FEE_DISTRIBUTION,
        METEORA_POOL_AUTHORITY_SEED,
        PRESALE_SEED,
        WRAPPED_SOL_MINT_ADDRESS,
    },
    error::PresaleError,
    state::presale::PresaleState,
    utils::is_authorized_to_finalize_presale,
};

#[derive(Accounts)]
pub struct DistributeFeeMeteora<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump
    )]
    pub presale: Account<'info, PresaleState>,

    /// CHECK: Pool creator authority. PDA.
    #[account(
        mut,
        seeds = [METEORA_POOL_AUTHORITY_SEED],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub fee_collector: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        associated_token::mint = wsol_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub pool_authority_wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // Temporary wsol vault to avoid closing the main wsol vault during sol unwraping
    #[account(
        init,
        payer = signer,
        seeds = [b"temp", presale.key().as_ref()],
        bump,
        token::mint = wsol_mint,
        token::authority = creator_authority,
        token::token_program = token_program,
    )]
    pub temporary_wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = presale.token)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = WRAPPED_SOL_MINT_ADDRESS)]
    pub wsol_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Token program
    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

pub fn distribute_fee_meteora(
    ctx: Context<DistributeFeeMeteora>,
    minimum_amount: u64,
) -> Result<()> {
    let presale = &mut ctx.accounts.presale;

    is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?;

    require!(
        ctx.accounts.fee_collector.key() == presale.fee_collector,
        PresaleError::InvalidFeeCollector
    );

    require!(
        ctx.accounts.owner.key() == presale.owner,
        PresaleError::InvalidFeeCollector
    );

    let fee_in_wsol = ctx.accounts.pool_authority_wsol_vault.amount;
    let fee_collector_amount = fee_in_wsol
        .checked_mul(METEORA_FEE_DISTRIBUTION.into())
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let owner_amount = fee_in_wsol
        .checked_sub(fee_collector_amount)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    require!(
        fee_collector_amount > minimum_amount,
        PresaleError::PlatformProfitTooLow
    );

    let seeds = [METEORA_POOL_AUTHORITY_SEED, &[ctx.bumps.creator_authority]];
    let signer_seeds = &[&seeds[..]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.pool_authority_wsol_vault.to_account_info(),
                to: ctx.accounts.temporary_wsol_vault.to_account_info(),
                mint: ctx.accounts.wsol_mint.to_account_info(),
                authority: ctx.accounts.creator_authority.to_account_info(),
            },
            signer_seeds,
        ),
        fee_in_wsol,
        ctx.accounts.wsol_mint.decimals,
    )?;

    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.temporary_wsol_vault.to_account_info(),
            destination: ctx.accounts.creator_authority.to_account_info(),
            authority: ctx.accounts.creator_authority.to_account_info(),
        },
        signer_seeds,
    ))?;

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator_authority.to_account_info(),
                to: ctx.accounts.owner.to_account_info(),
            },
            signer_seeds,
        ),
        owner_amount,
    )?;

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator_authority.to_account_info(),
                to: ctx.accounts.signer.to_account_info(),
            },
            signer_seeds,
        ),
        Rent::get()?.minimum_balance(token::TokenAccount::LEN),
    )?;

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator_authority.to_account_info(),
                to: ctx.accounts.fee_collector.to_account_info(),
            },
            signer_seeds,
        ),
        fee_collector_amount,
    )?;

    Ok(())
}
