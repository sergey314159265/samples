use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke,
        system_instruction,
    },
};
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{
            transfer_fee::{
                TransferFeeConfig,
                MAX_FEE_BASIS_POINTS,
            },
            BaseStateWithExtensions,
            StateWithExtensions,
        },
    },
    token_interface::{
        transfer_checked,
        Mint,
        TransferChecked,
    },
};

use crate::{
    constants::{
        ADMIN_FINALIZATION_TIMEOUT,
        PRESALE_SEED,
    },
    error::PresaleError,
    instructions::CommissionWithdrawn,
    program::TokenLaunchpad,
    state::{
        affiliate::AffiliateReferrerState,
        presale::{
            LaunchpadType,
            PresaleState,
        },
    },
};

pub fn transfer_sols<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let transfer_ix = system_instruction::transfer(&from.key(), &to.key(), amount);

    invoke(
        &transfer_ix,
        &[
            from.to_account_info(),
            to.to_account_info(),
            system_program.to_account_info(),
        ],
    )?;

    Ok(())
}

pub fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    mint: Account<'info, Mint>,
    amount: u64,
) -> Result<()> {
    // Set up the CPI (Cross-Program Invocation) context
    let cpi_accounts = TransferChecked {
        from: from.clone(),
        to: to.clone(),
        authority: from.clone(),
        mint: mint.to_account_info().clone(),
    };
    let cpi_program = token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Invoke the transfer based on the token program
    transfer_checked(cpi_ctx, amount, mint.decimals)?;
    Ok(())
}

pub fn tranfer_sol_from_vault<'info>(
    vault: AccountInfo<'info>,
    user: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let vault_lamports = **vault.lamports.borrow();

    require!(vault_lamports > amount, PresaleError::InsufficientFunds);

    **vault.try_borrow_mut_lamports()? -= amount;
    **user.try_borrow_mut_lamports()? += amount;

    Ok(())
}

pub fn record_contribution<'info>(
    referrer_state: &mut Account<'info, AffiliateReferrerState>,
    presale: &mut PresaleState,
    amount: u64,
) -> Result<()> {
    if referrer_state.total_sale == 0 {
        presale.total_ref_count += 1;
    }

    referrer_state.total_sale = referrer_state
        .total_sale
        .checked_add(amount)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    presale.total_ref_amount = presale
        .total_ref_amount
        .checked_add(amount)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    Ok(())
}

pub fn withdraw_commission<'info>(
    presale: &mut PresaleState,
    referrer_state: &mut Account<'info, AffiliateReferrerState>,
    vault: AccountInfo<'info>,
    withdrawer: AccountInfo<'info>,
) -> Result<()> {
    require!(referrer_state.total_sale > 0, PresaleError::Invalid);
    require!(!referrer_state.is_reward_claimed, PresaleError::Invalid);

    let service_fee_reserve = (presale.total_raised as u128)
        .checked_mul(presale.service_fee as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let total_raised_net = presale
        .total_raised
        .checked_sub(service_fee_reserve)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let current_reward = (total_raised_net as u128)
        .checked_mul(presale.commission_rate as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let total_reward = current_reward;

    let service_fee_reward_reserve = (referrer_state.total_sale as u128)
        .checked_mul(presale.service_fee as u128)
        .and_then(|f| f.checked_div(10000))
        .and_then(|f| u64::try_from(f).ok())
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let user_total_sale_net = referrer_state
        .total_sale
        .checked_sub(service_fee_reward_reserve)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let user_part_perce = user_total_sale_net
        .checked_mul(10000)
        .and_then(|f| f.checked_div(total_raised_net))
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let transfer_amount = total_reward
        .checked_mul(user_part_perce)
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;

    referrer_state.is_reward_claimed = true;

    tranfer_sol_from_vault(vault, withdrawer, transfer_amount)?;

    emit!(CommissionWithdrawn {
        amount: transfer_amount,
        referrer: referrer_state.referrer
    });

    Ok(())
}

pub fn calculate_presale_data(
    hard_cap: u128,
    service_fee: u128,
    liquidity_bp: u128,
    decimals_result: u128,
    token_price: u128,
    listing_rate: u128,
) -> Result<(u64, u64, u64, u64)> {
    let s_fee_amount = hard_cap
        .checked_mul(service_fee)
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let net_hardcap = hard_cap
        .checked_sub(s_fee_amount)
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let liquidity_sols = net_hardcap
        .checked_mul(liquidity_bp)
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let tokens_for_presale = hard_cap
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_div(token_price))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let tokens_for_liquidity = liquidity_sols
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_div(listing_rate))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let presale_tokens = tokens_for_presale
        .checked_add(tokens_for_liquidity)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    Ok((
        u64::try_from(liquidity_sols).unwrap(),
        u64::try_from(tokens_for_presale).unwrap(),
        u64::try_from(tokens_for_liquidity).unwrap(),
        u64::try_from(presale_tokens).unwrap(),
    ))
}

pub fn calculate_presale_data_degen(
    hard_cap: u128,
    service_fee: u128,
    liquidity_bp: u128,
    decimals_result: u128,
    token_price: u128,
    listing_rate: u128,
) -> Result<(u64, u64, u64, u64)> {
    let s_fee_amount = hard_cap
        .checked_mul(service_fee)
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let net_hardcap = hard_cap
        .checked_sub(s_fee_amount)
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let liquidity_sols = net_hardcap
        .checked_mul(liquidity_bp)
        .and_then(|f| f.checked_div(10000))
        .ok_or(PresaleError::ArithmeticOverflow)?;

    let tokens_for_presale = hard_cap
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_mul(100000000))
        .and_then(|f| f.checked_div(token_price))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let tokens_for_liquidity = liquidity_sols
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_mul(100000000))
        .and_then(|f| f.checked_div(listing_rate))
        .ok_or(PresaleError::ArithmeticOverflow)?;
    let presale_tokens = tokens_for_presale
        .checked_add(tokens_for_liquidity)
        .ok_or(PresaleError::ArithmeticOverflow)?;

    Ok((
        u64::try_from(liquidity_sols).unwrap(),
        u64::try_from(tokens_for_presale).unwrap(),
        u64::try_from(tokens_for_liquidity).unwrap(),
        u64::try_from(presale_tokens).unwrap(),
    ))
}

pub fn check_if_user_is_whitelisted(
    whitelist_entry: &AccountInfo,
    user_key: &Pubkey,
    presale_key: &Pubkey,
    program_id: &Pubkey,
) -> Result<()> {
    let (expected_pda, _) = Pubkey::find_program_address(
        &[b"whitelist", presale_key.as_ref(), user_key.as_ref()],
        program_id,
    );

    require!(
        whitelist_entry.key() == expected_pda,
        PresaleError::InvalidWhitelistEntry
    );

    require!(
        whitelist_entry.owner == program_id,
        PresaleError::InvalidWhitelistEntry
    );

    require!(
        **whitelist_entry.lamports.borrow() > 0,
        PresaleError::UninitializedWhitelistEntry
    );

    Ok(())
}

/// Calculate the fee for output amount
pub fn get_transfer_inverse_fee(mint_info: &AccountInfo, post_fee_amount: u64) -> Result<u64> {
    if *mint_info.owner == Token::id() {
        return Ok(0);
    }
    if post_fee_amount == 0 {
        return err!(PresaleError::Invalid);
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
                .ok_or(PresaleError::FeeCalculationError)?
        }
    } else {
        0
    };
    Ok(fee)
}

pub fn is_authorized_to_finalize_presale(presale: &PresaleState, signer: &Signer) -> Result<bool> {
    let current_time = Clock::get()?.unix_timestamp;

    let result = presale.owner == *signer.key
        || (presale.admin == *signer.key
            && ADMIN_FINALIZATION_TIMEOUT + presale.end_time > current_time)
        || (presale.manager == *signer.key && presale.launchpad_type == LaunchpadType::Degen);

    Ok(result)
}

pub fn validate_presale_pda(
    presale: &PresaleState,
    presale_key: Pubkey,
    presale_token_key: Pubkey,
) -> Result<u8> {
    let (presale_seed, bump) = Pubkey::find_program_address(
        &[
            PRESALE_SEED,
            &presale_token_key.to_bytes(),
            presale.identifier.as_ref(),
        ],
        &TokenLaunchpad::id(),
    );

    require!(presale_seed == presale_key, PresaleError::Invalid);
    require!(presale.token == presale_token_key, PresaleError::Invalid);

    Ok(bump)
}
