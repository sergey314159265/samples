use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount};
use crate::errors::ErrorCode;
use crate::StakeEntry;
use crate::StakePool;
use crate::STAKE_POOL_PREFIX;

#[derive(Accounts)]
pub struct UnstakeTokenCtx<'info> {
    #[account(mut, constraint = stake_entry.stake_pool == stake_pool.key() @ErrorCode::InvalidStakePool)]
    stake_pool: Box<Account<'info, StakePool>>,
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = stake_pool,
        associated_token::token_program = token_program,
    )]
    pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = stake_entry.staker == user.key() @ErrorCode::InvalidStakeEntryOwner)]
    stake_entry: Box<Account<'info, StakeEntry>>,

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

pub fn handler(ctx: Context<UnstakeTokenCtx>) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_entry = &mut ctx.accounts.stake_entry;
    let seeds = &[
        STAKE_POOL_PREFIX.as_bytes(),
        stake_pool.identifier.as_ref(),
        &[stake_pool.bump],
    ];
    let signer = [&seeds[..]];
    // FEATURE: Minimum stake seconds
    if stake_pool.min_stake_seconds.is_some()
        && stake_pool.min_stake_seconds.unwrap() > 0
        && ((Clock::get().unwrap().unix_timestamp - stake_entry.last_staked_at) as u32)
            < stake_pool.min_stake_seconds.unwrap()
    {
        return Err(error!(ErrorCode::MinStakeSecondsNotSatisfied));
    }
    
    let reward_amount: u64 = get_rewards(
        u128::from(stake_entry.amount), 
        u128::from(stake_pool.stake_reward), 
        u128::from(stake_entry.last_staked_at as u64)
    )?;

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
        stake_entry.amount + reward_amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    stake_entry.staker = Pubkey::default();
    stake_pool.total_staked = stake_pool
        .total_staked
        .checked_sub(stake_entry.amount)
        .expect("Sub error");
    stake_pool.total_stakers = stake_pool.total_stakers.checked_sub(1).expect("Sub error");
    stake_entry.amount = 0;

    Ok(())
}

pub fn get_rewards(
    amount: u128,
    stake_reward: u128,
    last_staked_at: u128
) -> Result<u64> {

    let seconds_in_year = 31536000 as u128;
    let total_seconds_for_rewards = (Clock::get().unwrap().unix_timestamp as u128).checked_sub(last_staked_at).ok_or(ArithmeticOverflow)?;
    let yearly_reward = amount.checked_mul(stake_reward).and_then(|f| f.checked_div(10000)).ok_or(ArithmeticOverflow)?;

    let reward = yearly_reward.checked_mul(total_seconds_for_rewards).and_then(|f| f.checked_div(seconds_in_year)).ok_or(ArithmeticOverflow)?;

    Ok(
        u64::try_from(reward).unwrap()
    ) 
}  