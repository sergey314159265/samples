use anchor_lang::prelude::*;
use solana_program::{
    pubkey,
    pubkey::Pubkey,
};

#[constant]
pub const FACTORY_CONFIG: &[u8] = b"launchpad_factory_config";

#[constant]
pub const MAX_CREATOR_FEE: u64 = 10_000_000_000_u64;

#[constant]
pub const MAX_SERVICE_FEE: u16 = 2500;

#[constant]
pub const ADMIN: Pubkey = pubkey!("DnTFwYMHpKWy4U6tCCv5uDC8WiiAdkVJQ9fhqBbMXcd2");

#[constant]
pub const DEGEN_MIN_HARD_CAP: u16 = 10;

#[constant]
pub const DEGEN_MAX_HARD_CAP: u16 = 500;

#[constant]
pub const DEGEN_AUTOFINALIZATION_FEE_SOL: f64 = 0.0554;
