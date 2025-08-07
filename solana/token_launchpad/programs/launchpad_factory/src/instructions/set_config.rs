use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct FactoryConfigUpdated {
    pub creator_fee: u64,
    pub service_fee: u16,
    pub fee_collector: Pubkey,
}

#[derive(Accounts)]
pub struct FactoryConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_CONFIG],
        bump,
    )]
    pub factory_config: Box<Account<'info, Factory>>,
    /// CHECK
    #[account(mut)]
    pub fee_collector_info: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn set_factory_config(
    ctx: Context<FactoryConfig>,
    creator_fee: u64,
    service_fee: u16,
) -> Result<()> {
    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;
    require!(
        factory.admin == *ctx.accounts.admin.key,
        FactoryError::Unauthorized
    );
    require!(creator_fee <= MAX_CREATOR_FEE, FactoryError::Invalid);
    require!(service_fee <= MAX_SERVICE_FEE, FactoryError::Invalid);

    factory.creator_fee = creator_fee;
    factory.service_fee = service_fee;
    factory.fee_collector = ctx.accounts.fee_collector_info.key();

    emit!(FactoryConfigUpdated {
        creator_fee,
        service_fee,
        fee_collector: ctx.accounts.fee_collector_info.key()
    });

    Ok(())
}
