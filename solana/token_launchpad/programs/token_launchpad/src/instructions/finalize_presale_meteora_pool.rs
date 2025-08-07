use crate::{
    constants::METEORA_POOL_AUTHORITY_SEED,
    dynamic_amm,
    dynamic_vault,
    state::presale::ListingPlatform,
    utils::{
        is_authorized_to_finalize_presale,
        transfer_sols,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        transfer_checked,
        TransferChecked,
    },
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use crate::{
    constants::PRESALE_SEED,
    error::PresaleError,
    state::presale::PresaleState,
};

#[derive(Accounts)]
pub struct FinalizePresaleMeteoraPool<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK
    #[account(
        mut,
        seeds = [METEORA_POOL_AUTHORITY_SEED],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Config account
    pub config: UncheckedAccount<'info>,

    /// CHECK: LP token mint of the pool
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Token A mint account. This is the mint of token A of the pool.
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: Token B mint account. This is the mint of token B of the pool.
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: Vault account for token A. Token A of the pool will be deposit / withdraw from this
    /// vault account.
    #[account(mut)]
    pub a_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Vault account for token B. Token B of the pool will be deposit / withdraw from this
    /// vault account.
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: Token vault account of vault A
    #[account(mut)]
    pub a_token_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Token vault account of vault B
    pub b_token_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: LP token mint of vault A
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: LP token mint of vault B
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw
    /// from the vault.
    #[account(mut)]
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault B. Used to receive/burn vault LP upon deposit/withdraw
    /// from the vault.
    #[account(mut)]
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: Payer pool LP token account. Used to receive LP during first deposit (initialize
    /// pool)
    #[account(mut)]
    pub payer_pool_lp: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Protocol fee token account for token A. Used to receive trading fee.
    pub protocol_token_a_fee: UncheckedAccount<'info>,

    /// CHECK: Protocol fee token account for token B. Used to receive trading fee.
    #[account(mut)]
    pub protocol_token_b_fee: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub payer_token_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = creator_authority,
        associated_token::token_program = token_program,
    )]
    pub payer_token_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub token_vault_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub vault_wsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Payer account. This account will be the creator of the pool, and the payer for PDA
    /// during initialize pool.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Rent account.
    pub rent: Sysvar<'info, Rent>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: LP mint metadata PDA. Metaplex do the checking.
    #[account(mut)]
    pub mint_metadata: UncheckedAccount<'info>,

    /// CHECK: Metadata program
    pub metadata_program: UncheckedAccount<'info>,

    /// CHECK: Vault program. The pool will deposit/withdraw liquidity from the vault.
    #[account(address = dynamic_vault::ID)]
    pub vault_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,
}

pub fn finalize_presale_meteora_pool(ctx: Context<FinalizePresaleMeteoraPool>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let now = Clock::get()?.unix_timestamp;

    require!(
        presale.listing_platform == ListingPlatform::Meteora,
        PresaleError::InvalidListingPlatform
    );

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.payer)?,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    let token_mint_key = ctx.accounts.token_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        PRESALE_SEED,
        token_mint_key.as_ref(),
        presale.identifier.as_ref(),
        &[ctx.bumps.presale],
    ]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_wsol_ata.to_account_info(),
                mint: ctx.accounts.token_b_mint.to_account_info(),
                to: ctx.accounts.payer_token_b.to_account_info(),
                authority: presale.to_account_info(),
            },
            signer_seeds,
        ),
        presale.sol_pool_reserve,
        ctx.accounts.token_b_mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.token_vault_account.to_account_info(),
                mint: ctx.accounts.token_a_mint.to_account_info(),
                to: ctx.accounts.payer_token_a.to_account_info(),
                authority: presale.to_account_info(),
            },
            signer_seeds,
        ),
        presale.token_pool_reserve,
        ctx.accounts.token_a_mint.decimals,
    )?;

    let amount = 34290400;
    transfer_sols(
        &ctx.accounts.payer,
        &ctx.accounts.creator_authority,
        &ctx.accounts.system_program,
        amount,
    )?;

    let cpi_account =
        dynamic_amm::cpi::accounts::InitializePermissionlessConstantProductPoolWithConfig {
            pool: ctx.accounts.pool.to_account_info(),
            config: ctx.accounts.config.to_account_info(),
            lp_mint: ctx.accounts.lp_mint.to_account_info(),
            token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
            token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
            a_vault: ctx.accounts.a_vault.to_account_info(),
            b_vault: ctx.accounts.b_vault.to_account_info(),
            a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
            b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
            a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
            b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
            a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
            b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
            payer_token_a: ctx.accounts.payer_token_a.to_account_info(),
            payer_token_b: ctx.accounts.payer_token_b.to_account_info(),
            payer_pool_lp: ctx.accounts.payer_pool_lp.to_account_info(),
            protocol_token_a_fee: ctx.accounts.protocol_token_a_fee.to_account_info(),
            protocol_token_b_fee: ctx.accounts.protocol_token_b_fee.to_account_info(),
            payer: ctx.accounts.creator_authority.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            mint_metadata: ctx.accounts.mint_metadata.to_account_info(),
            metadata_program: ctx.accounts.metadata_program.to_account_info(),
            vault_program: ctx.accounts.vault_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

    let seeds = [METEORA_POOL_AUTHORITY_SEED, &[ctx.bumps.creator_authority]];
    let creator_signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        cpi_account,
    )
    .with_signer(creator_signer_seeds);

    dynamic_amm::cpi::initialize_permissionless_constant_product_pool_with_config(
        cpi_context,
        presale.token_pool_reserve,
        presale.sol_pool_reserve,
    )?;

    Ok(())
}
