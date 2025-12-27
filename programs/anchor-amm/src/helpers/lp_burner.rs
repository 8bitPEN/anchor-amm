use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};

pub trait LPBurner<'info> {
    fn token_program(&self) -> &Program<'info, Token>;
    fn lp_token_mint(&self) -> &Account<'info, Mint>;
    fn lp_token_signer_token_account(&self) -> &Account<'info, TokenAccount>;
    fn signer(&self) -> &Signer<'info>;
    fn burn_lp_tokens(&self, lp_tokens_to_burn: u64) -> Result<()> {
        let burn_ctx = CpiContext::new(
            self.token_program().to_account_info(),
            Burn {
                mint: self.lp_token_mint().to_account_info(),
                authority: self.lp_token_signer_token_account().to_account_info(),
                from: self.signer().to_account_info(),
            },
        );
        burn(burn_ctx, lp_tokens_to_burn)?;
        Ok(())
    }
}
