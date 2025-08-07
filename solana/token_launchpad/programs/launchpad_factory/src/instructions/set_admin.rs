use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct AdminUpdated {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_CONFIG],
        bump
    )]
    pub factory_config: Box<Account<'info, Factory>>,

    /// CHECK
    #[account(mut)]
    pub new_admin: AccountInfo<'info>,

    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

pub fn set_admin(ctx: Context<SetAdmin>) -> Result<()> {
    require!(
        ctx.accounts.admin.key() == ctx.accounts.factory_config.admin
            || ctx.accounts.program_data.upgrade_authority_address
                == Some(ctx.accounts.admin.key()),
        FactoryError::Unauthorized
    );

    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;

    factory.admin = ctx.accounts.new_admin.key();

    emit!(AdminUpdated {
        admin: ctx.accounts.new_admin.key()
    });

    Ok(())
}
