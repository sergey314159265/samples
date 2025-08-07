use anchor_lang::prelude::*;
pub mod stake_pool;
pub use stake_pool::*;

pub mod stake_entry;
pub use stake_entry::*;

pub mod stake_unstake;
pub use stake_unstake::*;

pub mod errors;

declare_id!("FbSXzbQNgxERQkYzMsnyg7ckSKCCANHo62k23ULuF39Z");

#[program]
mod staking_22 {
    use super::*;
    pub fn init_pool(ctx: Context<InitPoolCtx>, ix: InitPoolIx) -> Result<()> {
        stake_pool::init_pool::handler(ctx, ix)
    }

    pub fn update_pool_reward(ctx: Context<UpdatePoolCtx>, stake_reward: u64) -> Result<()> {
        stake_pool::update_pool::reward_handler(ctx, stake_reward)
    }

    pub fn update_pool_stake_time(
        ctx: Context<UpdatePoolCtx>,
        min_stake_seconds: u32,
    ) -> Result<()> {
        stake_pool::update_pool::time_handler(ctx, min_stake_seconds)
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokensCtx>, amount: u64) -> Result<()> {
        stake_pool::withdraw::handler(ctx, amount)
    }

    pub fn deposite_tokens(ctx: Context<DepositeTokensCtx>, amount: u64) -> Result<()> {
        stake_pool::deposite::handler(ctx, amount)
    }

    pub fn init_entry(ctx: Context<InitEntryCtx>, identifier: String) -> Result<()> {
        stake_entry::init_entry::handler(ctx, identifier)
    }

    pub fn stake_tokens(ctx: Context<StakeTokenCtx>, amount: u64) -> Result<()> {
        stake_unstake::stake::handler(ctx, amount)
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokenCtx>) -> Result<()> {
        stake_unstake::unstake::handler(ctx)
    }
}