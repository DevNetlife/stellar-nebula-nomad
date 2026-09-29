//! Centralized test fixtures and helpers (#311).
//!
//! Import with:
//! ```ignore
//! use crate::common::*;
//! ```

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, BytesN, Env};
use stellar_nebula_nomad::NebulaNomadContractClient;

// ── Environment Setup ─────────────────────────────────────────────────────

/// Default ledger info with a reasonable starting state.
pub fn default_ledger() -> LedgerInfo {
    LedgerInfo {
        protocol_version: 22,
        sequence_number: 100,
        timestamp: 1_700_000_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    }
}

/// Creates a default env with mock auths and a standard ledger.
pub fn setup_env() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(default_ledger());
    let admin = Address::generate(&env);
    (env, admin)
}

/// Creates a full environment with a deployed contract and client.
pub fn setup_contract() -> (Env, NebulaNomadContractClient<'static>, Address) {
    let (env, admin) = setup_env();
    let contract_id = env.register(stellar_nebula_nomad::NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    // Safety: the client is created locally and used within the same test function.
    let client_static = unsafe { core::mem::transmute::<_, NebulaNomadContractClient<'static>>(client) };
    (env, client_static, admin)
}

// ── Ledger Helpers ─────────────────────────────────────────────────────────

/// Advance the ledger by `count` ledgers, each 5 seconds apart.
pub fn advance_ledger(env: &Env, count: u32) {
    let seq = env.ledger().sequence();
    env.ledger().set(LedgerInfo {
        sequence_number: seq + count,
        timestamp: env.ledger().timestamp() + (count as u64) * 5,
        ..default_ledger()
    });
}

/// Advance the ledger timestamp by `seconds` (no sequence change).
pub fn advance_time(env: &Env, seconds: u64) {
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + seconds,
        ..default_ledger()
    });
}

/// Set the ledger to an exact timestamp and sequence.
pub fn set_ledger(env: &Env, sequence: u32, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        sequence_number: sequence,
        timestamp,
        ..default_ledger()
    });
}

// ── Address Generation ─────────────────────────────────────────────────────

/// Generate N unique addresses.
pub fn generate_addresses(env: &Env, n: usize) -> Vec<Address> {
    (0..n).map(|_| Address::generate(env)).collect()
}

// ── Bytes Helpers ──────────────────────────────────────────────────────────

/// Create a fixed 32-byte hash from a seed byte.
pub fn test_hash(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [seed; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}
