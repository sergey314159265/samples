use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{
        Mint,
        TokenAccount,
    },
};

use crate::{
    constants::{
        VAULT_SEED,
        WRAPPED_SOL_MINT_ADDRESS,
    },
    error::PresaleError,
    state::presale::{
        LaunchpadType,
        PresaleState,
        PresaleType,
    },
    utils::{
        is_authorized_to_finalize_presale,
        tranfer_sol_from_vault,
        validate_presale_pda,
    },
};

#[derive(Accounts)]
pub struct FinalizeTransfer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = wsol_mint,
        associated_token::authority = presale,
        associated_token::token_program = token_program
    )]
    pub vault_wsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = wsol_mint.key() == WRAPPED_SOL_MINT_ADDRESS
    )]
    pub wsol_mint: InterfaceAccount<'info, Mint>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn finalize_transfer(ctx: Context<FinalizeTransfer>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.signer,
        &ctx.accounts.system_program,
    )?;
    let now = Clock::get().unwrap().unix_timestamp;

    let _ = validate_presale_pda(
        presale,
        ctx.accounts.presale.key(),
        ctx.accounts.token_mint.key(),
    )?;

    require!(
        is_authorized_to_finalize_presale(presale, &ctx.accounts.signer)?,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    let service_fee_reserve = (presale.total_raised as u128)
        .checked_mul(presale.service_fee as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let total_raised_net = presale
        .total_raised
        .checked_sub(service_fee_reserve)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let liquidity_pool_sol_reserve = (total_raised_net as u128)
        .checked_mul(presale.liquidity_bp as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let liquidity_pool_token_reserve = match presale.presale_type {
        PresaleType::HardCapped => {
            let multiplier = 10u64
                .checked_pow(ctx.accounts.token_mint.decimals as u32)
                .ok_or(PresaleError::ArithmeticOverflow)?;

            (liquidity_pool_sol_reserve as u128)
                .checked_mul(multiplier as u128)
                .and_then(|f| match presale.launchpad_type == LaunchpadType::Degen {
                    true => f.checked_mul(100000000),
                    false => Some(f),
                })
                .and_then(|f| f.checked_div(presale.listing_rate as u128))
                .and_then(|f| u64::try_from(f).ok())
                .ok_or(PresaleError::ArithmeticOverflow)?
        }
        PresaleType::FairLaunch => {
            let net_rate_bp = 10000u16
                .checked_sub(presale.service_fee)
                .ok_or(PresaleError::ArithmeticOverflow)?;
            let net_tokens_allocated = (presale.total_tokens_sold as u128)
                .checked_mul(net_rate_bp as u128)
                .and_then(|f| f.checked_div(10000))
                .ok_or(PresaleError::ArithmeticOverflow)?;

            net_tokens_allocated
                .checked_mul(presale.liquidity_bp as u128)
                .and_then(|f| f.checked_div(10000))
                .and_then(|f| u64::try_from(f).ok())
                .ok_or(PresaleError::ArithmeticOverflow)?
        }
    };

    presale.sol_pool_reserve = liquidity_pool_sol_reserve;
    presale.token_pool_reserve = liquidity_pool_token_reserve;

    presale.serialize_data(&ctx.accounts.presale)?;

    tranfer_sol_from_vault(
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.vault_wsol_ata.to_account_info(),
        liquidity_pool_sol_reserve,
    )?;

    Ok(())
}
