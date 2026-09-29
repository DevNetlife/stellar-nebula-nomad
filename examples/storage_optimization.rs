//! # Example: Gas-efficient storage patterns
//!
//! Module: `storage_optim` (library helpers used inside contract functions)
//!
//! Shows:
//!   * TTL-bumped writes (`store_with_bump`),
//!   * batched writes and reads (`batch_store_with_bump`, `get_optimized_entries`),
//!   * composite keys (`store_ship_nebula`, `get_ship_nebula_batch`),
//!   * `CachedEntry`: read once, update in memory, write once.
//!
//! Each section prints the Soroban CPU instruction cost, so you can see the
//! savings directly.
//!
//! These helpers touch contract storage, so they must run *inside* a
//! contract. Here we use `env.as_contract(...)`; in production you call them
//! from your own `#[contractimpl]` functions.
//!
//! Run:
//! ```text
//! cargo run --example storage_optimization
//! ```

use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, BytesN, Env, Symbol};
use stellar_nebula_nomad::{
    batch_store_with_bump, get_bump_config, get_optimized_entries, get_optimized_entry,
    get_ship_nebula_batch, initialize_bump_config, reset_burst_counter, store_ship_nebula,
    store_with_bump, CachedEntry, NebulaNomadContract, StorageError, StorageTier,
};

/// Run `f` and return the CPU instructions it consumed.
fn cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    let budget = env.cost_estimate().budget();
    budget.reset_unlimited();
    f();
    budget.cpu_instruction_cost()
}

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let host = env.register(NebulaNomadContract, ());
    let admin = Address::generate(&env);

    env.as_contract(&host, || {
        // ── Step 1: Configure default TTLs ───────────────────────────────
        initialize_bump_config(&env, &admin);
        let cfg = get_bump_config(&env);
        println!(
            "Bump config: default_ttl={} max_ttl={}",
            cfg.default_ttl, cfg.max_ttl
        );

        // ── Step 2: Single TTL-bumped write ──────────────────────────────
        // Writes the value and extends its TTL in one call, so it will not
        // be archived before the next write.
        let payload = BytesN::from_array(&env, &[42u8; 64]);
        let res = store_with_bump(&env, symbol_short!("config"), payload.clone()).unwrap();
        println!("Stored {:?} with ttl={}", res.key, res.ttl_applied);

        // ── Step 3: Batched writes ───────────────────────────────────────
        // One re-entrancy guard, one config read and one timestamp for the
        // whole batch. keys and values must have equal length.
        let keys = vec![
            &env,
            symbol_short!("a"),
            symbol_short!("b"),
            symbol_short!("c"),
        ];
        let values = vec![&env, payload.clone(), payload.clone(), payload.clone()];
        let stored = batch_store_with_bump(&env, keys.clone(), values).unwrap();
        println!("Batch stored {} entries", stored.len());

        let mismatched = batch_store_with_bump(&env, keys.clone(), vec![&env, payload.clone()]);
        assert!(matches!(mismatched, Err(StorageError::InvalidKey)));

        // ── Step 4: Single vs. batched reads ─────────────────────────────
        // Each single read also updates the burst-read counter. A batch read
        // updates the counter once for all keys.
        reset_burst_counter(&env);
        let single = cpu(&env, || {
            for k in keys.iter() {
                get_optimized_entry(&env, k).unwrap();
            }
        });
        reset_burst_counter(&env);
        let batched = cpu(&env, || {
            get_optimized_entries(&env, keys.clone()).unwrap();
        });
        println!("Read 3 entries: single={single} cpu, batched={batched} cpu");

        // ── Step 5: Composite keys ───────────────────────────────────────
        // All per-(ship, nebula) data sits in one slot instead of several.
        for nebula_id in 1u64..=3 {
            store_ship_nebula(&env, 7, nebula_id, 5, 1_000 * nebula_id).unwrap();
        }
        reset_burst_counter(&env);
        let records = get_ship_nebula_batch(&env, 7, vec![&env, 1u64, 2, 3]).unwrap();
        for r in records.iter() {
            println!(
                "  ship {} nebula {}: scans={} cache={}",
                r.ship_id, r.nebula_id, r.scan_count, r.resource_cache
            );
        }

        // ── Step 6: CachedEntry (read-through / write-back) ──────────────
        // A common anti-pattern re-reads and re-writes a counter on every
        // step. CachedEntry reads once and writes once on `flush`.
        let counter_key: Symbol = symbol_short!("visits");
        env.storage().instance().set(&counter_key, &0u64);

        let naive = cpu(&env, || {
            for _ in 0..10 {
                let v: u64 = env.storage().instance().get(&counter_key).unwrap_or(0);
                env.storage().instance().set(&counter_key, &(v + 1));
            }
        });

        let cached = cpu(&env, || {
            let mut visits =
                CachedEntry::<Symbol, u64>::new(StorageTier::Instance, counter_key.clone());
            for _ in 0..10 {
                let v = visits.get_or(&env, 0);
                visits.set(v + 1);
            }
            visits.flush(&env);
        });

        let final_count: u64 = env.storage().instance().get(&counter_key).unwrap();
        println!(
            "10 increments: naive={naive} cpu, cached={cached} cpu (final value {final_count})"
        );
    });
}
