use crate::{
    constants::PRESALE_SEED,
    dynamic_vault,
    error::PresaleError,
    state::presale::{
        ListingPlatform,
        PresaleState,
    },
    utils::is_authorized_to_finalize_presale,
};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct FinalizePresaleInitVaultMeteora<'info> {
    /// CHECK: Initialized by vault program
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, token_mint.key().as_ref(), presale.identifier.as_ref()],
        bump,
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    /// CHECK: Initialized by vault program
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,

    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Initialized by vault program
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by vault program
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Vault program.
    #[account(address = dynamic_vault::ID)]
    pub vault_program: UncheckedAccount<'info>,
}

pub fn finalize_presale_init_vault_meteora(
    ctx: Context<FinalizePresaleInitVaultMeteora>,
) -> Result<()> {
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

    let cpi_accounts = dynamic_vault::cpi::accounts::Initialize {
        vault: ctx.accounts.vault.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        token_vault: ctx.accounts.token_vault.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.vault_program.to_account_info(), cpi_accounts);

    dynamic_vault::cpi::initialize(cpi_context)?;

    Ok(())
}
