use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{LiquidityPool, LIQUIDITY_POOL_SEED};

pub trait VaultWithdrawer<'info> {
    fn token_program(&self) -> &Program<'info, Token>;
    fn token_a_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn token_b_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn token_a_mint(&self) -> &Account<'info, Mint>;
    fn token_b_mint(&self) -> &Account<'info, Mint>;
    fn token_a_vault(&self) -> &Account<'info, TokenAccount>;
    fn token_b_vault(&self) -> &Account<'info, TokenAccount>;
    fn liquidity_pool(&self) -> &Account<'info, LiquidityPool>;

    fn withdraw(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        if token_a_amount > 0 {
            self.withdraw_token(
                &self.token_a_mint(),
                &self.token_a_vault(),
                &self.token_a_signer_token_account(),
                token_a_amount,
            )?;
        }
        if token_b_amount > 0 {
            self.withdraw_token(
                &self.token_b_mint(),
                &self.token_b_vault(),
                &self.token_b_signer_token_account(),
                token_b_amount,
            )?;
        }
        Ok(())
    }

    fn withdraw_token(
        &self,
        mint: &Account<'info, Mint>,
        vault: &Account<'info, TokenAccount>,
        destination: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let token_a_key = self.token_a_mint().key();
        let token_b_key = self.token_b_mint().key();
        let bump = self.liquidity_pool().bump;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED.as_bytes(),
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &[bump],
        ]];

        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program().to_account_info(),
            TransferChecked {
                from: vault.to_account_info(),
                to: destination.to_account_info(),
                mint: mint.to_account_info(),
                authority: self.liquidity_pool().to_account_info(),
            },
            signer_seeds,
        );

        transfer_checked(transfer_ctx, amount, mint.decimals)
    }
}
