use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint,
    TokenInterface,
};

use crate::{
    constants::{
        CONTRIBUTE_SEED,
        REFERRER_SEED,
        VAULT_SEED,
    },
    error::PresaleError,
    instructions::TokensPurchased,
    state::{
        affiliate::AffiliateReferrerState,
        contribution::ContributionState,
        presale::{
            LaunchpadType,
            PresaleState,
            PresaleType,
        },
        vault::Vault,
    },
    utils::{
        check_if_user_is_whitelisted,
        record_contribution,
        transfer_sols,
    },
};

#[derive(Accounts)]
pub struct ContributeAffiliate<'info> {
    /// CHECK: Presale account (PDA)
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, presale.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [CONTRIBUTE_SEED, presale.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<ContributionState>()
    )]
    pub contribution: Box<Account<'info, ContributionState>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [REFERRER_SEED, presale.key().as_ref(), referrer.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<AffiliateReferrerState>()
    )]
    pub affiliate_referrer_state: Box<Account<'info, AffiliateReferrerState>>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK:
    pub whitelist_entry: AccountInfo<'info>,

    /// CHECK
    pub referrer: AccountInfo<'info>,
    pub token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn contribute_affiliate(ctx: Context<ContributeAffiliate>, amount: u64) -> Result<()> {
    let presale = &mut PresaleState::deserialize_data(
        &ctx.accounts.presale,
        &ctx.accounts.user,
        &ctx.accounts.system_program,
    )?;
    let contribution = &mut ctx.accounts.contribution;
    let referrer_state = &mut ctx.accounts.affiliate_referrer_state;
    let referrer = &mut ctx.accounts.referrer;
    let current_time = Clock::get().unwrap().unix_timestamp;

    if presale.whitelist_enabled {
        check_if_user_is_whitelisted(
            &ctx.accounts.whitelist_entry,
            &ctx.accounts.user.key(),
            &ctx.accounts.presale.key(),
            ctx.program_id,
        )?;
    }

    require!(
        presale.token == ctx.accounts.token.key(),
        PresaleError::Invalid
    );

    require!(
        current_time >= presale.start_time && current_time <= presale.end_time,
        PresaleError::PresaleNotActive
    );
    require!(!presale.presale_ended, PresaleError::PresaleEnded);
    require!(!presale.presale_canceled, PresaleError::PresaleCanceled);

    contribution.contributor = ctx.accounts.user.key();

    if presale.presale_type == PresaleType::HardCapped {
        require!(
            amount >= presale.min_contribution
                && (contribution.amount + amount) <= presale.max_contribution,
            PresaleError::ContributionNotWithinLimits
        );

        let current_cap = presale.hard_cap - presale.total_raised;

        let adjusted_amount = if amount > current_cap {
            current_cap
        } else {
            amount
        };

        require!(
            adjusted_amount > 0,
            PresaleError::ContributionNotWithinLimits
        );

        transfer_sols(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            adjusted_amount,
        )?;

        let decimals_result = 10u64
            .checked_pow(ctx.accounts.token.decimals as u32)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let tokens_scaled = (adjusted_amount as u128)
            .checked_mul(decimals_result as u128)
            .and_then(|f| match presale.launchpad_type == LaunchpadType::Degen {
                true => f.checked_mul(100000000),
                false => Some(f),
            })
            .and_then(|f| f.checked_div(presale.token_price as u128))
            .ok_or(PresaleError::ArithmeticOverflow)?;

        let tokens = u64::try_from(tokens_scaled).unwrap();

        presale.total_raised = presale
            .total_raised
            .checked_add(adjusted_amount)
            .ok_or(PresaleError::ArithmeticOverflow)?;
        presale.total_tokens_sold = presale
            .total_tokens_sold
            .checked_add(tokens)
            .ok_or(PresaleError::ArithmeticOverflow)?;
        contribution.amount = contribution
            .amount
            .checked_add(adjusted_amount)
            .ok_or(PresaleError::ArithmeticOverflow)?;
        contribution.tokens_purchased = contribution
            .tokens_purchased
            .checked_add(tokens)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        if presale.affiliate_enabled {
            // record contribution
            record_contribution(referrer_state, presale, adjusted_amount)?;

            if referrer_state.referrer == Pubkey::default() {
                referrer_state.referrer = referrer.to_account_info().key();
            }
        }

        emit!(TokensPurchased {
            purchaser: ctx.accounts.user.key(),
            presale: ctx.accounts.presale.key(),
            amount: adjusted_amount,
            timestamp: current_time
        });
    } else if presale.presale_type == PresaleType::FairLaunch {
        require!(
            amount >= presale.min_contribution
                && ((contribution.amount + amount) <= presale.max_contribution
                    || presale.max_contribution == 0),
            PresaleError::ContributionNotWithinLimits
        );

        transfer_sols(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            amount,
        )?;

        presale.total_raised = presale
            .total_raised
            .checked_add(amount)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        contribution.amount = contribution
            .amount
            .checked_add(amount)
            .ok_or(PresaleError::ArithmeticOverflow)?;

        if presale.affiliate_enabled {
            // record contribution
            record_contribution(referrer_state, presale, amount)?;

            if referrer_state.referrer == Pubkey::default() {
                referrer_state.referrer = referrer.to_account_info().key();
            }
        }

        emit!(TokensPurchased {
            purchaser: ctx.accounts.user.key(),
            presale: ctx.accounts.presale.key(),
            amount,
            timestamp: current_time
        });
    }

    presale.serialize_data(&ctx.accounts.presale)?;

    Ok(())
}
