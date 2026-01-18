use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    error::{AmmError, MathError},
    helpers::{
        calculate_constant_product, quote, LPMinter, ProtocolFeeMinter, ReserveSyncer,
        VaultDepositor,
    },
    LiquidityPool, LIQUIDITY_POOL_SEED,
};

// TODO (Pen): Should there be deposit fees? Not gonna bother with fees for now.
// TODO (Pen): Make the precision have a bigger upper limit (19).
// TODO (Pen): Think about wrapped SOL
// TODO (Pen): Price oracle
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
    pub lp_token_signer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_a_signer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub token_b_signer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_a_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_pool
    )]
    pub token_b_vault: Box<Account<'info, TokenAccount>>,
    pub token_a_mint: Box<Account<'info, Mint>>,
    pub token_b_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp_token_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED.as_bytes(), token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    /// Protocol fee LP token account owned by the pool PDA
    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program
    )]
    pub fee_lp_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = system_program,
        associated_token::token_program = token_program
    )]
    // this is a dead address, sending here won't reduce the supply, but still effectively burn tokens
    pub lp_token_system_program_token_account: Box<Account<'info, TokenAccount>>,
}

pub fn handler(
    ctx: Context<Deposit>,
    token_a_amount_desired: u64,
    token_b_amount_desired: u64,
    token_a_amount_min: u64,
    token_b_amount_min: u64,
    expiration: i64,
) -> Result<()> {
    require!(
        token_a_amount_desired > 0 && token_b_amount_desired > 0,
        AmmError::ZeroAmount
    );
    require_gt!(
        expiration,
        Clock::get()?.unix_timestamp,
        AmmError::DeadlineExceeded,
    );
    if ctx.accounts.lp_token_mint.supply == 0 {
        ctx.accounts
            .deposit(token_a_amount_desired, token_b_amount_desired)?;

        // Reload vaults and sync reserves
        ctx.accounts.token_a_vault.reload()?;
        ctx.accounts.token_b_vault.reload()?;
        ctx.accounts.sync_reserves();
        ctx.accounts.liquidity_pool.k_last = (ctx.accounts.liquidity_pool.token_a_reserves as u128)
            .checked_mul(ctx.accounts.liquidity_pool.token_b_reserves as u128)
            .ok_or(MathError::Overflow)?;
        let lp_tokens_to_mint: u64 = calculate_constant_product(
            token_a_amount_desired as u128,
            token_b_amount_desired as u128,
        )?
        .isqrt()
        .try_into()
        .map_err(|_| MathError::Overflow)?;
        require_gt!(
            lp_tokens_to_mint,
            1000,
            AmmError::InsufficientInitialLiquidity
        );
        ctx.accounts.mint_lp_tokens(
            &ctx.accounts.lp_token_system_program_token_account,
            1000,
            ctx.bumps.lp_token_mint,
        )?;
        ctx.accounts.mint_lp_tokens(
            &ctx.accounts.lp_token_signer_token_account,
            lp_tokens_to_mint - 1000,
            ctx.bumps.lp_token_mint,
        )?;
        return Ok(());
    }
    let token_a_amount_desired = token_a_amount_desired as u128;
    let token_b_amount_desired = token_b_amount_desired as u128;
    let token_a_amount_min = token_a_amount_min as u128;
    let token_b_amount_min = token_b_amount_min as u128;

    // Mint protocol fees before adding liquidity
    ctx.accounts.mint_protocol_fee(ctx.bumps.lp_token_mint)?;
    ctx.accounts.lp_token_mint.reload()?;
    let (token_a_deposit_amount, token_b_deposit_amount) = ctx.accounts.optimize_deposit_amounts(
        token_a_amount_desired,
        token_b_amount_desired,
        token_a_amount_min,
        token_b_amount_min,
    )?;
    ctx.accounts.deposit(
        token_a_deposit_amount
            .try_into()
            .map_err(|_| MathError::Overflow)?,
        token_b_deposit_amount
            .try_into()
            .map_err(|_| MathError::Overflow)?,
    )?;
    //NOTE in Uniswap, they decide how many tokens to mint based on the minimum ratio, but since we have already optimized the deposits,
    //NOTE it's safe to just go with token a, since it should be equal to token_b.
    let lp_tokens_to_mint: u64 = token_a_deposit_amount
        .checked_mul(ctx.accounts.lp_token_mint.supply as u128)
        .ok_or(MathError::Overflow)?
        .checked_div(ctx.accounts.liquidity_pool.token_a_reserves as u128)
        .ok_or(MathError::DivisionByZero)?
        .try_into()
        .map_err(|_| MathError::Overflow)?;
    ctx.accounts.mint_lp_tokens(
        &ctx.accounts.lp_token_signer_token_account,
        lp_tokens_to_mint,
        ctx.bumps.lp_token_mint,
    )?;

    // Reload vaults and sync reserves
    ctx.accounts.token_a_vault.reload()?;
    ctx.accounts.token_b_vault.reload()?;
    ctx.accounts.sync_reserves();
    // Update k_last for protocol fee tracking
    ctx.accounts.liquidity_pool.k_last = (ctx.accounts.liquidity_pool.token_a_reserves as u128)
        .checked_mul(ctx.accounts.liquidity_pool.token_b_reserves as u128)
        .ok_or(MathError::Overflow)?;

    Ok(())
}

impl<'info> Deposit<'info> {
    /// Calculates optimal deposit amounts that maintain the pool's current ratio.
    ///
    /// Since AMM pools require deposits in the exact ratio of existing reserves,
    /// this function adjusts the user's desired amounts to match the pool ratio
    /// while maximizing the deposit within slippage constraints.
    ///
    /// # Algorithm
    /// 1. First tries to use all of `token_a_amount_desired` and calculates the
    ///    corresponding optimal token B amount based on current reserves
    /// 2. If optimal B ≤ desired B: uses (desired A, optimal B)
    /// 3. Otherwise: flips the calculation — uses all of `token_b_amount_desired`
    ///    and calculates the optimal token A amount
    ///
    /// # Arguments
    /// * `token_a_amount_desired` - Maximum amount of token A the user wants to deposit
    /// * `token_b_amount_desired` - Maximum amount of token B the user wants to deposit
    /// * `token_a_amount_min` - Minimum acceptable token A deposit (slippage protection)
    /// * `token_b_amount_min` - Minimum acceptable token B deposit (slippage protection)
    ///
    /// # Returns
    /// A tuple `(token_a_amount, token_b_amount)` representing the optimized deposit amounts.
    ///
    /// # Errors
    /// Returns `AmmError::SlippageExceeded` if the optimal amounts fall below minimums.
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
                AmmError::SlippageExceeded
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
                AmmError::SlippageExceeded
            );
            return Ok((token_a_optimal_amount, token_b_amount_desired));
        }
    }
}
impl<'info> LPMinter<'info> for Deposit<'info> {
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
}
impl<'info> VaultDepositor<'info> for Deposit<'info> {
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
impl<'info> ReserveSyncer<'info> for Deposit<'info> {
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

impl<'info> ProtocolFeeMinter<'info> for Deposit<'info> {
    fn fee_lp_token_account(&self) -> &Account<'info, TokenAccount> {
        &self.fee_lp_token_account
    }

    fn liquidity_pool(&self) -> &Account<'info, LiquidityPool> {
        &self.liquidity_pool
    }
}
