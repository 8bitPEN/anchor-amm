#![allow(ambiguous_glob_reexports)]
// ^ this is so that I can use instruction handlers
// with the "deposit::handler" or "intitialize_pool::handler" format.
// it shouldn't cause any issues because I'm always fully qualifying it.
pub mod deposit;
pub mod initialize_pool;
pub use deposit::*;
pub use initialize_pool::*;
