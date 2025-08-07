use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        self,
        Token,
    },
    token_interface::{
        Mint,
        TokenAccount,
    },
};

use crate::{
    constants::{
        PRESALE_SEED,
        WRAPPED_SOL_MINT_ADDRESS,
    },
    error::PresaleError,
    state::presale::PresaleState,
    utils::is_authorized_to_finalize_presale,
};

#[derive(Accounts)]
pub struct FinalizeWrapSol<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    #[account(mut)]
    pub vault_wsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = wsol_mint.key() == WRAPPED_SOL_MINT_ADDRESS
    )]
    pub wsol_mint: InterfaceAccount<'info, Mint>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn finalize_wrap_sol(ctx: Context<FinalizeWrapSol>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let now = Clock::get().unwrap().unix_timestamp;

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    let cpi_accounts = token::SyncNative {
        account: ctx.accounts.vault_wsol_ata.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::sync_native(cpi_ctx)?;

    Ok(())
}
