use anchor_lang::prelude::*;

#[account(discriminator = 1)]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_amount: u64, // token a and token b amounts are normalized to be on the same precision
    pub token_b_amount: u64,
    pub constant_product: u64,
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub precision: u8,
    pub bump: u8,
}
