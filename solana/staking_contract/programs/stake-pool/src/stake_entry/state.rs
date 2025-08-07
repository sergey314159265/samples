use anchor_lang::prelude::*;

pub const STAKE_ENTRY_PREFIX: &str = "stake-entry";
pub const STAKE_ENTRY_SIZE: usize = 8 + std::mem::size_of::<StakeEntry>() + 8;
#[account]
pub struct StakeEntry {
    pub bump: u8,
    pub staker: Pubkey,
    pub stake_pool: Pubkey,
    pub amount: u64,
    pub last_staked_at: i64,
    pub identifier: String,
}