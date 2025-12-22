use anchor_lang::prelude::*;
use std::cmp::Ordering;

use crate::error::MathError;
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

pub fn calculate_constant_product(token_a_amount: u128, token_b_amount: u128) -> Result<u128> {
    Ok(token_a_amount
        .checked_mul(token_b_amount)
        .ok_or(MathError::OverflowError)?)
}
