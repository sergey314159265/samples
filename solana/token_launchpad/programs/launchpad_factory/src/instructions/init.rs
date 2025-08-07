use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct FactoryInit {
    pub init: bool,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut, constraint = admin.key() == ADMIN @FactoryError::Unauthorized)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [FACTORY_CONFIG],
        bump,
        space = 8 + std::mem::size_of::<Factory>() + 32 * 5
    )]
    pub factory_config: Box<Account<'info, Factory>>,
    /// CHECK
    #[account(mut)]
    pub fee_collector_info: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub manager: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, creator_fee: u64, service_fee: u16) -> Result<()> {
    let factory = &mut ctx.accounts.factory_config;

    require!(creator_fee <= MAX_CREATOR_FEE, FactoryError::Invalid);
    require!(service_fee <= MAX_SERVICE_FEE, FactoryError::Invalid);

    factory.admin = ctx.accounts.admin.key();
    factory.creator_fee = creator_fee;
    factory.service_fee = service_fee;
    factory.fee_collector = ctx.accounts.fee_collector_info.key();
    factory.manager = ctx.accounts.manager.key();

    emit!(FactoryInit { init: true });

    Ok(())
}
