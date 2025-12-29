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
    #[instruction(discriminator = 1)]
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        initialize_pool::handler(ctx)
    }
    #[instruction(discriminator = 2)]
    pub fn deposit(
        ctx: Context<Deposit>,
        token_a_amount_desired: u64,
        token_b_amount_desired: u64,
        token_a_amount_min: u64,
        token_b_amount_min: u64,
        expiration: i64,
    ) -> Result<()> {
        deposit::handler(
            ctx,
            token_a_amount_desired,
            token_b_amount_desired,
            token_a_amount_min,
            token_b_amount_min,
            expiration,
        )
    }
    #[instruction(discriminator = 3)]
    pub fn swap(
        ctx: Context<Swap>,
        token_0_amount: u64,
        token_1_min_amount: u64,
        expiration: i64,
    ) -> Result<()> {
        swap::handler(ctx, token_0_amount, token_1_min_amount, expiration)
    }
    #[instruction(discriminator = 4)]
    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_amount_to_burn: u64,
        amount_a_min: u64,
        amount_b_min: u64,
        expiration: i64,
    ) -> Result<()> {
        withdraw::handler(
            ctx,
            lp_amount_to_burn,
            amount_a_min,
            amount_b_min,
            expiration,
        )
    }
    #[instruction(discriminator = 5)]
    pub fn sync(ctx: Context<SyncReserves>) -> Result<()> {
        sync_reserves::handler(ctx)
    }
    #[instruction(discriminator = 6)]
    pub fn skim(ctx: Context<SkimReserves>) -> Result<()> {
        skim_reserves::handler(ctx)
    }
}
