use anchor_lang::prelude::*;

#[event]
pub struct LaunchpadCreated {
    pub launchpad: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FactoryInit {
    pub init: bool,
}

#[event]
pub struct FactoryConfigUpdated {
    pub creator_fee: u64,
    pub service_fee: u16,
    pub fee_collector: Pubkey,
}

#[event]
pub struct CreatorFeeUpdated {
    pub creator_fee: u64,
}

#[event]
pub struct ServiceFeeUpdated {
    pub service_fee: u16,
}

#[event]
pub struct FeeCollectorUpdated {
    pub fee_collector: Pubkey,
}

#[event]
pub struct AdminUpdated {
    pub admin: Pubkey,
}
