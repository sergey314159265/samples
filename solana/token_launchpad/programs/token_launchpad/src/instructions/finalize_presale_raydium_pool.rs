use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::TransferChecked,
    token_interface::{
        transfer_checked,
        Mint,
        TokenAccount,
        TokenInterface,
    },
};
use raydium_cp_swap::{
    program::RaydiumCpSwap,
    states::AmmConfig,
};

use crate::{
    constants::{
        PRESALE_SEED,
        VAULT_SEED,
        WRAPPED_SOL_MINT_ADDRESS,
    },
    error::PresaleError,
    state::{
        presale::{
            ListingPlatform,
            PresaleState,
        },
        vault::Vault,
    },
    utils::{
        get_transfer_inverse_fee,
        is_authorized_to_finalize_presale,
    },
};

#[derive(Accounts)]
pub struct FinalizePresaleRaydiumPool<'info> {
    pub cp_swap_program: Program<'info, RaydiumCpSwap>,
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: pool vault and lp mint authority
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(mut)]
    pub pool_state: UncheckedAccount<'info>,

    /// Token_0 mint, the key must smaller then token_1 mint.
    pub token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token_1 mint, the key must grater then token_0 mint.
    pub token_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// payer token0 account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_0_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_0_program
    )]
    pub creator_token_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// creator token1 account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_1_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_1_program
    )]
    pub creator_token_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,

    /// CHECK: Token_0 vault for the pool, init by cp-swap
    #[account(mut)]
    pub token_0_vault: UncheckedAccount<'info>,

    /// CHECK: Token_1 vault for the pool, init by cp-swap
    #[account(mut)]
    pub token_1_vault: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(mut)]
    pub token_vault_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub vault_wsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// create pool fee account
    #[account(mut)]
    pub create_pool_fee: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: an account to store oracle observations, init by cp-swap
    #[account(mut)]
    pub observation_state: UncheckedAccount<'info>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Spl token program or token program 2022
    pub token_0_program: Interface<'info, TokenInterface>,
    /// Spl token program or token program 2022
    pub token_1_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn finalize_presale_raydium_pool(ctx: Context<FinalizePresaleRaydiumPool>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let token_mint = &ctx.accounts.token_mint;
    let now = Clock::get().unwrap().unix_timestamp;
    let liquidity_pool_sol_reserve = presale.sol_pool_reserve;
    let liquidity_pool_token_reserve = presale.token_pool_reserve;

    require!(
        presale.listing_platform == ListingPlatform::Raydium,
        PresaleError::InvalidListingPlatform
    );

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    let token_transfer_fee = get_transfer_inverse_fee(
        &ctx.accounts.token_mint.to_account_info(),
        liquidity_pool_token_reserve,
    )?;
    let liquidity_pool_token_reserve_with_fee = liquidity_pool_token_reserve
        .checked_add(token_transfer_fee)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let cpi_accounts = raydium_cp_swap::cpi::accounts::Initialize {
        creator: ctx.accounts.signer.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        token_0_mint: ctx.accounts.token_0_mint.to_account_info(),
        token_1_mint: ctx.accounts.token_1_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        creator_token_0: ctx.accounts.creator_token_0.to_account_info(),
        creator_token_1: ctx.accounts.creator_token_1.to_account_info(),
        creator_lp_token: ctx.accounts.creator_lp_token.to_account_info(),
        token_0_vault: ctx.accounts.token_0_vault.to_account_info(),
        token_1_vault: ctx.accounts.token_1_vault.to_account_info(),
        create_pool_fee: ctx.accounts.create_pool_fee.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_0_program: ctx.accounts.token_0_program.to_account_info(),
        token_1_program: ctx.accounts.token_1_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let token_mint_key = token_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        PRESALE_SEED,
        token_mint_key.as_ref(),
        presale.identifier.as_ref(),
        &[ctx.bumps.presale],
    ]];

    let cpi_context = CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts)
        .with_signer(signer_seeds);

    //Transfer tokens and wsol from vault to signer for liquidity pool usage
    let (
        signer_wsol_token,
        wsol_mint,
        wsol_token_program,
        token_0_amount,
        signer_token,
        token_mint,
        token_ata_program,
        token_1_amount,
    ) = match ctx.accounts.token_0_mint.key() == WRAPPED_SOL_MINT_ADDRESS {
        true => (
            ctx.accounts.creator_token_0.as_ref(),
            ctx.accounts.token_0_mint.as_ref(),
            ctx.accounts.token_0_program.as_ref(),
            liquidity_pool_sol_reserve,
            ctx.accounts.creator_token_1.as_ref(),
            ctx.accounts.token_1_mint.as_ref(),
            ctx.accounts.token_1_program.as_ref(),
            liquidity_pool_token_reserve,
        ),
        false => (
            ctx.accounts.creator_token_1.as_ref(),
            ctx.accounts.token_1_mint.as_ref(),
            ctx.accounts.token_1_program.as_ref(),
            liquidity_pool_token_reserve,
            ctx.accounts.creator_token_0.as_ref(),
            ctx.accounts.token_0_mint.as_ref(),
            ctx.accounts.token_0_program.as_ref(),
            liquidity_pool_sol_reserve,
        ),
    };

    transfer_checked(
        CpiContext::new_with_signer(
            wsol_token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_wsol_ata.to_account_info(),
                mint: wsol_mint.to_account_info(),
                to: signer_wsol_token.to_account_info(),
                authority: presale.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity_pool_sol_reserve,
        wsol_mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new_with_signer(
            token_ata_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.token_vault_account.to_account_info(),
                mint: token_mint.to_account_info(),
                to: signer_token.to_account_info(),
                authority: presale.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity_pool_token_reserve_with_fee,
        token_mint.decimals,
    )?;

    raydium_cp_swap::cpi::initialize(cpi_context, token_0_amount, token_1_amount, 0)?;

    Ok(())
}
