use anchor_lang::prelude::*;

pub const STAKE_POOL_DEFAULT_SIZE: usize = 8 + 1 + 32 + 8 + 4 + 5 + 8 + 32 + 24 + 8;
pub const STAKE_POOL_PREFIX: &str = "stake-pool";
#[account]
pub struct StakePool {
    pub bump: u8,
    pub authority: Pubkey,
    pub total_staked: u64,
    pub total_stakers: u32,
    pub min_stake_seconds: Option<u32>,
    pub stake_reward: u64,
    pub mint: Pubkey,
    pub identifier: String,
}
