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
    constants::METEORA_POOL_AUTHORITY_SEED,
    dynamic_amm::{
        self,
        accounts::Pool,
    },
    dynamic_vault,
    state::presale::PresaleState, utils::{is_authorized_to_finalize_presale, validate_presale_pda},
};

#[derive(Accounts)]
pub struct ClaimFeeMeteora<'info> {
    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Pool LP mint
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    /// CHECK: Pool creator authority. PDA.
    #[account(
        mut,
        seeds = [METEORA_POOL_AUTHORITY_SEED],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Lock escrow of creator PDA
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Token account of vault A
    #[account(mut)]
    pub a_token_vault: UncheckedAccount<'info>,

    /// CHECK: Token account of vault B
    #[account(mut)]
    pub b_token_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token a. token a of the pool will be deposit / withdraw from this
    /// vault account.
    #[account(mut)]
    pub a_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token b. token b of the pool will be deposit / withdraw from this
    /// vault account.
    #[account(mut)]
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw
    /// from the vault.
    #[account(mut)]
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw
    /// from the vault.
    #[account(mut)]
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault a
    #[account(mut)]
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault b
    #[account(mut)]
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub creator_a_token: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut, 
        associated_token::mint = token_b_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub creator_b_token: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Protocol fee token account for token A. Used to receive trading fee.
    pub protocol_token_fee: UncheckedAccount<'info>,

    #[account(address = pool.token_a_mint)]
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = pool.token_b_mint)]
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: Token program
    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Dynamic AMM
    #[account(
        address = dynamic_amm::ID
    )]
    pub dynamic_amm: UncheckedAccount<'info>,

    /// CHECK: Dynamic vault
    #[account(
        address = dynamic_vault::ID
    )]
    pub dynamic_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn claim_fee_meteora(ctx: Context<ClaimFeeMeteora>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.signer,
        &ctx.accounts.system_program,
    )?;

    let _ = validate_presale_pda(
        presale,
        ctx.accounts.presale.key(),
        ctx.accounts.token_a_mint.key(),
    )?;

    is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?;

    let accounts = dynamic_amm::cpi::accounts::ClaimFee {
        pool: ctx.accounts.pool.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        source_tokens: ctx.accounts.escrow_vault.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        user_a_token: ctx.accounts.creator_a_token.to_account_info(),
        user_b_token: ctx.accounts.creator_b_token.to_account_info(),
        vault_program: ctx.accounts.dynamic_vault.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
        b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
    };

    let seeds = [METEORA_POOL_AUTHORITY_SEED, &[ctx.bumps.creator_authority]];
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.dynamic_amm.to_account_info(),
        accounts,
        signer_seeds,
    );

    dynamic_amm::cpi::claim_fee(cpi_context, u64::MAX)?;

    let swap_accounts = dynamic_amm::cpi::accounts::Swap {
        pool: ctx.accounts.pool.to_account_info(),
        user_source_token: ctx.accounts.creator_a_token.to_account_info(),
        user_destination_token: ctx.accounts.creator_b_token.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
        b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        protocol_token_fee: ctx.accounts.protocol_token_fee.to_account_info(),
        user: ctx.accounts.creator_authority.to_account_info(),
        vault_program: ctx.accounts.dynamic_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.dynamic_amm.to_account_info(),
        swap_accounts,
        signer_seeds,
    );

    ctx.accounts.creator_a_token.reload()?;
    let amount_to_swap = ctx.accounts.creator_a_token.amount;

    dynamic_amm::cpi::swap(cpi_context, amount_to_swap, 0)?;

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
