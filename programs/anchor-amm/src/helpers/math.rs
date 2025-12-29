use crate::error::{AmmError, MathError};
use anchor_lang::prelude::*;
/// Calculates the equivalent amount of token B for a given amount of token A,
/// based on current pool reserves. Used for proportional deposits/withdrawals.
///
/// Formula: `token_b_out = (token_a_amount * token_b_reserves) / token_a_reserves`
///
/// # Arguments
/// * `token_a_amount` - The amount of token A being quoted
/// * `token_a_reserves` - Current token A reserves in the pool
/// * `token_b_reserves` - Current token B reserves in the pool
///
/// # Errors
/// * `AmmError::ZeroAmount` - If `token_a_amount` is zero
/// * `AmmError::InsufficientLiquidity` - If either reserve is zero
/// * `MathError::Overflow` - If multiplication overflows
/// * `MathError::DivisionByZero` - If division by zero occurs
pub fn quote(token_a_amount: u128, token_a_reserves: u128, token_b_reserves: u128) -> Result<u128> {
    require_gt!(token_a_amount, 0, AmmError::ZeroAmount);
    require!(
        token_a_reserves > 0 && token_b_reserves > 0,
        AmmError::InsufficientLiquidity
    );
    let result = token_a_amount
        .checked_mul(token_b_reserves)
        .ok_or(MathError::Overflow)?
        .checked_div(token_a_reserves)
        .ok_or(MathError::DivisionByZero)?;
    Ok(result)
}

pub fn calculate_constant_product(token_a_amount: u128, token_b_amount: u128) -> Result<u128> {
    Ok(token_a_amount
        .checked_mul(token_b_amount)
        .ok_or(MathError::Overflow)?)
}

/// Calculates the output amount for a constant product swap.
///
/// Formula: `Δy = (y * Δx) / (x + Δx)`
///
/// This is derived from the constant product invariant `x * y = k`:
/// - After swap: `(x + Δx) * (y - Δy) = k`
/// - Solving for Δy gives the formula above
///
/// # Arguments
/// * `amount_in` - The input token amount (Δx)
/// * `reserve_in` - The input token's reserve (x)
/// * `reserve_out` - The output token's reserve (y)
///
/// # Returns
/// The output token amount (Δy)
///
/// # Errors
/// * `MathError::Overflow` - If any arithmetic operation overflows
/// * `MathError::DivisionByZero` - If `reserve_in + amount_in` is zero
pub fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128> {
    let numerator = reserve_out
        .checked_mul(amount_in)
        .ok_or(MathError::Overflow)?;
    let denominator = reserve_in
        .checked_add(amount_in)
        .ok_or(MathError::Overflow)?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivisionByZero.into())
}

/// Calculates the amount of tokens received when burning LP tokens.
///
/// Formula: `amount_out = (reserves * lp_amount) / lp_supply`
///
/// This gives the proportional share of a token reserve based on
/// the fraction of total LP supply being burned.
///
/// # Arguments
/// * `reserves` - The token's current reserve in the pool
/// * `lp_amount` - The amount of LP tokens being burned
/// * `lp_supply` - The total supply of LP tokens
///
/// # Returns
/// The amount of tokens to withdraw
///
/// # Errors
/// * `MathError::Overflow` - If multiplication overflows
/// * `MathError::DivisionByZero` - If `lp_supply` is zero
pub fn get_withdraw_amount(reserves: u128, lp_amount: u128, lp_supply: u128) -> Result<u128> {
    reserves
        .checked_mul(lp_amount)
        .ok_or(MathError::Overflow)?
        .checked_div(lp_supply)
        .ok_or(MathError::DivisionByZero.into())
}
