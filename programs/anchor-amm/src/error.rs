use anchor_lang::prelude::*;

// this probably needs better naming
#[error_code]
pub enum MathError {
    #[msg("The given precision for the function was out of range.")]
    PrecisionError,
    #[msg("The calculation overflowed")]
    OverflowError,
}
