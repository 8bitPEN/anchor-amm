use crate::LiquidityPool;
use crate::error::AMMError;
use crate::helpers::common_precision;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};



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
) -> Result<()> {
    
    require_keys_neq!(ctx.accounts.token_a_mint.key(),
    ctx.accounts.token_b_mint.key(), AMMError::TokenMintsEqual); 
    let precision = common_precision(
        ctx.accounts.token_a_mint.decimals, 
        ctx.accounts.token_b_mint.decimals,         
    );
    *ctx.accounts.liquidity_pool = LiquidityPool {
        token_a_mint: ctx.accounts.token_a_mint.key(),
        token_b_mint: ctx.accounts.token_b_mint.key(),
        token_a_reserves: 0,
        token_b_reserves: 0,
        lp_fee_bps: 3,
        protocol_fee_bps: 2,
        precision,
        bump: ctx.bumps.liquidity_pool,
    };
    Ok(())
}
