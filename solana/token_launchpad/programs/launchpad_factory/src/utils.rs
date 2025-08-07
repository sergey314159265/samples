use anchor_lang::{
    prelude::*,
    solana_program::{
        clock::Clock,
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
            ExtensionType,
            StateWithExtensions,
        },
    },
    token_interface::{
        spl_token_2022::extension::BaseStateWithExtensions,
        transfer_checked,
        Mint,
        TransferChecked,
    },
};

use crate::error::FactoryError;

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
    mint: InterfaceAccount<'info, Mint>,
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    // Set up the CPI (Cross-Program Invocation) context
    let cpi_accounts = TransferChecked {
        from: from.clone(),
        mint: mint.to_account_info().clone(),
        to: to.clone(),
        authority: owner.clone(),
    };
    let cpi_program = token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Invoke the transfer based on the token program
    transfer_checked(cpi_ctx, amount, mint.decimals)?;
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
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let net_hardcap = hard_cap
        .checked_sub(s_fee_amount)
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let liquidity_sols = net_hardcap
        .checked_mul(liquidity_bp)
        .and_then(|f| f.checked_div(10000))
        .ok_or(FactoryError::ArithmeticOverflow)?;

    let tokens_for_presale = hard_cap
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_div(token_price))
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let tokens_for_liquidity = liquidity_sols
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_div(listing_rate))
        .ok_or(FactoryError::ArithmeticOverflow)?;

    let presale_tokens = tokens_for_presale
        .checked_add(tokens_for_liquidity)
        .ok_or(FactoryError::ArithmeticOverflow)?;

    Ok((
        u64::try_from(liquidity_sols).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(tokens_for_presale).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(tokens_for_liquidity).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(presale_tokens).map_err(|_| FactoryError::TypeConversionError)?,
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
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let net_hardcap = hard_cap
        .checked_sub(s_fee_amount)
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let liquidity_sols = net_hardcap
        .checked_mul(liquidity_bp)
        .and_then(|f| f.checked_div(10000))
        .ok_or(FactoryError::ArithmeticOverflow)?;

    let tokens_for_presale = hard_cap
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_mul(100000000))
        .and_then(|f| f.checked_div(token_price))
        .ok_or(FactoryError::ArithmeticOverflow)?;

    let tokens_for_liquidity = liquidity_sols
        .checked_mul(decimals_result)
        .and_then(|f| f.checked_mul(100000000))
        .and_then(|f| f.checked_div(listing_rate))
        .ok_or(FactoryError::ArithmeticOverflow)?;
    let presale_tokens = tokens_for_presale
        .checked_add(tokens_for_liquidity)
        .ok_or(FactoryError::ArithmeticOverflow)?;

    Ok((
        u64::try_from(liquidity_sols).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(tokens_for_presale).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(tokens_for_liquidity).map_err(|_| FactoryError::TypeConversionError)?,
        u64::try_from(presale_tokens).map_err(|_| FactoryError::TypeConversionError)?,
    ))
}

/// Calculate the fee for output amount
pub fn get_transfer_inverse_fee(mint_info: &AccountInfo, post_fee_amount: u64) -> Result<u64> {
    if *mint_info.owner == Token::id() {
        return Ok(0);
    }
    if post_fee_amount == 0 {
        return err!(FactoryError::Invalid);
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
                .ok_or(FactoryError::FeeCalculationError)?
        }
    } else {
        0
    };
    Ok(fee)
}

pub fn is_supported_mint(mint_account: &InterfaceAccount<Mint>) -> Result<bool> {
    let mint_info = mint_account.to_account_info();
    if *mint_info.owner == Token::id() {
        return Ok(true);
    }
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;
    let extensions = mint.get_extension_types()?;
    for e in extensions {
        if e != ExtensionType::TransferFeeConfig
            && e != ExtensionType::MetadataPointer
            && e != ExtensionType::TokenMetadata
        {
            return Ok(false);
        }
    }
    Ok(true)
}
