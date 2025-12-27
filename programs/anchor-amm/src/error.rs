use anchor_lang::prelude::*;

/// Arithmetic and mathematical operation errors
#[error_code]
pub enum MathError {
    #[msg("Arithmetic operation overflowed")]
    Overflow,
    #[msg("Cannot divide by zero")]
    DivisionByZero,
    #[msg("Token precision must be between 1 and 12 decimals")]
    InvalidPrecision,
}

/// AMM protocol errors
#[error_code]
pub enum AmmError {
    // Input validation
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Token A and Token B mints cannot be the same")]
    IdenticalMints,
    #[msg("Transaction deadline has passed")]
    DeadlineExceeded,

    // Liquidity errors
    #[msg("Pool has insufficient liquidity for this operation")]
    InsufficientLiquidity,
    #[msg("Initial liquidity deposit must mint more than 1000 LP tokens")]
    InsufficientInitialLiquidity,
    #[msg("Cannot withdraw minimum locked liquidity (1000 LP tokens)")]
    MinimumLiquidityLocked,

    // Slippage protection
    #[msg("Output amount is less than minimum specified")]
    SlippageExceeded,

    // Skim operation
    #[msg("No excess tokens in vault to skim")]
    NoExcessTokens,
}
