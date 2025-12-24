use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

pub trait LPMinter<'info> {
    fn token_program(&self) -> &Program<'info, Token>;
    fn token_a_mint(&self) -> &Account<'info, Mint>;
    fn token_b_mint(&self) -> &Account<'info, Mint>;
    fn lp_token_mint(&self) -> &Account<'info, Mint>;
    fn lp_token_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn mint_lp_tokens(&self, lp_tokens_to_mint: u64, lp_token_mint_bump: u8) -> Result<()> {
        // --
        // this is just so rust doesn't bother me with borrow rules
        let token_a_key = self.token_a_mint().key();
        let token_b_key = self.token_b_mint().key();
        // --
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
                to: self.lp_token_signer_token_account().to_account_info(),
                authority: self.lp_token_mint().to_account_info(),
            },
            signer_seeds,
        );
        mint_to(mint_to_ctx, lp_tokens_to_mint)
    }
}
