use crate::{
    constants::METEORA_POOL_AUTHORITY_SEED,
    dynamic_amm::{
        self,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        TokenAccount,
        TokenInterface,
    },
};

#[derive(Accounts)]
pub struct LockPoolMeteora<'info> {
    /// CHECK: Validated by dynamic_amm program
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Initialized by dynamic_amm program
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    /// CHECK
    #[account(
        mut,
        seeds = [METEORA_POOL_AUTHORITY_SEED],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub lp_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = creator_authority,
    )]
    pub owner_pool_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Validated by dynamic_amm program
    pub a_vault: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by dynamic_amm program
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: Initialized by dynamic_amm program
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow,
        payer = payer
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn finalize_presale_meteora_lock_pool(ctx: Context<LockPoolMeteora>) -> Result<()> {
    let seeds = [METEORA_POOL_AUTHORITY_SEED, &[ctx.bumps.creator_authority]];
    let creator_signer_seeds = &[&seeds[..]];

    let cpi_accounts_create_lock = dynamic_amm::cpi::accounts::CreateLockEscrow {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_context_create_lock = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        cpi_accounts_create_lock,
    )
    .with_signer(creator_signer_seeds);

    dynamic_amm::cpi::create_lock_escrow(cpi_context_create_lock)?;

    let cpi_accounts_lock = dynamic_amm::cpi::accounts::Lock {
        pool: ctx.accounts.pool.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        source_tokens: ctx.accounts.owner_pool_lp.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
    };

    let cpi_context_lock = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        cpi_accounts_lock,
    )
    .with_signer(creator_signer_seeds);

    dynamic_amm::cpi::lock(cpi_context_lock, ctx.accounts.owner_pool_lp.amount)?;

    Ok(())
}
