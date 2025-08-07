use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use crate::constants::{
    METEORA_POOL_AUTHORITY_SEED,
    WRAPPED_SOL_MINT_ADDRESS,
};

#[derive(Accounts)]
pub struct InitMeteoraPoolAuthority<'info> {
    /// CHECK: Pool creator authority. PDA.
    #[account(
        mut,
        seeds = [METEORA_POOL_AUTHORITY_SEED],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = wsol_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account()]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(address = WRAPPED_SOL_MINT_ADDRESS)]
    pub wsol_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

pub fn init_meteora_pool_authority(_ctx: Context<InitMeteoraPoolAuthority>) -> Result<()> {
    Ok(())
}
