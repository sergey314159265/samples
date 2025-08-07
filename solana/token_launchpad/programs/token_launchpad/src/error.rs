use anchor_lang::prelude::*;

#[error_code]
pub enum PresaleError {
    #[msg("Presale is not active")]
    PresaleNotActive,
    #[msg("Presale has ended")]
    PresaleEnded,
    #[msg("Contribution not within limits")]
    ContributionNotWithinLimits,
    #[msg("Exceeds hard cap")]
    ExceedsHardCap,
    #[msg("Presale not ended")]
    PresaleNotEnded,
    #[msg("Presale not finalized")]
    PresaleNotFinalized,
    #[msg("Presale canceled")]
    PresaleCanceled,
    #[msg("Presale canceled")]
    PresaleRefund,
    #[msg("No tokens to claim")]
    NoTokensToClaim,
    #[msg("Presale is not refunded")]
    PresaleNotRefunded,
    #[msg("not authorized")]
    Unauthorized,
    #[msg("Liquidity is not unlocked yet")]
    LiquidityLocked,
    #[msg("Presale not ended or canceled")]
    PresaleEndedOrCanceled,
    #[msg("Presale not cancelled")]
    PresaleNotCancelled,
    #[msg("Insufficient Funds")]
    InsufficientFunds,
    #[msg("Invalid")]
    Invalid,
    #[msg("Invalid referrer")]
    InvalidReferrer,
    #[msg("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Invalid raydium cp swap program")]
    InvalidRaydiumCpSwapProgram,
    #[msg("Invalid raydium authority")]
    InvalidRaydiumAuthority,
    #[msg("Invalid raydium pool state")]
    InvalidRaydiumPoolState,
    #[msg("Invalid raydium amm config")]
    InvalidRaydiumAmmConfig,
    #[msg("Invalid raydium token mint account ordering")]
    InvalidRaydiumTokenMintAccountOrdering,
    #[msg("Invalid raydium LP token mint account")]
    InvalidRaydiumLpTokenMintAccount,
    #[msg("Invalid raydium token 0 vault account")]
    InvalidRaydiumToken0VaultAccount,
    #[msg("Invalid raydium token 1 vault account")]
    InvalidRaydiumToken1VaultAccount,
    #[msg("Invalid raydium observation state account")]
    InvalidRaydiumObservationStateAccount,
    #[msg("Invalid fee collector")]
    InvalidFeeCollector,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Unable to create program address (PDA)")]
    UnableToCreateProgramAddress,
    #[msg("Presale finalization preconditions not met")]
    PresaleFinalizationPreconditionsNotMet,
    #[msg("Invalid whitelist entry")]
    InvalidWhitelistEntry,
    #[msg("Uninitialized whitelist entry")]
    UninitializedWhitelistEntry,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Instruction is inaccessible if owner reward is withdrawn")]
    OwnerRewardWithdrawn,
    #[msg("Instruction can't be called for this listing platform")]
    InvalidListingPlatform,
    #[msg("Platform profit should be greater than minimum amount")]
    PlatformProfitTooLow,
}
