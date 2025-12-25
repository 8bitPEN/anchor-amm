use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::LiquidityPool;

/// Trait for syncing pool reserves with actual vault balances.
///
/// This is modeled after Uniswap V2's `sync()` function. It allows anyone to
/// force the stored reserves to match the actual token balances in the vaults.
///
/// # Use Cases
/// - Recovery when tokens are sent directly to vaults (bypassing swap/deposit)
/// - Correcting reserve drift if accounting gets out of sync
/// - Arbitrage opportunities when reserves don't reflect actual balances
pub trait ReserveSyncer<'info> {
    fn liquidity_pool(&mut self) -> &mut Account<'info, LiquidityPool>;
    fn token_a_vault(&self) -> &Account<'info, TokenAccount>;
    fn token_b_vault(&self) -> &Account<'info, TokenAccount>;

    /// Syncs the pool's stored reserves with the actual vault token balances.
    ///
    /// Updates `token_a_reserves` and `token_b_reserves` in the liquidity pool
    /// to match the current `amount` held in each vault's token account.
    fn sync_reserves(&mut self) {
        self.liquidity_pool().token_a_reserves = self.token_a_vault().amount;
        self.liquidity_pool().token_b_reserves = self.token_b_vault().amount;
    }
}
