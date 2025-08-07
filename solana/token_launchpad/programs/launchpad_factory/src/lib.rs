#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
use crate::instructions::*;
use token_launchpad::state::presale::*;

declare_id!("2e52Hn9bP9B1wJ6Ehy6T9y9Fmzd33poU3tSoCySYyqmj");

#[program]
pub mod launchpad_factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, creator_fee: u64, service_fee: u16) -> Result<()> {
        instructions::initialize(ctx, creator_fee, service_fee)
    }

    pub fn set_factory_config(
        ctx: Context<FactoryConfig>,
        creator_fee: u64,
        service_fee: u16,
    ) -> Result<()> {
        instructions::set_factory_config(ctx, creator_fee, service_fee)
    }

    pub fn set_creator_fee(ctx: Context<SetCreatorFee>, creator_fee: u64) -> Result<()> {
        instructions::set_creator_fee(ctx, creator_fee)
    }

    pub fn set_service_fee(ctx: Context<SetServiceFee>, service_fee: u16) -> Result<()> {
        instructions::set_service_fee(ctx, service_fee)
    }

    pub fn set_fee_collector(ctx: Context<SetFeeCollector>) -> Result<()> {
        instructions::set_fee_collector(ctx)
    }

    pub fn set_manager(ctx: Context<SetManager>) -> Result<()> {
        instructions::set_manager(ctx)
    }

    pub fn create_presale(
        ctx: Context<CreatePresale>,
        presale_type: PresaleType,
        tokens_allocated: u64,
        token_price: u64,
        hard_cap: u64,
        soft_cap: u64,
        min_contribution: u64,
        max_contribution: u64,
        start_time: i64,
        end_time: i64,
        listing_rate: u64,
        liquidity_lock_time: i64,
        liquidity_bp: u16,
        refund_type: RefundType,
        listing_opt: ListingOpt,
        liquidity_type: LiquidityType,
        liquidity_pool_provider: ListingPlatform,
        identifier: String,
        affiliate_enabled: bool,
        whitelist_enabled: bool,
        comm_rate: u16,
        launchpad_type: LaunchpadType,
    ) -> Result<()> {
        instructions::create_presale(
            ctx,
            presale_type,
            tokens_allocated,
            token_price,
            hard_cap,
            soft_cap,
            min_contribution,
            max_contribution,
            start_time,
            end_time,
            listing_rate,
            liquidity_lock_time,
            liquidity_bp,
            refund_type,
            listing_opt,
            liquidity_type,
            liquidity_pool_provider,
            identifier,
            affiliate_enabled,
            whitelist_enabled,
            comm_rate,
            launchpad_type,
        )
    }

    pub fn set_admin(ctx: Context<SetAdmin>) -> Result<()> {
        instructions::set_admin(ctx)
    }
}
