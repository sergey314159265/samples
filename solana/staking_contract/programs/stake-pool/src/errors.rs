use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // validations
    #[msg("Invalid stake pool")]
    InvalidStakePool = 0,
    #[msg("Invalid stake mint")]
    InvalidStakeMint,
    #[msg("Invalid stake entry owner")]
    InvalidStakeEntryOwner,

    // actions
    #[msg("Invalid user original mint token account")]
    InvalidUserStakeMintTokenAccount,
    #[msg("Min Stake Seconds Not Satisfied")]
    MinStakeSecondsNotSatisfied,
    #[msg("Invalid Admin")]
    InvalidAdmin,
    #[msg("Tokens Already Staked")]
    TokensAlreadyStaked,
    #[msg("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Invalid Input")]
    InvalidInput,
}   