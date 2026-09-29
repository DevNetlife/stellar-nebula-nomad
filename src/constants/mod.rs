//! Centralized constants module for Stellar Nebula Nomad.
//! Exposes game mechanics, economic, limits, and timing constants.

pub mod economic;
pub mod game;
pub mod limits;
pub mod timing;

pub use economic::*;
pub use game::*;
pub use limits::*;
pub use timing::*;

use soroban_sdk::{symbol_short, Env, Symbol};

/// Governance key symbols for storage-based configurable constants.
pub struct ConfigKeys;

impl ConfigKeys {
    pub fn craft_fee(_env: &Env) -> Symbol {
        symbol_short!("CRFT_FEE")
    }
    pub fn max_price_change(_env: &Env) -> Symbol {
        symbol_short!("MAX_PCHG")
    }
}

/// Helper functions to fetch configurable constants from contract state with fallback to module defaults.
pub fn get_crafting_fee_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&ConfigKeys::craft_fee(env))
        .unwrap_or(BASE_CRAFTING_FEE_BPS)
}

pub fn set_crafting_fee_bps(env: &Env, new_fee: u32) {
    env.storage()
        .instance()
        .set(&ConfigKeys::craft_fee(env), &new_fee);
}
