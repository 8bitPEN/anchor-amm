use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

/// Trait for minting LP (Liquidity Provider) tokens in an AMM pool.
///
/// Implement this trait on any Anchor accounts struct that needs to mint LP tokens
/// to a user after they provide liquidity. The LP token mint acts as its own authority
/// via PDA signing.
pub trait LPMinter<'info> {
    fn token_program(&self) -> &Program<'info, Token>;
    fn token_a_mint(&self) -> &Account<'info, Mint>;
    fn token_b_mint(&self) -> &Account<'info, Mint>;
    fn lp_token_mint(&self) -> &Account<'info, Mint>;

    /// Mints LP tokens to the specified token account.
    ///
    /// # Arguments
    /// * `mint_to_account` - The destination token account to receive minted LP tokens
    /// * `lp_tokens_to_mint` - The amount of LP tokens to mint
    /// * `lp_token_mint_bump` - The PDA bump seed for the LP token mint
    ///
    /// # PDA Seeds
    /// The LP token mint PDA is derived from: `["lp_token_mint", token_a_mint, token_b_mint]`
    fn mint_lp_tokens(
        &self,
        mint_to_account: &Account<'info, TokenAccount>,
        lp_tokens_to_mint: u64,
        lp_token_mint_bump: u8,
    ) -> Result<()> {
        // Extract keys upfront to satisfy borrow checker
        let token_a_key = self.token_a_mint().key();
        let token_b_key = self.token_b_mint().key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"lp_token_mint",
            token_a_key.as_ref(),
            token_b_key.as_ref(),
            &[lp_token_mint_bump],
        ]];

        let mint_to_ctx = CpiContext::new_with_signer(
            self.token_program().to_account_info(),
            MintTo {
                mint: self.lp_token_mint().to_account_info(),
                to: mint_to_account.to_account_info(),
                authority: self.lp_token_mint().to_account_info(),
            },
            signer_seeds,
        );

        mint_to(mint_to_ctx, lp_tokens_to_mint)
    }
}
