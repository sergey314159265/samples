use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct FeeCollectorUpdated {
    pub fee_collector: Pubkey,
}

#[derive(Accounts)]
pub struct SetFeeCollector<'info> {
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

pub fn set_fee_collector(ctx: Context<SetFeeCollector>) -> Result<()> {
    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;
    require!(
        factory.admin == *ctx.accounts.admin.key,
        FactoryError::Unauthorized
    );

    factory.fee_collector = ctx.accounts.fee_collector_info.key();

    emit!(FeeCollectorUpdated {
        fee_collector: ctx.accounts.fee_collector_info.key()
    });

    Ok(())
}
