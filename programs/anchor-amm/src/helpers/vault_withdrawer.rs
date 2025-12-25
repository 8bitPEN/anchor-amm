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
        let token_a_key = self.token_a_mint().key();
        let token_b_key = self.token_b_mint().key();
        let bump = self.liquidity_pool().bump;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED.as_ref(),
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &[bump],
        ]];

        let token_a_transfer_ctx = CpiContext::new_with_signer(
            self.token_program().to_account_info(),
            TransferChecked {
                from: self.token_a_vault().to_account_info(),
                to: self.token_a_signer_token_account().to_account_info(),
                mint: self.token_a_mint().to_account_info(),
                authority: self.liquidity_pool().to_account_info(),
            },
            signer_seeds,
        );
        let token_b_transfer_ctx = CpiContext::new_with_signer(
            self.token_program().to_account_info(),
            TransferChecked {
                from: self.token_b_vault().to_account_info(),
                to: self.token_b_signer_token_account().to_account_info(),
                mint: self.token_b_mint().to_account_info(),
                authority: self.liquidity_pool().to_account_info(),
            },
            signer_seeds,
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
