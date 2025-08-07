use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    constants::VAULT_SEED,
    error::PresaleError,
    state::{
        contribution::ContributionState,
        presale::PresaleState,
        vault::Vault,
    },
    utils::tranfer_sol_from_vault,
};

#[derive(Accounts)]
pub struct RefundContributors<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [crate::constants::CONTRIBUTE_SEED, presale.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, ContributionState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub token: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn refund_contributors(ctx: Context<RefundContributors>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.user,
        &ctx.accounts.system_program,
    )?;
    let contribution = &mut ctx.accounts.contribution;
    let now = Clock::get()?.unix_timestamp;

    if !presale.presale_refund
        && !presale.presale_ended
        && now > presale.end_time
        && presale.total_raised < presale.soft_cap
    {
        presale.presale_refund = true;
    }

    require!(presale.presale_refund, PresaleError::PresaleNotRefunded);
    require!(
        contribution.contributor == ctx.accounts.user.key(),
        PresaleError::Unauthorized
    );

    let refund_amount: u64 = contribution.amount;
    if refund_amount > 0 {
        contribution.amount = 0;
        contribution.tokens_purchased = 0;

        // Transfer SOL back to the user
        tranfer_sol_from_vault(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.user.to_account_info(),
            refund_amount,
        )?;
    }

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
