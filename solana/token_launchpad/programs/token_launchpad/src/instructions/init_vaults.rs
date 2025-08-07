use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use crate::{
    constants::{
        FACTORY_PROGRAM_ID,
        VAULT_SEED,
    },
    error::PresaleError,
    state::{
        presale::PresaleState,
        vault::Vault,
    },
};

#[derive(Accounts)]
pub struct InitializeVaults<'info> {
    #[account(mut)]
    pub presale: Box<Account<'info, PresaleState>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Vault>()
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = token,
        associated_token::authority = presale,
        associated_token::token_program = token_program,

    )]
    pub token_vault_account: InterfaceAccount<'info, TokenAccount>,

    pub token: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK
    #[account(signer, owner = FACTORY_PROGRAM_ID)]
    pub factory_pda: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn init_vaults(ctx: Context<InitializeVaults>) -> Result<()> {
    require!(
        ctx.accounts.factory_pda.owner.key() == FACTORY_PROGRAM_ID,
        PresaleError::Unauthorized
    );
    require!(
        ctx.accounts.factory_pda.is_signer,
        PresaleError::Unauthorized
    );

    let vault = &mut ctx.accounts.vault;
    vault.authority = ctx.accounts.owner.key();
    Ok(())
}
