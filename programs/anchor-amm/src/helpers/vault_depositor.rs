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
    /// - Token 0 from `token_a_signer_token_account` → `token_a_vault`
    /// - Token 1 from `token_b_signer_token_account` → `token_b_vault`
    ///
    /// # Arguments
    /// * `token_a_amount` - Amount of token 0 to deposit (in token 0's native decimals)
    /// * `token_b_amount` - Amount of token 1 to deposit (in token 1's native decimals)
    ///
    /// # Errors
    /// Returns an error if either transfer CPI fails (e.g., insufficient balance).
    fn deposit(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        if token_a_amount > 0 {
            self.deposit_token(
                &self.token_a_mint(),
                &self.token_a_signer_token_account(),
                &self.token_a_vault(),
                token_a_amount,
            )?;
        }
        if token_b_amount > 0 {
            self.deposit_token(
                &self.token_b_mint(),
                &self.token_b_signer_token_account(),
                &self.token_b_vault(),
                token_b_amount,
            )?;
        }
        Ok(())
    }

    fn deposit_token(
        &self,
        mint: &Account<'info, Mint>,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let transfer_ctx = CpiContext::new(
            self.token_program().to_account_info(),
            TransferChecked {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: self.signer().to_account_info(),
            },
        );
        transfer_checked(transfer_ctx, amount, mint.decimals)
    }
}
