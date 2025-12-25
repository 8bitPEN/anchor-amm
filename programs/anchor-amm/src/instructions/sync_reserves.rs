use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{helpers::ReserveSyncer, LiquidityPool, LIQUIDITY_POOL_SEED};
#[derive(Accounts)]
pub struct SyncReserves<'info> {
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
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<SyncReserves>) -> Result<()> {
    ctx.accounts.sync_reserves();
    Ok(())
}

impl<'info> ReserveSyncer<'info> for SyncReserves<'info> {
    fn liquidity_pool(&mut self) -> &mut Account<'info, LiquidityPool> {
        &mut self.liquidity_pool
    }
    fn token_a_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_a_vault
    }

    fn token_b_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_b_vault
    }
}
