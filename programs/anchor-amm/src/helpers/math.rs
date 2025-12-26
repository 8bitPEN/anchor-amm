use anchor_lang::prelude::*;
use std::cmp::Ordering;

use crate::error::{AMMError, MathError};
// TODO (Pen): Revisit this, when you know more about max or min decimals and normalization
pub fn normalize_amounts(
    amount_a: u64,
    precision_a: u8,
    amount_b: u64,
    precision_b: u8,
) -> Result<(u64, u64, u8)> {
    // maybe this isn't neccessary
    require!(
        precision_a > 0 && precision_a <= 12 && precision_b > 0 && precision_b <= 12,
        MathError::PrecisionError
    );
    let precision_diff = precision_a.abs_diff(precision_b);
    let padding = 10_u64.pow(precision_diff as u32);
    // TODO casting up and down, this is dangerous now, gotta cast to u128 first before math'ing
    let (adjusted_amount_a, adjusted_amount_b, precision) = match precision_a.cmp(&precision_b) {
        Ordering::Equal => (amount_a, amount_b, precision_a),
        Ordering::Greater => (
            amount_a,
            amount_b
                .checked_mul(padding)
                .ok_or(MathError::OverflowError)?,
            precision_a,
        ),
        Ordering::Less => (
            amount_a
                .checked_mul(padding)
                .ok_or(MathError::OverflowError)?,
            amount_b,
            precision_b,
        ),
    };
    Ok((adjusted_amount_a, adjusted_amount_b, precision))
}
// TODO (Pen): Maybe a common precision would be better at 12 or something large.
pub fn common_precision(precision_a: u8, precision_b: u8) -> u8 {
    precision_a.max(precision_b)
}

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
/// * `AMMError::InssufficientAmount` - If `token_a_amount` is zero
/// * `AMMError::InsufficientLiquidity` - If either reserve is zero
/// * `MathError::OverflowError` - If multiplication overflows
/// * `MathError::ZeroDivisionError` - If division by zero occurs
pub fn quote(token_a_amount: u128, token_a_reserves: u128, token_b_reserves: u128) -> Result<u128> {
    require_gt!(token_a_amount, 0, AMMError::InssufficientAmount);
    require!(
        token_a_reserves > 0 && token_b_reserves > 0,
        AMMError::InsufficientLiquidity
    );
    let result = token_a_amount
        .checked_mul(token_b_reserves)
        .ok_or(MathError::OverflowError)?
        .checked_div(token_a_reserves)
        .ok_or(MathError::ZeroDivisionError)?;
    Ok(result)
}

pub fn calculate_constant_product(token_a_amount: u128, token_b_amount: u128) -> Result<u128> {
    Ok(token_a_amount
        .checked_mul(token_b_amount)
        .ok_or(MathError::OverflowError)?)
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
/// * `MathError::OverflowError` - If any arithmetic operation overflows
/// * `MathError::ZeroDivisionError` - If `reserve_in + amount_in` is zero
pub fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128> {
    let numerator = reserve_out
        .checked_mul(amount_in)
        .ok_or(MathError::OverflowError)?;
    let denominator = reserve_in
        .checked_add(amount_in)
        .ok_or(MathError::OverflowError)?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::ZeroDivisionError.into())
}
