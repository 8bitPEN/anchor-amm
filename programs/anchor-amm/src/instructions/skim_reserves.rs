pub use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{error::AMMError, helpers::VaultWithdrawer, LiquidityPool, LIQUIDITY_POOL_SEED};

#[derive(Accounts)]
pub struct SkimReserves<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED.as_ref(), token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_b_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_a_signer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_b_signer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<SkimReserves>) -> Result<()> {
    let token_a_excess = ctx
        .accounts
        .token_a_vault
        .amount
        .saturating_sub(ctx.accounts.liquidity_pool.token_a_reserves);
    let token_b_excess = ctx
        .accounts
        .token_b_vault
        .amount
        .saturating_sub(ctx.accounts.liquidity_pool.token_b_reserves);

    require!(
        token_a_excess > 0 || token_b_excess > 0,
        AMMError::NothingToSkim
    );

    ctx.accounts.withdraw(token_a_excess, token_b_excess)?;
    Ok(())
}

impl<'info> VaultWithdrawer<'info> for SkimReserves<'info> {
    fn token_program(&self) -> &Program<'info, Token> {
        &self.token_program
    }

    fn token_a_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_a_signer_token_account
    }

    fn token_b_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_b_signer_token_account
    }

    fn token_a_mint(&self) -> &Account<'info, Mint> {
        &self.token_a_mint
    }

    fn token_b_mint(&self) -> &Account<'info, Mint> {
        &self.token_b_mint
    }

    fn token_a_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_a_vault
    }

    fn token_b_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_b_vault
    }

    fn liquidity_pool(&self) -> &Account<'info, LiquidityPool> {
        &self.liquidity_pool
    }
}
