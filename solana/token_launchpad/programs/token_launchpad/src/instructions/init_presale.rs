use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint,
    TokenInterface,
};

use crate::{
    constants::{
        FACTORY_PROGRAM_ID,
        PRESALE_SEED,
        PRESALE_VERSION,
    },
    error::PresaleError,
    state::presale::{
        LiquidityType,
        PresaleParams,
        PresaleState,
        PresaleType,
    },
};

#[derive(Accounts)]
#[instruction(presale_config: PresaleParams)]
pub struct InitializePresale<'info> {
    #[account(
        init_if_needed,
        constraint = !presale.is_init @PresaleError::Invalid,
        payer = owner,
        seeds = [PRESALE_SEED, token.key().as_ref(), presale_config.identifier.as_ref()],
        bump,
        space = 8 + std::mem::size_of::<PresaleState>()
    )]
    pub presale: Box<Account<'info, PresaleState>>,

    pub token: InterfaceAccount<'info, Mint>,

    /// CHECK
    #[account(mut)]
    pub fee_collector: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK
    #[account(signer, owner = FACTORY_PROGRAM_ID)]
    pub factory_pda: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn init_presale(ctx: Context<InitializePresale>, presale_config: PresaleParams) -> Result<()> {
    require!(
        ctx.accounts.factory_pda.owner.key() == FACTORY_PROGRAM_ID,
        PresaleError::Unauthorized
    );
    require!(
        ctx.accounts.factory_pda.is_signer,
        PresaleError::Unauthorized
    );

    configure_presale(
        &mut ctx.accounts.presale,
        presale_config,
        ctx.accounts.fee_collector.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token.to_account_info(),
    )?;

    Ok(())
}

fn configure_presale<'info>(
    presale: &mut Account<'info, PresaleState>,
    params: PresaleParams,
    fee_collector: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    mint: AccountInfo<'info>,
) -> Result<()> {
    presale.version = PRESALE_VERSION;
    presale.token_price = params.token_price;
    presale.hard_cap = params.hard_cap;
    presale.soft_cap = params.soft_cap;
    presale.min_contribution = params.min_contribution;
    presale.max_contribution = params.max_contribution;
    presale.start_time = params.start_time;
    presale.end_time = params.end_time;
    presale.listing_rate = params.listing_rate;
    presale.liquidity_bp = params.liquidity_bp;
    presale.service_fee = params.service_fee;
    presale.refund_type = params.refund_type;
    presale.listing_opt = params.listing_opt;
    presale.liquidity_type = params.liquidity_type;
    presale.listing_platform = params.listing_platform;
    presale.total_raised = 0;
    presale.presale_ended = false;
    presale.presale_canceled = false;
    presale.presale_refund = false;
    presale.is_init = true;
    presale.fee_collector = fee_collector.key();
    presale.owner = owner.key();
    presale.token = mint.key();
    presale.identifier = params.identifier;
    presale.affiliate_enabled = params.affiliate_enabled;
    presale.whitelist_enabled = params.whitelist_enabled;
    presale.commission_rate = params.commission_rate;
    presale.total_ref_amount = 0;
    presale.total_ref_count = 0;
    presale.total_tokens_sold = 0;
    presale.presale_type = params.presale_type;
    presale.tokens_claimed_by_owner = 0;
    presale.owner_reward_withdrawn = false;
    presale.sol_pool_reserve = 0;
    presale.token_pool_reserve = 0;
    presale.launchpad_type = params.launchpad_type;
    presale.manager = params.manager;
    presale.admin = params.admin;

    if presale.liquidity_type == LiquidityType::Lock {
        presale.liquidity_lock_time = params.liquidity_lock_time;
    } else {
        presale.liquidity_lock_time = 0;
    }

    if presale.presale_type == PresaleType::FairLaunch {
        presale.total_tokens_sold = params.tokens_allocated;
    }

    Ok(())
}
