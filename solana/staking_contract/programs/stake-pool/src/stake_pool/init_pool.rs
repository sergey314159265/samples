use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};
use crate::errors::ErrorCode;
use crate::StakePool;
use crate::STAKE_POOL_DEFAULT_SIZE;
use crate::STAKE_POOL_PREFIX;
use solana_program::{pubkey, pubkey::Pubkey};

const ADMIN: Pubkey = pubkey!("4bRYs66kGxujekaRGHJjvjP4g7SCou28FZJ8LPDsyDnR");
#[derive(Accounts)]
#[instruction(ix: InitPoolIx)]
pub struct InitPoolCtx<'info> {
    #[account(
        init,
        payer = payer,
        space = STAKE_POOL_DEFAULT_SIZE,
        seeds = [STAKE_POOL_PREFIX.as_bytes(), ix.identifier.as_ref()],
        bump
    )]
    stake_pool: Account<'info, StakePool>,
    #[account(
        init_if_needed,
        payer=payer,
        associated_token::mint = mint,
        associated_token::authority = stake_pool,
        associated_token::token_program = token_program,
    )]
    pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mint::token_program = token_program,
    )]
    mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    payer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = payer.key() == ADMIN @ErrorCode::InvalidAdmin)]
    payer: Signer<'info>,
    token_program: Program<'info, Token2022>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitPoolCtx>, ix: InitPoolIx) -> Result<()> {
    let bump = ctx.bumps.stake_pool;
    let new_stake_pool = StakePool {
        bump,
        authority: ctx.accounts.payer.key(),
        total_staked: 0,
        total_stakers: 0,
        min_stake_seconds: Some(ix.min_stake_seconds),
        stake_reward: ix.stake_reward,
        mint: ctx.accounts.mint.key(),
        identifier: ix.identifier,
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.payer_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        ix.deposite_amount,
        ctx.accounts.mint.decimals,
    )?;

    let stake_pool = &mut ctx.accounts.stake_pool;

    stake_pool.set_inner(new_stake_pool);
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitPoolIx {
    pub min_stake_seconds: u32,
    pub stake_reward: u64,
    pub deposite_amount: u64,
    pub identifier: String,
}
