use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::spl_token_2022,
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};
use solana_program::{
    native_token::sol_to_lamports,
    pubkey::Pubkey,
};
use token_launchpad::{
    cpi::{
        accounts::{
            InitializePresale,
            InitializeVaults,
        },
        initialize_presale,
        initialize_vaults,
    },
    state::presale::{
        LaunchpadType,
        LiquidityType,
        ListingOpt,
        ListingPlatform,
        PresaleParams,
        PresaleType,
        RefundType,
    },
};

use crate::{
    constants::*,
    error::FactoryError,
    state::*,
    utils::{
        calculate_presale_data,
        calculate_presale_data_degen,
        get_transfer_inverse_fee,
        is_supported_mint,
        transfer_sols,
        transfer_tokens,
    },
};

#[event]
pub struct LaunchpadCreated {
    pub launchpad: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct CreatePresale<'info> {
    /// CHECK
    #[account(mut)]
    pub presale: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub token_vault_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [FACTORY_CONFIG],
        bump,
    )]
    pub factory_config: Box<Account<'info, Factory>>,

    /// CHECK
    #[account(mut)]
    pub fee_collector: AccountInfo<'info>,

    /// CHECK
    #[account(mut, address = factory_config.manager.key())]
    pub manager: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
    // This is the reference to the TokenPresale program where the presale is managed
    /// CHECK:
    pub presale_program: UncheckedAccount<'info>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn create_presale(
    ctx: Context<CreatePresale>,
    presale_type: PresaleType,
    tokens_allocated: u64,
    token_price: u64,
    hard_cap: u64,
    soft_cap: u64,
    min_contribution: u64,
    max_contribution: u64,
    start_time: i64,
    end_time: i64,
    listing_rate: u64,
    liquidity_lock_time: i64,
    liquidity_bp: u16,
    refund_type: RefundType,
    listing_opt: ListingOpt,
    liquidity_type: LiquidityType,
    listing_platform: ListingPlatform,
    identifier: String,
    affiliate_enabled: bool,
    whitelist_enabled: bool,
    comm_rate: u16,
    launchpad_type: LaunchpadType,
) -> Result<()> {
    let presale_account = &mut ctx.accounts.presale;
    let token_mint = &ctx.accounts.token_mint;
    let owner = &ctx.accounts.owner;
    let factory = &mut ctx.accounts.factory_config;
    let fee_collector = &mut ctx.accounts.fee_collector;
    let manager = factory.manager;
    let admin = factory.admin;

    require!(
        factory.fee_collector != Pubkey::default(),
        FactoryError::NoFeeWallet
    );
    require!(
        factory.fee_collector == *fee_collector.key,
        FactoryError::InvalidFeeAccount
    );

    require!(is_supported_mint(token_mint)?, FactoryError::InvalidMint);

    require!(
        listing_platform == ListingPlatform::Raydium
            || token_mint.to_account_info().owner != &spl_token_2022::ID,
        FactoryError::InvalidMint
    );

    let now = Clock::get()?.unix_timestamp;

    if presale_type == PresaleType::HardCapped {
        require!(soft_cap < hard_cap, FactoryError::Invalid);
    }

    require!(
        min_contribution < max_contribution
            || (presale_type == PresaleType::FairLaunch && max_contribution == 0),
        FactoryError::Invalid
    );
    require!(start_time < end_time, FactoryError::Invalid);
    require!(end_time > now, FactoryError::Invalid);
    require!(start_time > now, FactoryError::Invalid);

    if (hard_cap < sol_to_lamports(DEGEN_MIN_HARD_CAP.into())
        || hard_cap > sol_to_lamports(DEGEN_MAX_HARD_CAP.into()))
        && launchpad_type == LaunchpadType::Degen
    {
        msg!(
            "Hardcap limits: min = {}, max = {}",
            DEGEN_MIN_HARD_CAP,
            DEGEN_MAX_HARD_CAP
        );
        return err!(FactoryError::InvalidHardcap);
    };

    transfer_sols(
        &ctx.accounts.owner.to_account_info(),
        fee_collector,
        &ctx.accounts.system_program.to_account_info(),
        factory.creator_fee,
    )?;

    if launchpad_type == LaunchpadType::Degen {
        transfer_sols(
            &ctx.accounts.owner.to_account_info(),
            &ctx.accounts.manager.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            sol_to_lamports(DEGEN_AUTOFINALIZATION_FEE_SOL),
        )?;
    };

    let bump = ctx.bumps.factory_config;
    let signer: &[&[&[u8]]] = &[&[FACTORY_CONFIG, &[bump]]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.presale_program.to_account_info(),
        InitializePresale {
            owner: owner.to_account_info(),
            presale: presale_account.to_account_info(),
            token: token_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            fee_collector: fee_collector.clone(),
            factory_pda: factory.to_account_info(),
        },
        signer,
    );

    let tokens_allocated_in_lamports = 10u64
        .checked_pow(token_mint.decimals as u32)
        .and_then(|f| f.checked_mul(tokens_allocated))
        .ok_or(FactoryError::ArithmeticOverflow)?;

    let presale_config = PresaleParams {
        token_price,
        hard_cap,
        soft_cap,
        min_contribution,
        max_contribution,
        start_time,
        end_time,
        listing_rate,
        liquidity_lock_time,
        liquidity_bp,
        service_fee: factory.service_fee,
        refund_type,
        listing_opt,
        liquidity_type,
        listing_platform,
        identifier,
        affiliate_enabled,
        commission_rate: comm_rate,
        whitelist_enabled,
        presale_type: presale_type.clone(),
        tokens_allocated: tokens_allocated_in_lamports,
        launchpad_type: launchpad_type.clone(),
        manager,
        admin,
    };

    // Call the `initialize_presale` function from the TokenPresale program
    initialize_presale(cpi_ctx, presale_config)?;

    let cpi_ctx_vault = CpiContext::new_with_signer(
        ctx.accounts.presale_program.to_account_info(),
        InitializeVaults {
            presale: presale_account.to_account_info(),
            vault: ctx.accounts.vault.to_account_info(),
            token_vault_account: ctx.accounts.token_vault_account.to_account_info(),
            owner: owner.to_account_info(),
            token: token_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            factory_pda: factory.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        },
        signer,
    );

    initialize_vaults(cpi_ctx_vault)?;

    if presale_type == PresaleType::HardCapped {
        let decimals_result = 10u64
            .checked_pow(token_mint.decimals as u32)
            .ok_or(FactoryError::ArithmeticOverflow)?;

        let (_, _, tokens_for_liquidity, presale_tokens) = match launchpad_type {
            LaunchpadType::Pro => calculate_presale_data(
                u128::from(hard_cap),
                u128::from(factory.service_fee),
                u128::from(liquidity_bp),
                u128::from(decimals_result),
                u128::from(token_price),
                u128::from(listing_rate),
            )?,
            LaunchpadType::Degen => calculate_presale_data_degen(
                u128::from(hard_cap),
                u128::from(factory.service_fee),
                u128::from(liquidity_bp),
                u128::from(decimals_result),
                u128::from(token_price),
                u128::from(listing_rate),
            )?,
        };

        let transfer_presale_tokens_amount = {
            let transfer_fee_raydium = get_transfer_inverse_fee(
                &ctx.accounts.token_mint.to_account_info(),
                tokens_for_liquidity,
            )?;
            let tokens_to_transfer = presale_tokens
                .checked_add(transfer_fee_raydium)
                .ok_or(FactoryError::ArithmeticOverflow)?;
            let transfer_fee = get_transfer_inverse_fee(
                &ctx.accounts.token_mint.to_account_info(),
                tokens_to_transfer,
            )?;

            tokens_to_transfer
                .checked_add(transfer_fee)
                .ok_or(FactoryError::ArithmeticOverflow)?
        };

        transfer_tokens(
            ctx.accounts.owner_token_account.to_account_info(),
            ctx.accounts.token_vault_account.to_account_info(),
            ctx.accounts.token_mint.clone(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            transfer_presale_tokens_amount,
        )?;
    } else if presale_type == PresaleType::FairLaunch {
        let transfer_presale_tokens_amount = {
            let net_rate_bp = 10000u16
                .checked_sub(factory.service_fee)
                .ok_or(FactoryError::ArithmeticOverflow)?;
            let net_tokens_allocated = (tokens_allocated_in_lamports as u128)
                .checked_mul(net_rate_bp as u128)
                .and_then(|f| f.checked_div(10000))
                .ok_or(FactoryError::ArithmeticOverflow)?;
            let tokens_for_liquidity = net_tokens_allocated
                .checked_mul(liquidity_bp as u128)
                .and_then(|f| f.checked_div(10000))
                .and_then(|f| u64::try_from(f).ok())
                .ok_or(FactoryError::ArithmeticOverflow)?;
            let transfer_fee_raydium = get_transfer_inverse_fee(
                &ctx.accounts.token_mint.to_account_info(),
                tokens_for_liquidity,
            )?;
            let tokens_to_transfer = tokens_allocated_in_lamports
                .checked_add(tokens_for_liquidity)
                .and_then(|f| f.checked_add(transfer_fee_raydium))
                .ok_or(FactoryError::ArithmeticOverflow)?;
            let transfer_fee = get_transfer_inverse_fee(
                &ctx.accounts.token_mint.to_account_info(),
                tokens_to_transfer,
            )?;

            tokens_to_transfer
                .checked_add(transfer_fee)
                .ok_or(FactoryError::ArithmeticOverflow)?
        };

        transfer_tokens(
            ctx.accounts.owner_token_account.to_account_info(),
            ctx.accounts.token_vault_account.to_account_info(),
            ctx.accounts.token_mint.clone(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            transfer_presale_tokens_amount,
        )?;
    }

    let now = Clock::get()?.unix_timestamp;

    emit!(LaunchpadCreated {
        launchpad: presale_account.key(),
        owner: owner.key(),
        timestamp: now
    });

    Ok(())
}
