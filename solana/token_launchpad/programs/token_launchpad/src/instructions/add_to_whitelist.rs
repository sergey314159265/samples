use anchor_lang::prelude::*;

use crate::state::{presale::PresaleState, whitelist::WhitelistEntry};

#[derive(Accounts)]
#[instruction(_user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(
        has_one = owner
    )]
    pub presale: Box<Account<'info, PresaleState>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [b"whitelist", presale.key().as_ref(), _user.key().as_ref()],
        bump,
        space = 8
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub system_program: Program<'info, System>,
}

pub fn add_to_whitelist(_ctx: Context<AddToWhitelist>, _user: Pubkey) -> Result<()> {
    Ok(())
}
