use anchor_lang::prelude::*;
use solana_program::{
    pubkey,
    pubkey::Pubkey,
};

#[constant]
pub const PRESALE_SEED: &[u8] = b"presale";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const LP_TOKEN_LOCK_SEED: &[u8] = b"lp_token_lock";

#[constant]
pub const TOKEN_VAULT_SEED: &[u8] = b"token_vault";

#[constant]
pub const FACTORY_CONFIG: &[u8] = b"launchpad_factory_config";

#[constant]
pub const CONTRIBUTE_SEED: &[u8] = b"contribute";

#[constant]
pub const REFERRER_SEED: &[u8] = b"referrer";

#[constant]
pub const METEORA_POOL_AUTHORITY_SEED: &[u8] = b"meteora_pool_authority";

#[constant]
pub const FACTORY_PROGRAM_ID: Pubkey = pubkey!("2e52Hn9bP9B1wJ6Ehy6T9y9Fmzd33poU3tSoCySYyqmj");

#[constant]
pub const WRAPPED_SOL_MINT_ADDRESS: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

#[constant]
pub const ADMIN_FINALIZATION_TIMEOUT: i64 = 72 * 3600; // 72 hours in seconds

// Determines the percentage of fee, expressed in basis points, to be distributed to the
// fee collector
// Example: value = 6000 - 60% to the fee colletor, 40% to the owner of presale
#[constant]
pub const METEORA_FEE_DISTRIBUTION: u16 = 5000;

#[constant]
pub const PRESALE_VERSION: u8 = 1;
