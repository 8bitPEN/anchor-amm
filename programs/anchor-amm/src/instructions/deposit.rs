use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Deposit {
    
}

pub fn handler(ctx: Context<Deposit>, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
    Ok(())
}
