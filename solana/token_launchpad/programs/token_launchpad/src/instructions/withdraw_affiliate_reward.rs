use anchor_lang::prelude::*;

use crate::{
    constants::{
        REFERRER_SEED,
        VAULT_SEED,
    },
    error::PresaleError,
    state::{
        affiliate::AffiliateReferrerState,
        presale::PresaleState,
        vault::Vault,
    },
    utils::withdraw_commission,
};
#[event]
pub struct CommissionWithdrawn {
    pub referrer: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct WithdrawAffiliateCommission<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [REFERRER_SEED, presale.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub affiliate_referrer_state: Account<'info, AffiliateReferrerState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_affiliate_commission(ctx: Context<WithdrawAffiliateCommission>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.user,
        &ctx.accounts.system_program,
    )?;
    require!(presale.presale_ended, PresaleError::Invalid);
    withdraw_commission(
        presale,
        &mut ctx.accounts.affiliate_referrer_state,
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.user.to_account_info(),
    )?;

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
