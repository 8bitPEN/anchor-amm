pub use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    error::{AmmError, MathError},
    helpers::{get_withdraw_amount, LPBurner, ReserveSyncer, VaultWithdrawer},
    LiquidityPool, LIQUIDITY_POOL_SEED,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub lp_token_signer_token_account: Account<'info, TokenAccount>,
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
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"lp_token_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED.as_bytes(), token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
pub fn handler(
    ctx: Context<Withdraw>,
    lp_amount_to_burn: u64,
    amount_a_min: u64,
    amount_b_min: u64,
    expiration: i64,
) -> Result<()> {
    require!(lp_amount_to_burn > 0, AmmError::ZeroAmount);
    require_gt!(
        expiration,
        Clock::get()?.unix_timestamp,
        AmmError::DeadlineExceeded,
    );

    let lp_supply = ctx.accounts.lp_token_mint.supply as u128;
    let lp_amount = lp_amount_to_burn as u128;

    let token_a_out: u64 = get_withdraw_amount(
        ctx.accounts.liquidity_pool.token_a_reserves as u128,
        lp_amount,
        lp_supply,
    )?
    .try_into()
    .map_err(|_| MathError::Overflow)?;

    let token_b_out: u64 = get_withdraw_amount(
        ctx.accounts.liquidity_pool.token_b_reserves as u128,
        lp_amount,
        lp_supply,
    )?
    .try_into()
    .map_err(|_| MathError::Overflow)?;

    require!(
        token_a_out >= amount_a_min && token_b_out >= amount_b_min,
        AmmError::SlippageExceeded
    );

    require!(
        token_a_out > 0 && token_b_out > 0,
        AmmError::InsufficientLiquidity
    );

    ctx.accounts.withdraw(token_a_out, token_b_out)?;
    ctx.accounts.burn_lp_tokens(lp_amount_to_burn)?;

    // Reload vaults and sync reserves
    ctx.accounts.token_a_vault.reload()?;
    ctx.accounts.token_b_vault.reload()?;
    ctx.accounts.sync_reserves();

    Ok(())
}
impl<'info> LPBurner<'info> for Withdraw<'info> {
    fn token_program(&self) -> &Program<'info, Token> {
        &self.token_program
    }

    fn lp_token_mint(&self) -> &Account<'info, Mint> {
        &self.lp_token_mint
    }

    fn lp_token_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.lp_token_signer_token_account
    }

    fn signer(&self) -> &Signer<'info> {
        &self.signer
    }
}
impl<'info> VaultWithdrawer<'info> for Withdraw<'info> {
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

impl<'info> ReserveSyncer<'info> for Withdraw<'info> {
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
