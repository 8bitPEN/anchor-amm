use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    error::{AmmError, MathError},
    helpers::{get_amount_out, ReserveSyncer, VaultDepositor, VaultWithdrawer},
    LiquidityPool, LIQUIDITY_POOL_SEED,
};
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_0_signer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_1_signer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_0_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_1_vault: Account<'info, TokenAccount>,
    pub token_0_mint: Account<'info, Mint>,
    pub token_1_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [
            b"lp_token_mint", 
            liquidity_pool.token_a_mint.as_ref(), 
            liquidity_pool.token_b_mint.as_ref()
        ],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED.as_bytes(), token_0_mint.key().as_ref(), token_1_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Swap>,
    token_0_amount: u64,
    token_1_min_amount: u64,
    expiration: i64,
) -> Result<()> {
    let is_token_a = ctx.accounts.token_0_mint.key() == ctx.accounts.liquidity_pool.token_a_mint;
    ctx.accounts.validate(token_0_amount, token_1_min_amount, expiration, is_token_a)?;
    let token_0_amount_with_fees = (token_0_amount as u128)
        .checked_mul(997)
        .ok_or(MathError::Overflow)?
        .checked_div(1000)
        .ok_or(MathError::Overflow)?;

    let token_1_out: u64 = if is_token_a {
        get_amount_out(
            token_0_amount_with_fees,
            ctx.accounts.liquidity_pool.token_a_reserves as u128,
            ctx.accounts.liquidity_pool.token_b_reserves as u128,
        )
    } else {
        get_amount_out(
            token_0_amount_with_fees,
            ctx.accounts.liquidity_pool.token_b_reserves as u128,
            ctx.accounts.liquidity_pool.token_a_reserves as u128,
        )
    }?
    .try_into()
    .map_err(|_| MathError::Overflow)?;
    require_gt!(token_1_out, token_1_min_amount, AmmError::SlippageExceeded);

    // Deposit token_0 from user into vault
    ctx.accounts.deposit_token(
        &ctx.accounts.token_0_mint,
        &ctx.accounts.token_0_signer_token_account,
        &ctx.accounts.token_0_vault,
        token_0_amount,
    )?;

    // Withdraw token_1 from vault to user
    ctx.accounts.withdraw_token(
        &ctx.accounts.token_1_mint,
        &ctx.accounts.token_1_vault,
        &ctx.accounts.token_1_signer_token_account,
        token_1_out,
    )?;

    // Reload vault accounts to get updated balances after transfers
    ctx.accounts.token_0_vault.reload()?;
    ctx.accounts.token_1_vault.reload()?;

    // Sync reserves with actual vault balances
    ctx.accounts.sync_reserves();

    Ok(())
}

impl<'info> Swap<'info> {
    pub fn validate(
        &self,
        token_0_amount: u64,
        token_1_min_amount: u64,
        expiration: i64,
        is_token_a: bool,
    ) -> Result<()> {
        require!(
            token_0_amount > 0 || token_1_min_amount > 0,
            AmmError::ZeroAmount
        );
        require_gt!(
            expiration,
            Clock::get()?.unix_timestamp,
            AmmError::DeadlineExceeded,
        );
        require!(
            token_1_min_amount < self.token_1_vault.amount,
            AmmError::InsufficientLiquidity
        );

        if is_token_a {
            require_keys_eq!(
                self.token_1_mint.key(),
                self.liquidity_pool.token_b_mint,
                AmmError::MintMismatch
            );
        } else {
            require_keys_eq!(
                self.token_0_mint.key(),
                self.liquidity_pool.token_b_mint,
                AmmError::MintMismatch
            );
            require_keys_eq!(
                self.token_1_mint.key(),
                self.liquidity_pool.token_a_mint,
                AmmError::MintMismatch
            );
        }

        Ok(())
    }
}

impl<'info> VaultWithdrawer<'info> for Swap<'info> {
    fn token_program(&self) -> &Program<'info, Token> {
        &self.token_program
    }

    fn token_a_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_0_signer_token_account
    }

    fn token_b_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_1_signer_token_account
    }

    fn token_a_mint(&self) -> &Account<'info, Mint> {
        &self.token_0_mint
    }

    fn token_b_mint(&self) -> &Account<'info, Mint> {
        &self.token_1_mint
    }

    fn token_a_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_0_vault
    }

    fn token_b_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_1_vault
    }

    fn liquidity_pool(&self) -> &Account<'info, LiquidityPool> {
        &self.liquidity_pool
    }
}

impl<'info> VaultDepositor<'info> for Swap<'info> {
    fn token_program(&self) -> &Program<'info, Token> {
        &self.token_program
    }

    fn token_a_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_0_signer_token_account
    }

    fn token_b_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.token_1_signer_token_account
    }

    fn token_a_mint(&self) -> &Account<'info, Mint> {
        &self.token_0_mint
    }

    fn token_b_mint(&self) -> &Account<'info, Mint> {
        &self.token_1_mint
    }

    fn token_a_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_0_vault
    }

    fn token_b_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_1_vault
    }

    fn signer(&self) -> &Signer<'info> {
        &self.signer
    }
}

impl<'info> ReserveSyncer<'info> for Swap<'info> {
    fn liquidity_pool(&mut self) -> &mut Account<'info, LiquidityPool> {
        &mut self.liquidity_pool
    }

    fn token_a_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_0_vault
    }

    fn token_b_vault(&self) -> &Account<'info, TokenAccount> {
        &self.token_1_vault
    }
}
