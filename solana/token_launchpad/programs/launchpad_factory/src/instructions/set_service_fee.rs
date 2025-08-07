use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct ServiceFeeUpdated {
    pub service_fee: u16,
}

#[derive(Accounts)]
pub struct SetServiceFee<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_CONFIG],
        bump,
    )]
    pub factory_config: Box<Account<'info, Factory>>,
    pub system_program: Program<'info, System>,
}

pub fn set_service_fee(ctx: Context<SetServiceFee>, service_fee: u16) -> Result<()> {
    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;
    require!(
        factory.admin == *ctx.accounts.admin.key,
        FactoryError::Unauthorized
    );
    require!(service_fee <= MAX_SERVICE_FEE, FactoryError::Invalid);

    factory.service_fee = service_fee;

    emit!(ServiceFeeUpdated { service_fee });

    Ok(())
}
