use anchor_lang::prelude::*;

use crate::{
    error::PresaleError,
    state::presale::PresaleState,
};
#[event]
pub struct PresaleCanceled {
    pub success: bool,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct CancelPresale<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn cancel_presale(ctx: Context<CancelPresale>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.user,
        &ctx.accounts.system_program,
    )?;

    require!(
        presale.owner == *ctx.accounts.user.key,
        PresaleError::Unauthorized
    );

    require!(!presale.presale_ended, PresaleError::PresaleEnded);

    require!(
        !presale.owner_reward_withdrawn,
        PresaleError::OwnerRewardWithdrawn
    );

    presale.presale_canceled = true;
    presale.presale_refund = true;

    emit!(PresaleCanceled {
        success: true,
        timestamp: Clock::get().unwrap().unix_timestamp
    });

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
