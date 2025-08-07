use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct ManagerUpdated {
    pub manager: Pubkey,
}

#[derive(Accounts)]
pub struct SetManager<'info> {
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
    pub manager_info: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn set_manager(ctx: Context<SetManager>) -> Result<()> {
    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;
    require!(
        factory.admin == *ctx.accounts.admin.key,
        FactoryError::Unauthorized
    );

    factory.manager = ctx.accounts.manager_info.key();

    emit!(ManagerUpdated {
        manager: ctx.accounts.manager_info.key()
    });

    Ok(())
}
