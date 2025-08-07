use anchor_lang::prelude::*;

#[account]
pub struct Factory {
    pub admin: Pubkey,
    pub creator_fee: u64,
    pub service_fee: u16,
    pub fee_collector: Pubkey,
    pub manager: Pubkey,
}
