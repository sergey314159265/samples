use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::{
        transfer_checked,
        TransferChecked,
    },
    token_interface::{
        Mint,
        TokenAccount,
    },
};

use crate::{
    constants::LP_TOKEN_LOCK_SEED,
    error::PresaleError,
    state::{
        lock::LiquidityLock,
        presale::PresaleState,
    },
    utils::validate_presale_pda,
};

#[derive(Accounts)]
pub struct WithdrawLockedLpTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(
        seeds = [LP_TOKEN_LOCK_SEED, presale.key().as_ref()],
        bump,
    )]
    pub lp_token_lock: Box<Account<'info, LiquidityLock>>,

    #[account(
        associated_token::mint = lp_mint,
        associated_token::authority = lp_token_lock,
        associated_token::token_program = token_program
    )]
    pub lp_token_lock_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub creator_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn withdraw_locked_lp_tokens(ctx: Context<WithdrawLockedLpTokens>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.signer,
        &ctx.accounts.system_program,
    )?;
    let lock = &mut ctx.accounts.lp_token_lock;
    let now = Clock::get().unwrap().unix_timestamp;

    let _ = validate_presale_pda(
        presale,
        ctx.accounts.presale.key(),
        ctx.accounts.token_mint.key(),
    )?;

    require!(
        lock.owner == *ctx.accounts.signer.key,
        PresaleError::Unauthorized
    );

    require!(now > lock.unlock_time, PresaleError::LiquidityLocked);

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.lp_token_lock_ata.to_account_info(),
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.creator_lp_token.to_account_info(),
                authority: ctx.accounts.signer.to_account_info().clone(),
            },
        ),
        lock.locked_amount,
        9,
    )?;

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
