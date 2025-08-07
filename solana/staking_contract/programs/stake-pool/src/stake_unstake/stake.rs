use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::{
    token::Token, 
    token_2022::{
        Token2022,
        spl_token_2022::{
            self,
            extension::{
                transfer_fee::{TransferFeeConfig, MAX_FEE_BASIS_POINTS},
                StateWithExtensions,
            }
        }
    },
    token_interface::{self, TokenAccount, Mint, spl_token_2022::extension::BaseStateWithExtensions},

};
use crate::errors::ErrorCode;
use crate::stake_entry::StakeEntry;
use crate::stake_pool::StakePool;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct StakeTokenCtx<'info> {
    #[account(mut, constraint = stake_pool.key() == stake_entry.stake_pool @ErrorCode::InvalidStakePool)]
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
        user_token_account.amount > 0
        && user_token_account.mint == stake_mint.key()
        && user_token_account.owner == user.key()
        @ ErrorCode::InvalidUserStakeMintTokenAccount
    )]
    user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    token_program: Program<'info, Token2022>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeTokenCtx>, amount: u64) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_entry = &mut ctx.accounts.stake_entry;

    if stake_entry.amount > 0 {
        return err!(ErrorCode::TokensAlreadyStaked);
    }

    let transfer_amount = {
        let transfer_fee = get_transfer_inverse_fee(&ctx.accounts.stake_mint.to_account_info(), amount)?;

        amount.checked_add(transfer_fee).unwrap()
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        transfer_amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    stake_entry.staker = ctx.accounts.user.key();
    stake_entry.stake_pool = stake_pool.key();
    stake_entry.last_staked_at = Clock::get().unwrap().unix_timestamp;
    stake_entry.amount = stake_entry
        .amount
        .checked_add(amount)
        .unwrap();
    stake_pool.total_stakers = stake_pool.total_stakers.checked_add(1).unwrap();
    stake_pool.total_staked = stake_pool
        .total_staked
        .checked_add(amount)
        .expect("Add error");

    Ok(())
}

pub fn get_transfer_inverse_fee(mint_info: &AccountInfo, post_fee_amount: u64) -> Result<u64> {
    if *mint_info.owner == Token::id() {
        return Ok(0);
    }
    if post_fee_amount == 0 {
        return err!(ErrorCode::InvalidInput);
    }
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        let epoch = Clock::get()?.epoch;

        let transfer_fee = transfer_fee_config.get_epoch_fee(epoch);
        if u16::from(transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
            u64::from(transfer_fee.maximum_fee)
        } else {
            transfer_fee_config
                .calculate_inverse_epoch_fee(epoch, post_fee_amount)
                .unwrap()
        }
    } else {
        0
    };
    Ok(fee)
}