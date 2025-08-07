use anchor_lang::prelude::*;

#[account]
pub struct AffiliateReferrerState {
    pub referrer: Pubkey,
    pub total_sale: u64,
    pub is_reward_claimed: bool,
}
