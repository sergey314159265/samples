use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
};

#[event]
pub struct CreatorFeeUpdated {
    pub creator_fee: u64,
}

#[derive(Accounts)]
pub struct SetCreatorFee<'info> {
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

pub fn set_creator_fee(ctx: Context<SetCreatorFee>, creator_fee: u64) -> Result<()> {
    let factory: &mut Box<Account<'_, Factory>> = &mut ctx.accounts.factory_config;
    require!(
        factory.admin == *ctx.accounts.admin.key,
        FactoryError::Unauthorized
    );
    require!(creator_fee <= MAX_CREATOR_FEE, FactoryError::Invalid);
    factory.creator_fee = creator_fee;

    emit!(CreatorFeeUpdated { creator_fee });

    Ok(())
}
