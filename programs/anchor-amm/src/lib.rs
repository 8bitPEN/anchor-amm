pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7Zk1fV2VY517YbBFtWV76K58msXYcBjGoqsTauox1GcQ");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        token_a_amount: u64,
        token_b_amount: u64,
        fee_bps: u64,
    ) -> Result<()> {
        initialize_pool::handler(ctx, token_a_amount, token_b_amount, fee_bps)
    }
}
