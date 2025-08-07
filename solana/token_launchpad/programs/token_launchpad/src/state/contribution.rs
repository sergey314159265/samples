use anchor_lang::prelude::*;

#[account]
pub struct ContributionState {
    pub contributor: Pubkey,
    pub amount: u64,
    pub tokens_purchased: u64,
}
