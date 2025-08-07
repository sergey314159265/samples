use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint,
        TokenInterface,
    },
};

use crate::{
    constants::VAULT_SEED,
    error::PresaleError,
    state::{
        presale::PresaleState,
        vault::Vault,
    },
    utils::{
        tranfer_sol_from_vault,
        validate_presale_pda,
    },
};

#[event]
pub struct OwnerRewardWithdrawn {
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct WithdrawOwnerReward<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_owner_reward(ctx: Context<WithdrawOwnerReward>) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.owner,
        &ctx.accounts.system_program,
    )?;
    let now = Clock::get().unwrap().unix_timestamp;

    let _ = validate_presale_pda(
        presale,
        ctx.accounts.presale.key(),
        ctx.accounts.token_mint.key(),
    )?;

    require!(
        presale.token == ctx.accounts.token_mint.key(),
        PresaleError::InvalidTokenMint
    );

    require!(
        presale.owner == *ctx.accounts.owner.key,
        PresaleError::Unauthorized
    );

    require!(
        now > presale.end_time || presale.total_raised >= presale.hard_cap,
        PresaleError::PresaleNotEnded
    );

    require!(
        !presale.presale_canceled && !presale.presale_refund,
        PresaleError::PresaleFinalizationPreconditionsNotMet
    );

    require!(
        !presale.owner_reward_withdrawn,
        PresaleError::OwnerRewardWithdrawn
    );

    let service_fee_reserve = (presale.total_raised as u128)
        .checked_mul(presale.service_fee as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let total_raised_net = presale
        .total_raised
        .checked_sub(service_fee_reserve)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let affiliate_reward_pool_reserve = if presale.affiliate_enabled {
        (total_raised_net as u128)
            .checked_mul(presale.commission_rate as u128)
            .and_then(|f| f.checked_div(10000))
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(PresaleError::ArithmeticOverflow)?
    } else {
        0u64
    };

    let liquidity_pool_reserve = (total_raised_net as u128)
        .checked_mul(presale.liquidity_bp as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let presale_owner_reward: u64 = total_raised_net
        .checked_sub(affiliate_reward_pool_reserve)
        .and_then(|f| f.checked_sub(liquidity_pool_reserve))
        .and_then(|f| f.checked_sub(presale.tokens_claimed_by_owner))
        .ok_or(PresaleError::ArithmeticOverflow)?;

    presale.tokens_claimed_by_owner = presale
        .tokens_claimed_by_owner
        .checked_add(presale_owner_reward)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    tranfer_sol_from_vault(
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        presale_owner_reward,
    )?;

    presale.owner_reward_withdrawn = true;

    emit!(OwnerRewardWithdrawn {
        amount: presale_owner_reward,
        timestamp: now
    });

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
