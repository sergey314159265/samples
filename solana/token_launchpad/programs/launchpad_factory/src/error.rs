use anchor_lang::prelude::*;

#[error_code]
pub enum FactoryError {
    #[msg("Caller must be admin")]
    Unauthorized,
    #[msg("Invalid fee account")]
    InvalidFeeAccount,
    #[msg("Invalid")]
    Invalid,
    #[msg("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Fee wallet not available")]
    NoFeeWallet,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Type conversion error")]
    TypeConversionError,
    #[msg("Token is not suitable for liquidity pool")]
    InvalidMint,
    #[msg("Hardcap is beyond the limits")]
    InvalidHardcap,
}
