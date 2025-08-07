use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LiquidityLock {
    pub owner: Pubkey,
    pub unlock_time: i64,
    pub locked_amount: u64,
}
