use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked},
};

use crate::{
    error::{AMMError, MathError},
    helpers::quote,
    LiquidityPool,
};
// TODO (Pen): What happens if an attacker just sends coins to the ATA of the liquidity pool? The fix is probably to keep count of the actual reserves
// TODO (Pen): Should there be deposit fees? Not gonna bother with fees for now.
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
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
        seeds = [b"liquidity_pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Deposit>,
    token_a_amount_desired: u64,
    token_b_amount_desired: u64,
    token_a_amount_min: u64,
    token_b_amount_min: u64,
) -> Result<()> {
    let token_a_amount_desired = token_a_amount_desired as u128;
    let token_b_amount_desired = token_b_amount_desired as u128;
    let token_a_amount_min = token_a_amount_min as u128;
    let token_b_amount_min = token_b_amount_min as u128;
    let (token_a_deposit_amount, token_b_deposit_amount) = ctx.accounts.optimize_deposit_amounts(
        token_a_amount_desired,
        token_b_amount_desired,
        token_a_amount_min,
        token_b_amount_min,
    )?;
    ctx.accounts.transfer_to_vaults(
        token_a_deposit_amount
            .try_into()
            .map_err(|_| MathError::OverflowError)?,
        token_b_deposit_amount
            .try_into()
            .map_err(|_| MathError::OverflowError)?,
    )?;
    //NOTE in Uniswap, they decide how many tokens to mint based on the minimum ratio, but since we have already optimized the deposits,
    //NOTE it's safe to just go with token a, since it should be equal to token_b.
    let lp_tokens_to_mint: u64 = token_a_deposit_amount
        .checked_mul(ctx.accounts.lp_token_mint.supply as u128)
        .ok_or(MathError::OverflowError)?
        .checked_div(ctx.accounts.liquidity_pool.token_a_reserves as u128)
        .ok_or(MathError::ZeroDivisionError)?
        .try_into()
        .map_err(|_| MathError::OverflowError)?;
    ctx.accounts
        .mint_lp_tokens(lp_tokens_to_mint, ctx.bumps.lp_token_mint)?;
    ctx.accounts.liquidity_pool.token_a_reserves = ctx
        .accounts
        .liquidity_pool
        .token_a_reserves
        .checked_add(
            token_a_deposit_amount
                .try_into()
                .map_err(|_| MathError::OverflowError)?,
        )
        .ok_or(MathError::OverflowError)?;
    ctx.accounts.liquidity_pool.token_b_reserves = ctx
        .accounts
        .liquidity_pool
        .token_b_reserves
        .checked_add(
            token_b_deposit_amount
                .try_into()
                .map_err(|_| MathError::OverflowError)?,
        )
        .ok_or(MathError::OverflowError)?;
    Ok(())
}

impl<'info> Deposit<'info> {
    pub fn optimize_deposit_amounts(
        &self,
        token_a_amount_desired: u128,
        token_b_amount_desired: u128,
        token_a_amount_min: u128,
        token_b_amount_min: u128,
    ) -> Result<(u128, u128)> {
        // let's say we want to use all of our token_amount_a_desired, so we have to see what the optimal is for token b
        let token_b_optimal_amount = quote(
            token_a_amount_desired,
            self.liquidity_pool.token_a_reserves as u128,
            self.liquidity_pool.token_b_reserves as u128,
        )?;
        // if the optimal amount is the same as we desired or more favorable
        if token_b_optimal_amount <= token_b_amount_desired {
            require!(
                token_b_optimal_amount >= token_b_amount_min,
                AMMError::SlippageLimitExceeded
            );
            return Ok((token_a_amount_desired, token_b_optimal_amount));
        } else {
            let token_a_optimal_amount = quote(
                token_b_amount_desired,
                self.liquidity_pool.token_b_reserves as u128,
                self.liquidity_pool.token_a_reserves as u128,
            )?;
            require!(
                token_a_optimal_amount >= token_a_amount_min,
                AMMError::SlippageLimitExceeded
            );
            return Ok((token_a_optimal_amount, token_b_amount_desired));
        }
    }
    pub fn mint_lp_tokens(&self, lp_tokens_to_mint: u64, lp_token_mint_bump: u8) -> Result<()> {
        // --
        // this is just so rust doesn't bother me with borrow rules
        let token_a_key = self.token_a_mint.key();
        let token_b_key = self.token_b_mint.key();
        // --
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"lp_token_mint",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &[lp_token_mint_bump],
        ]];

        let mint_to_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.lp_token_mint.to_account_info(),
                to: self.lp_token_signer_token_account.to_account_info(),
                authority: self.lp_token_mint.to_account_info(),
            },
            signer_seeds,
        );
        mint_to(mint_to_ctx, lp_tokens_to_mint)
    }
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
