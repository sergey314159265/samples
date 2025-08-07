use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::StakePool;
#[derive(Accounts)]
pub struct UpdatePoolCtx<'info> {
    #[account(
        mut,
        constraint = stake_pool.authority == payer.key() @ErrorCode::InvalidAdmin,
    )]
    stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn reward_handler(ctx: Context<UpdatePoolCtx>, stake_reward: u64) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    require!(stake_reward > 0 && stake_reward < 10000, ErrorCode::InvalidInput);
    stake_pool.stake_reward = stake_reward;
    Ok(())
}

pub fn time_handler(ctx: Context<UpdatePoolCtx>, min_stake_seconds: u32) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    require!(min_stake_seconds > 0, ErrorCode::InvalidInput);
    stake_pool.min_stake_seconds = Some(min_stake_seconds);
    Ok(())
}
