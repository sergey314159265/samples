use anchor_lang::prelude::*;
use crate::StakeEntry;
use crate::StakePool;
use crate::STAKE_ENTRY_PREFIX;
use crate::STAKE_ENTRY_SIZE;

#[derive(Accounts)]
#[instruction(identifier: String)]
pub struct InitEntryCtx<'info> {
    #[account(
        init,
        payer = payer,
        space = STAKE_ENTRY_SIZE,
        seeds = [STAKE_ENTRY_PREFIX.as_bytes(), identifier.as_ref(),stake_pool.key().as_ref(), payer.key().as_ref()],
        bump,
    )]
    stake_entry: Box<Account<'info, StakeEntry>>,
    #[account(mut)]
    stake_pool: Box<Account<'info, StakePool>>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitEntryCtx>, identifier: String) -> Result<()> {
    let stake_entry = &mut ctx.accounts.stake_entry;
    stake_entry.bump = ctx.bumps.stake_entry;
    stake_entry.stake_pool = ctx.accounts.stake_pool.key();
    stake_entry.staker = ctx.accounts.payer.key();
    stake_entry.amount = 0;
    stake_entry.identifier = identifier;

    Ok(())
}
