use std::cmp::Ordering;

use crate::error::MathError;
use crate::LiquidityPool;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

// TODO make spls and t22 cross compatible in one pool
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_a_signer_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_b_signer_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_a_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_b_vault: InterfaceAccount<'info, TokenAccount>,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    token_a_amount: u64,
    token_b_amount: u64,
    fee_bps: u64,
) -> Result<()> {
    ctx.accounts.validate(token_a_amount, token_b_amount)?;
    ctx.accounts
        .transfer_to_vaults(token_a_amount, token_b_amount)?;
    let (token_a_amount_adjusted, token_b_amount_adjusted, common_precision) = normalize_amounts(
        token_a_amount,
        ctx.accounts.token_a_mint.decimals,
        token_b_amount,
        ctx.accounts.token_b_mint.decimals,
    )?;
    let constant_product = (token_a_amount_adjusted as u128)
        .checked_mul(token_b_amount_adjusted as u128)
        .ok_or(MathError::OverflowError)? as u64;

    *ctx.accounts.liquidity_pool = LiquidityPool {
        token_a: ctx.accounts.token_a_mint.key(),
        token_b: ctx.accounts.token_b_mint.key(),
        token_a_amount,
        token_b_amount,
        constant_product,
        lp_fee_bps: fee_bps,
        protocol_fee_bps: 2,
        precision: common_precision,
        bump: ctx.bumps.liquidity_pool,
    };
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
        MathError::PrecisionError
    );
    let precision_diff = precision_a.abs_diff(precision_b);
    let padding = 10_u64.pow(precision_diff as u32);

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

impl<'info> InitializePool<'info> {
    /// Validate that the amounts are greater than 0.
    pub fn validate(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        require_gt!(token_a_amount, 0);
        require_gt!(token_b_amount, 0);
        Ok(())
    }

    /// Transfers initial liquidity from the signer's token accounts to the pool vaults.
    ///
    /// Performs two CPI calls to transfer tokens:
    /// - Token A from `token_a_signer_token_account` → `token_a_vault`
    /// - Token B from `token_b_signer_token_account` → `token_b_vault`
    ///
    /// # Arguments
    /// * `token_a_amount` - Amount of token A to deposit (in token A's native decimals)
    /// * `token_b_amount` - Amount of token B to deposit (in token B's native decimals)
    ///
    /// # Errors
    /// Returns an error if either transfer CPI fails (e.g., insufficient balance).
    pub fn transfer_to_vaults(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        let token_a_transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.token_a_signer_token_account.to_account_info(),
                mint: self.token_a_mint.to_account_info(),
                to: self.token_a_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );
        let token_b_transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.token_b_signer_token_account.to_account_info(),
                mint: self.token_b_mint.to_account_info(),
                to: self.token_b_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );
        transfer_checked(
            token_a_transfer_ctx,
            token_a_amount,
            self.token_a_mint.decimals,
        )?;
        transfer_checked(
            token_b_transfer_ctx,
            token_b_amount,
            self.token_b_mint.decimals,
        )?;
        Ok(())
    }
}
