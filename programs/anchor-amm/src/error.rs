use anchor_lang::prelude::*;

// this probably needs better naming
#[error_code]
pub enum MathError {
    #[msg("The given precision for the function was out of range.")]
    PrecisionError,
    #[msg("The calculation overflowed")]
    OverflowError,
    #[msg("Division by zero.")]
    ZeroDivisionError,
}
// TODO (Pen): is this even allowed, are two "error_codes allowed?"
// TODO (Pen): Better error messages, names
#[error_code]
pub enum AMMError {
    #[msg("Inssufficient amount")]
    InssufficientAmount,
    #[msg("Inssufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Slippage limit exceeded")]
    SlippageLimitExceeded, // we could probably name this better lol
    #[msg("Zero input amount")]
    ZeroInputAmount,
    #[msg("The provided liquidity is too low")]
    LowLiquidity,
    #[msg("The two mints are the same")]
    TokenMintsEqual,
    #[msg("There is nothing to skim")]
    NothingToSkim,
}
