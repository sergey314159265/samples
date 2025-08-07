use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount};
use crate::errors::ErrorCode;
use crate::StakePool;
use crate::STAKE_POOL_PREFIX;

#[derive(Accounts)]
pub struct WithdrawTokensCtx<'info> {
    #[account(mut, constraint = stake_pool.authority == user.key() @ErrorCode::InvalidAdmin,)]
    stake_pool: Box<Account<'info, StakePool>>,
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = stake_pool,
        associated_token::token_program = token_program,
    )]
    pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(constraint = stake_pool.mint == stake_mint.key() @ ErrorCode::InvalidStakeMint)]
    stake_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    user: Signer<'info>,
    #[account(mut, constraint =
        user_token_account.mint == stake_mint.key()
        && user_token_account.owner == user.key()
        @ ErrorCode::InvalidUserStakeMintTokenAccount
    )]
    user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    token_program: Program<'info, Token2022>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawTokensCtx>, amount: u64) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let seeds = &[
        STAKE_POOL_PREFIX.as_bytes(),
        stake_pool.identifier.as_ref(),
        &[stake_pool.bump],
    ];
    let signer = [&seeds[..]];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.pool_token_account.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: stake_pool.to_account_info(),
            },
            &signer,
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    Ok(())
}