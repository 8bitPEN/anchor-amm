use anchor_lang::prelude::*;

#[account(discriminator = 1)]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_reserves: u64, // be careful because the reserves are not normalized!
    pub token_b_reserves: u64,
    pub k_last: u128,
    pub bump: u8,
}
