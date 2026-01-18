use crate::error::AmmError;
use crate::{LiquidityPool, LIQUIDITY_POOL_SEED};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

// TODO remove token accounts for signers etc

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_a_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub token_b_vault: Box<Account<'info, TokenAccount>>,
    pub token_a_mint: Box<Account<'info, Mint>>,
    pub token_b_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [LIQUIDITY_POOL_SEED.as_bytes(), token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        space = LiquidityPool::DISCRIMINATOR.len() + LiquidityPool::INIT_SPACE,
    )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        init,
        payer = signer,
        mint::decimals = token_a_mint.decimals.max(token_b_mint.decimals),
        mint::authority = lp_token_mint.key(),
        seeds = [b"lp_token_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub lp_token_signer_token_account: Box<Account<'info, TokenAccount>>,
    /// Protocol fee LP token account owned by the pool PDA
    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub fee_lp_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePool>) -> Result<()> {
    require_keys_neq!(
        ctx.accounts.token_a_mint.key(),
        ctx.accounts.token_b_mint.key(),
        AmmError::IdenticalMints
    );
    **ctx.accounts.liquidity_pool = LiquidityPool {
        token_a_mint: ctx.accounts.token_a_mint.key(),
        token_b_mint: ctx.accounts.token_b_mint.key(),
        token_a_reserves: 0,
        token_b_reserves: 0,
        k_last: 0,
        bump: ctx.bumps.liquidity_pool,
    };
    Ok(())
}
