#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("6QDo8CPvZji1xzuZpTPhurXoPKK8r3jc4QkTcTFmuRBm");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

declare_program!(dynamic_vault);
declare_program!(dynamic_amm);

use instructions::*;
use state::presale::PresaleParams;

#[program]
pub mod token_launchpad {
    use super::*;

    pub fn initialize_presale(
        ctx: Context<InitializePresale>,
        presale_config: PresaleParams,
    ) -> Result<()> {
        instructions::init_presale(ctx, presale_config)
    }

    pub fn initialize_vaults(ctx: Context<InitializeVaults>) -> Result<()> {
        instructions::init_vaults(ctx)
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        instructions::contribute(ctx, amount)
    }

    pub fn contribute_affiliate(ctx: Context<ContributeAffiliate>, amount: u64) -> Result<()> {
        instructions::contribute_affiliate(ctx, amount)
    }

    pub fn finalize_presale(ctx: Context<FinalizePresale>) -> Result<()> {
        instructions::finalize_presale(ctx)
    }

    pub fn finalize_presale_raydium_pool(ctx: Context<FinalizePresaleRaydiumPool>) -> Result<()> {
        instructions::finalize_presale_raydium_pool(ctx)
    }

    pub fn finalize_presale_meteora_pool(ctx: Context<FinalizePresaleMeteoraPool>) -> Result<()> {
        instructions::finalize_presale_meteora_pool(ctx)
    }

    pub fn finalize_presale_init_vault_meteora(
        ctx: Context<FinalizePresaleInitVaultMeteora>,
    ) -> Result<()> {
        instructions::finalize_presale_init_vault_meteora(ctx)
    }

    pub fn finalize_presale_meteora_lock_pool(ctx: Context<LockPoolMeteora>) -> Result<()> {
        instructions::finalize_presale_meteora_lock_pool(ctx)
    }

    pub fn claim_fee_meteora(ctx: Context<ClaimFeeMeteora>) -> Result<()> {
        instructions::claim_fee_meteora(ctx)
    }

    pub fn distribute_fee_meteora(
        ctx: Context<DistributeFeeMeteora>,
        minimum_amount: u64,
    ) -> Result<()> {
        instructions::distribute_fee_meteora(ctx, minimum_amount)
    }

    pub fn init_meteora_pool_authority(ctx: Context<InitMeteoraPoolAuthority>) -> Result<()> {
        instructions::init_meteora_pool_authority(ctx)
    }

    pub fn cancel_presale(ctx: Context<CancelPresale>) -> Result<()> {
        instructions::cancel_presale(ctx)
    }

    pub fn withdraw_unsold_tokens(ctx: Context<WithdrawUnsoldTokens>) -> Result<()> {
        instructions::withdraw_unsold_tokens(ctx)
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        instructions::claim_tokens(ctx)
    }

    pub fn refund_contributors(ctx: Context<RefundContributors>) -> Result<()> {
        instructions::refund_contributors(ctx)
    }

    pub fn withdraw_affiliate_commission(ctx: Context<WithdrawAffiliateCommission>) -> Result<()> {
        instructions::withdraw_affiliate_commission(ctx)
    }

    pub fn finalize_transfer(ctx: Context<FinalizeTransfer>) -> Result<()> {
        instructions::finalize_transfer(ctx)
    }

    pub fn finalize_wrap_sol(ctx: Context<FinalizeWrapSol>) -> Result<()> {
        instructions::finalize_wrap_sol(ctx)
    }

    pub fn withdraw_locked_lp_tokens(ctx: Context<WithdrawLockedLpTokens>) -> Result<()> {
        instructions::withdraw_locked_lp_tokens(ctx)
    }

    pub fn finalize_lp_lock_burn(ctx: Context<FinalizeLpLockBurn>) -> Result<()> {
        instructions::finalize_lp_lock_burn(ctx)
    }

    pub fn withdraw_owner_reward(ctx: Context<WithdrawOwnerReward>) -> Result<()> {
        instructions::withdraw_owner_reward(ctx)
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, user: Pubkey) -> Result<()> {
        instructions::add_to_whitelist(ctx, user)
    }
}
