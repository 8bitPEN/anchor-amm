use crate::error::MathError;
use crate::LiquidityPool;
use crate::helpers::{LPMinter, VaultDepositor, calculate_constant_product, common_precision};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};


// TODO (Pen): Make the precision have a bigger upper limit (19).
// OPTIMIZE (Pen): Maybe making the initialize instruction only to initialize would be better, 
// OPTIMIZE (Pen): so that the user doesn't need to actually deposit immediately
// TODO (Pen): Minimum liquidity burn
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
    pub token_a_signer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_b_signer_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_b_vault: Account<'info, TokenAccount>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        seeds = [b"liquidity_pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        space = LiquidityPool::DISCRIMINATOR.len() + LiquidityPool::INIT_SPACE,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account(
        init, 
        payer = signer,
        mint::decimals = token_a_mint.decimals.max(token_b_mint.decimals),
        mint::authority = lp_token_mint.key(),
        seeds = [b"lp_token_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub lp_token_signer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
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
       
    let lp_tokens_to_mint: u64 = calculate_constant_product(token_a_amount as u128, token_b_amount as u128)?
        .isqrt()
        .try_into()
        .map_err(|_| MathError::OverflowError)?;
    ctx.accounts.mint_lp_tokens(lp_tokens_to_mint, ctx.bumps.lp_token_mint)?;

    *ctx.accounts.liquidity_pool = LiquidityPool {
        token_a_mint: ctx.accounts.token_a_mint.key(),
        token_b_mint: ctx.accounts.token_b_mint.key(),
        token_a_reserves: token_a_amount,
        token_b_reserves: token_b_amount,
        lp_fee_bps: fee_bps,
        protocol_fee_bps: 2,
        precision: common_precision(ctx.accounts.token_a_mint.decimals, ctx.accounts.token_b_mint.decimals),
        bump: ctx.bumps.liquidity_pool,
    };
    Ok(())
}


impl<'info> InitializePool<'info> {
    /// Validate that the amounts are greater than 0.
    pub fn validate(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        require_gt!(token_a_amount, 0);
        require_gt!(token_b_amount, 0);
        Ok(())
    }

}
impl<'info> LPMinter<'info> for InitializePool<'info> {
    fn token_program(&self) -> &Program<'info, Token> {
        &self.token_program
    }

    fn token_a_mint(&self) -> &Account<'info, Mint> {
        &self.token_a_mint
    }

    fn token_b_mint(&self) -> &Account<'info, Mint> {
        &self.token_b_mint
    }

    fn lp_token_mint(&self) -> &Account<'info, Mint> {
        &self.lp_token_mint
    }

    fn lp_token_signer_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.lp_token_signer_token_account
    }
}
impl<'info> VaultDepositor<'info> for InitializePool<'info> {
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

    fn signer(&self) -> &Signer<'info> {
        &self.signer
    }
}