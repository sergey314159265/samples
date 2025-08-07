use anchor_lang::prelude::*;

use crate::{
    error::PresaleError,
    utils::transfer_sols,
};

/// Latest presale version: [`crate::constants::PRESALE_VERSION`]
/// Use helper methods `PresaleState::serialize_data` and `PresaleState::deserialize_data`
/// to interact with the presale state account.
#[account]
#[derive(InitSpace)]
pub struct PresaleState {
    pub version: u8,
    pub owner: Pubkey,
    pub token: Pubkey,
    pub token_price: u64, //1e8 for degen
    pub hard_cap: u64,
    pub soft_cap: u64,
    pub min_contribution: u64,
    pub max_contribution: u64,
    pub total_raised: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub presale_ended: bool,
    pub presale_canceled: bool,
    pub presale_refund: bool,
    pub is_init: bool,
    pub listing_rate: u64, // 1e8 for degen
    pub liquidity_lock_time: i64,
    pub liquidity_bp: u16,
    pub service_fee: u16,
    pub refund_type: RefundType,
    pub listing_opt: ListingOpt,
    pub liquidity_type: LiquidityType,
    pub listing_platform: ListingPlatform,
    pub fee_collector: Pubkey,
    #[max_len(25)]
    pub identifier: String,
    pub affiliate_enabled: bool,
    pub total_ref_amount: u64,
    pub commission_rate: u16,
    pub total_ref_count: u64,
    pub total_tokens_sold: u64,
    pub whitelist_enabled: bool,
    pub presale_type: PresaleType,
    pub tokens_claimed_by_owner: u64,
    pub owner_reward_withdrawn: bool,
    pub sol_pool_reserve: u64,
    pub token_pool_reserve: u64,
    pub launchpad_type: LaunchpadType,
    pub manager: Pubkey,
    pub admin: Pubkey,
}

#[account]
pub struct PresaleStateV0 {
    pub owner: Pubkey,
    pub token: Pubkey,
    pub token_price: u64,
    pub hard_cap: u64,
    pub soft_cap: u64,
    pub min_contribution: u64,
    pub max_contribution: u64,
    pub total_raised: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub presale_ended: bool,
    pub presale_canceled: bool,
    pub presale_refund: bool,
    pub is_init: bool,
    pub listing_rate: u64,
    pub liquidity_lock_time: i64,
    pub liquidity_bp: u16,
    pub service_fee: u16,
    pub refund_type: RefundType,
    pub listing_opt: ListingOpt,
    pub liquidity_type: LiquidityType,
    pub listing_platform: ListingPlatform,
    pub fee_collector: Pubkey,
    pub identifier: String,
    pub affiliate_enabled: bool,
    pub total_ref_amount: u64,
    pub commission_rate: u16,
    pub total_ref_count: u64,
    pub total_tokens_sold: u64,
    pub whitelist_enabled: bool,
    pub presale_type: PresaleType,
    pub tokens_claimed_by_owner: u64,
    pub owner_reward_withdrawn: bool,
    pub sol_pool_reserve: u64,
    pub token_pool_reserve: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PresaleParams {
    pub token_price: u64,
    pub hard_cap: u64,
    pub soft_cap: u64,
    pub min_contribution: u64,
    pub max_contribution: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub listing_rate: u64,
    pub liquidity_lock_time: i64,
    pub liquidity_bp: u16,
    pub service_fee: u16,
    pub refund_type: RefundType,
    pub listing_opt: ListingOpt,
    pub liquidity_type: LiquidityType,
    pub listing_platform: ListingPlatform,
    pub identifier: String,
    pub affiliate_enabled: bool,
    pub whitelist_enabled: bool,
    pub commission_rate: u16,
    pub presale_type: PresaleType,
    pub tokens_allocated: u64,
    pub launchpad_type: LaunchpadType,
    pub manager: Pubkey,
    pub admin: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PresaleType {
    HardCapped,
    FairLaunch,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RefundType {
    Burn,
    Refund,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ListingOpt {
    Auto,
    Manual,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum LiquidityType {
    Burn,
    Lock,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ListingPlatform {
    Raydium,
    Meteora,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum LaunchpadType {
    Pro,
    Degen,
}

impl PresaleState {
    /// Serializes the presale data into the provided account.
    /// Always use this method in the end of any presale-related transaction
    pub fn serialize_data<'info>(&self, presale_account: &AccountInfo<'info>) -> Result<()> {
        let mut account_data = presale_account.try_borrow_mut_data()?;
        let mut cursor = std::io::Cursor::new(&mut account_data[8..]);
        self.serialize(&mut cursor)?;

        Ok(())
    }

    /// Deserializes the presale data from the provided account.
    /// Always use this method instead of directly accessing Account<'info, PresaleState>
    pub fn deserialize_data<'info>(
        presale_account: &AccountInfo<'info>,
        signer: &Signer<'info>,
        system_program: &Program<'info, System>,
    ) -> Result<Box<Self>> {
        let data = presale_account.try_borrow_data()?;
        let expected_v0_size = 8 + PresaleStateV0::INIT_SPACE;
        let data_size = data.len();
        drop(data);

        // Migrate to V1 if needed
        if data_size == expected_v0_size {
            msg!("Migrating presale account to V1");
            Self::migrate_to_v1(presale_account, signer, system_program)?;
        }

        let data = presale_account.try_borrow_data()?;
        let presale = Box::new(PresaleState::deserialize(&mut &data[8..])?);

        Ok(presale)
    }

    fn migrate_to_v1<'info>(
        presale_state_info: &AccountInfo<'info>,
        payer: &Signer<'info>,
        system_program: &AccountInfo<'info>,
    ) -> Result<()> {
        let new_space = 8 + PresaleState::INIT_SPACE;
        presale_state_info.realloc(new_space, false)?;

        let data = presale_state_info.try_borrow_data()?;
        let old_struct = PresaleStateV0::deserialize(&mut &data[8..])?;
        drop(data);

        let new_struct = PresaleState {
            version: 1,
            owner: old_struct.owner,
            token: old_struct.token,
            token_price: old_struct.token_price,
            hard_cap: old_struct.hard_cap,
            soft_cap: old_struct.soft_cap,
            min_contribution: old_struct.min_contribution,
            max_contribution: old_struct.max_contribution,
            total_raised: old_struct.total_raised,
            start_time: old_struct.start_time,
            end_time: old_struct.end_time,
            presale_ended: old_struct.presale_ended,
            presale_canceled: old_struct.presale_canceled,
            presale_refund: old_struct.presale_refund,
            is_init: old_struct.is_init,
            listing_rate: old_struct.listing_rate,
            liquidity_lock_time: old_struct.liquidity_lock_time,
            liquidity_bp: old_struct.liquidity_bp,
            service_fee: old_struct.service_fee,
            refund_type: old_struct.refund_type,
            listing_opt: old_struct.listing_opt,
            liquidity_type: old_struct.liquidity_type,
            listing_platform: old_struct.listing_platform,
            fee_collector: old_struct.fee_collector,
            identifier: old_struct.identifier,
            affiliate_enabled: old_struct.affiliate_enabled,
            total_ref_amount: old_struct.total_ref_amount,
            commission_rate: old_struct.commission_rate,
            total_ref_count: old_struct.total_ref_count,
            total_tokens_sold: old_struct.total_tokens_sold,
            whitelist_enabled: old_struct.whitelist_enabled,
            presale_type: old_struct.presale_type,
            tokens_claimed_by_owner: old_struct.tokens_claimed_by_owner,
            owner_reward_withdrawn: old_struct.owner_reward_withdrawn,
            sol_pool_reserve: old_struct.sol_pool_reserve,
            token_pool_reserve: old_struct.token_pool_reserve,
            launchpad_type: LaunchpadType::Degen,
            manager: payer.key(),
            admin: payer.key(),
        };

        let old_rent = Rent::get()?.minimum_balance(8 + PresaleStateV0::INIT_SPACE);
        let new_rent = Rent::get()?.minimum_balance(8 + PresaleState::INIT_SPACE);

        if new_rent > old_rent {
            let additional_lamports = new_rent
                .checked_sub(old_rent)
                .ok_or(PresaleError::ArithmeticOverflow)?;

            transfer_sols(
                payer,
                presale_state_info,
                system_program,
                additional_lamports,
            )?;
        }

        let mut account_data = presale_state_info.try_borrow_mut_data()?;
        let mut cursor = std::io::Cursor::new(&mut account_data[8..]);

        new_struct.serialize(&mut cursor)?;

        Ok(())
    }
}

impl Space for PresaleType {
    const INIT_SPACE: usize = 1;
}

impl Space for RefundType {
    const INIT_SPACE: usize = 1;
}

impl Space for ListingOpt {
    const INIT_SPACE: usize = 1;
}

impl Space for LiquidityType {
    const INIT_SPACE: usize = 1;
}

impl Space for ListingPlatform {
    const INIT_SPACE: usize = 1;
}

impl Space for LaunchpadType {
    const INIT_SPACE: usize = 1;
}

impl Space for PresaleStateV0 {
    const INIT_SPACE: usize = std::mem::size_of::<PresaleStateV0>() + 25 + 8;
}
