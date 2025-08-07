use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::{
        burn,
        transfer_checked,
        Burn,
        TransferChecked,
    },
    token_interface::{
        Mint,
        TokenAccount,
    },
};

use crate::{
    constants::{
        LP_TOKEN_LOCK_SEED,
        PRESALE_SEED,
    },
    error::PresaleError,
    state::{
        lock::LiquidityLock,
        presale::{
            LiquidityType,
            PresaleState,
        },
    },
    utils::is_authorized_to_finalize_presale,
};

#[derive(Accounts)]
pub struct FinalizeLpLockBurn<'info> {
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    #[account(
        init,
        payer = signer,
        seeds = [LP_TOKEN_LOCK_SEED, presale.key().as_ref()],
        bump,
        space = 8 + LiquidityLock::INIT_SPACE
    )]
    pub lp_token_lock: Box<Account<'info, LiquidityLock>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = lp_token_lock,
        associated_token::token_program = token_program
    )]
    pub lp_token_lock_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
}

pub fn finalize_lp_lock_burn(ctx: Context<FinalizeLpLockBurn>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let now = Clock::get()?.unix_timestamp;

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    match presale.liquidity_type {
        LiquidityType::Lock => {
            let lock = &mut ctx.accounts.lp_token_lock;

            lock.owner = presale.owner;
            lock.locked_amount = ctx.accounts.creator_lp_token.amount;
            lock.unlock_time = now + presale.liquidity_lock_time;

            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.creator_lp_token.to_account_info(),
                        mint: ctx.accounts.lp_mint.to_account_info(),
                        to: ctx.accounts.lp_token_lock_ata.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info().clone(),
                    },
                ),
                lock.locked_amount,
                9,
            )?;
        }
        LiquidityType::Burn => {
            burn(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        from: ctx.accounts.creator_lp_token.to_account_info(),
                        mint: ctx.accounts.lp_mint.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info().clone(),
                    },
                ),
                ctx.accounts.creator_lp_token.amount,
            )?;
        }
    };

    Ok(())
}
