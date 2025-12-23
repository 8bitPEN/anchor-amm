use anchor_lang::prelude::*;

#[account(discriminator = 1)]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_reserves: u64, // be careful because the reserves are not normalized!
    pub token_b_reserves: u64,
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub precision: u8,
    pub bump: u8,
}
