pub mod constants;
pub mod error;
mod helpers;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7Zk1fV2VY517YbBFtWV76K58msXYcBjGoqsTauox1GcQ");

#[program]
pub mod anchor_amm {

    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        initialize_pool::handler(ctx)
    }
    pub fn deposit(
        ctx: Context<Deposit>,
        token_a_amount_desired: u64,
        token_b_amount_desired: u64,
        token_a_amount_min: u64,
        token_b_amount_min: u64,
    ) -> Result<()> {
        deposit::handler(
            ctx,
            token_a_amount_desired,
            token_b_amount_desired,
            token_a_amount_min,
            token_b_amount_min,
        )
    }
    pub fn sync(ctx: Context<SyncReserves>) -> Result<()> {
        sync_reserves::handler(ctx)
    }
    pub fn skim(ctx: Context<SkimReserves>) -> Result<()> {
        skim_reserves::handler(ctx)
    }
    //TODO (Pen): there could be a deadline here so trade can expire
    pub fn swap(ctx: Context<Swap>, token_a_amount: u64, token_b_min_amount: u64) -> Result<()> {
        swap::handler(ctx, token_a_amount, token_b_min_amount)
    }
}
