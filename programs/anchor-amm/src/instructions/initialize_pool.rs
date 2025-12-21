use std::cmp::Ordering;

use crate::error::FunctionError;
use crate::LiquidityPool;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_a_mint: InterfaceAccount<'info, Mint>,
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = signer,
        seeds = [b"liquidity_pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        space = LiquidityPool::DISCRIMINATOR.len() + LiquidityPool::INIT_SPACE,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<()> {
    ctx.accounts.validate(token_a_amount, token_b_amount)?;
    
    Ok(())
}
///
// TODO (Pen): Revisit this, when you know more about max or min decimals and normalization
fn normalize_amounts(
    amount_a: u64,
    precision_a: u8,
    amount_b: u64,
    precision_b: u8,
) -> Result<(u64, u64, u8)> {
    // maybe this isn't neccessary
    require!(
        precision_a > 0 && precision_a <= 12 && precision_b > 0 && precision_b <= 12,
        FunctionError::PrecisionError
    );
    let precision_diff = precision_a.abs_diff(precision_b);
    let padding = 10_u64.pow(precision_diff as u32);

    let (adjusted_amount_a, adjusted_amount_b, precision) = match precision_a.cmp(&precision_b) {
        Ordering::Equal => (amount_a, amount_b, precision_a),
        Ordering::Greater => (
            amount_a,
            amount_b
                .checked_mul(padding)
                .ok_or(FunctionError::OverflowError)?,
            precision_a,
        ),
        Ordering::Less => (
            amount_a
                .checked_mul(padding)
                .ok_or(FunctionError::OverflowError)?,
            amount_b,
            precision_b,
        ),
    };
    Ok((adjusted_amount_a, adjusted_amount_b, precision))
}
impl<'info> InitializePool<'info> {
    /// Validate that the amounts are greater than 0.
    pub fn validate(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        require_gt!(token_a_amount, 0);
        require_gt!(token_b_amount, 0);
        Ok(())
    }
}
