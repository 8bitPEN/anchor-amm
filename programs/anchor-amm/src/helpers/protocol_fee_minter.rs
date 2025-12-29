use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::{error::MathError, LiquidityPool};

use super::LPMinter;

/// Trait for minting protocol fees as LP tokens before liquidity events.
///
/// Implements the Uniswap V2 protocol fee mechanism where 1/6 of swap fees
/// are minted as LP tokens to the protocol's fee token account (owned by the pool PDA).
///
/// The fee is calculated by comparing current k (reserve0 * reserve1) with k_last
/// (the k value at the last liquidity event). Any growth in sqrt(k) indicates
/// accumulated swap fees.
///
/// This trait composes with `LPMinter` to handle the actual token minting.
pub trait ProtocolFeeMinter<'info>: LPMinter<'info> {
    fn fee_lp_token_account(&self) -> &Account<'info, TokenAccount>;
    fn liquidity_pool(&self) -> &Account<'info, LiquidityPool>;

    /// Mints protocol fee LP tokens if there has been fee accumulation since k_last.
    ///
    /// # Algorithm (from Uniswap V2)
    /// 1. Calculate rootK = sqrt(reserve0 * reserve1)
    /// 2. Calculate rootKLast = sqrt(k_last)
    /// 3. If rootK > rootKLast (fees accumulated):
    ///    - numerator = totalSupply * (rootK - rootKLast)
    ///    - denominator = rootK * 5 + rootKLast
    ///    - liquidity = numerator / denominator
    ///    - Mint `liquidity` LP tokens to fee account
    ///
    /// The formula gives protocol 1/6 of the swap fees (the 5 creates this ratio).
    ///
    /// # Arguments
    /// * `lp_token_mint_bump` - The PDA bump seed for the LP token mint
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if minting fails
    fn mint_protocol_fee(&self, lp_token_mint_bump: u8) -> Result<()> {
        let k_last = self.liquidity_pool().k_last;

        // If k_last is 0, this is either first deposit or fees are disabled
        if k_last == 0 {
            return Ok(());
        }

        let reserve_a = self.liquidity_pool().token_a_reserves as u128;
        let reserve_b = self.liquidity_pool().token_b_reserves as u128;

        let k = reserve_a
            .checked_mul(reserve_b)
            .ok_or(MathError::Overflow)?;
        let root_k = k.isqrt();
        let root_k_last = k_last.isqrt();

        // Only mint if k has grown (fees accumulated from swaps)
        if root_k > root_k_last {
            let total_supply = self.lp_token_mint().supply as u128;

            // numerator = totalSupply * (rootK - rootKLast)
            let numerator = total_supply
                .checked_mul(root_k.checked_sub(root_k_last).ok_or(MathError::Overflow)?)
                .ok_or(MathError::Overflow)?;

            // denominator = rootK * 5 + rootKLast
            let denominator = root_k
                .checked_mul(5)
                .ok_or(MathError::Overflow)?
                .checked_add(root_k_last)
                .ok_or(MathError::Overflow)?;

            // liquidity = numerator / denominator
            let liquidity = numerator
                .checked_div(denominator)
                .ok_or(MathError::DivisionByZero)?;

            if liquidity > 0 {
                let liquidity_u64: u64 = liquidity.try_into().map_err(|_| MathError::Overflow)?;

                // Use composed LPMinter to mint tokens to fee account
                self.mint_lp_tokens(
                    self.fee_lp_token_account(),
                    liquidity_u64,
                    lp_token_mint_bump,
                )?;
            }
        }

        Ok(())
    }
}
