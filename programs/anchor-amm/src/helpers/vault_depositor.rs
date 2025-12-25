use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

/// Trait for depositing tokens into AMM pool vaults.
///
/// Implement this trait on any Anchor accounts struct that needs to transfer
/// tokens from a user's token accounts into the pool's liquidity vaults.
pub trait VaultDepositor<'info> {
    fn token_program(&self) -> &Program<'info, Token>;
    fn token_a_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn token_b_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn token_a_mint(&self) -> &Account<'info, Mint>;
    fn token_b_mint(&self) -> &Account<'info, Mint>;
    fn token_a_vault(&self) -> &Account<'info, TokenAccount>;
    fn token_b_vault(&self) -> &Account<'info, TokenAccount>;
    fn signer(&self) -> &Signer<'info>;

    /// Transfers liquidity from the signer's token accounts to the pool vaults.
    ///
    /// Performs two CPI calls to transfer tokens:
    /// - Token A from `token_a_signer_token_account` → `token_a_vault`
    /// - Token B from `token_b_signer_token_account` → `token_b_vault`
    ///
    /// # Arguments
    /// * `token_a_amount` - Amount of token A to deposit (in token A's native decimals)
    /// * `token_b_amount` - Amount of token B to deposit (in token B's native decimals)
    ///
    /// # Errors
    /// Returns an error if either transfer CPI fails (e.g., insufficient balance).
    fn deposit(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        let token_a_transfer_ctx = CpiContext::new(
            self.token_program().to_account_info(),
            TransferChecked {
                from: self.token_a_signer_token_account().to_account_info(),
                mint: self.token_a_mint().to_account_info(),
                to: self.token_a_vault().to_account_info(),
                authority: self.signer().to_account_info(),
            },
        );
        let token_b_transfer_ctx = CpiContext::new(
            self.token_program().to_account_info(),
            TransferChecked {
                from: self.token_b_signer_token_account().to_account_info(),
                mint: self.token_b_mint().to_account_info(),
                to: self.token_b_vault().to_account_info(),
                authority: self.signer().to_account_info(),
            },
        );
        transfer_checked(
            token_a_transfer_ctx,
            token_a_amount,
            self.token_a_mint().decimals,
        )?;
        transfer_checked(
            token_b_transfer_ctx,
            token_b_amount,
            self.token_b_mint().decimals,
        )?;
        Ok(())
    }
}
