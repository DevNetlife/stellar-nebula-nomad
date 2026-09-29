//! Before/after benchmarks for Issue #437 (storage access) and
//! Issue #438 (nebula generation).
//!
//! Run with:
//!
//! ```text
//! cargo bench --bench nebula_storage_benchmarks
//! ```
//!
//! Every comparison runs both implementations in the same `Env` with the
//! same inputs and reports Soroban budget metrics (CPU instructions and
//! memory bytes). These are the numbers the network uses to charge fees, so
//! they stand in for gas.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, BytesN, Env, Symbol, Vec,
};
use stellar_nebula_nomad::nebula_gen::{NebulaGen, NebulaGenClient};
use stellar_nebula_nomad::{
    get_optimized_entries, get_optimized_entry, initialize_bump_config, reset_burst_counter,
    store_with_bump, Anomaly, AnomalyType, CachedEntry, ResourceClass, StorageTier,
};

// ─── Legacy nebula generator (pre-#438 hot path) ─────────────────────────

#[contracttype]
#[derive(Clone)]
enum LegacyKey {
    Layout(u64),
}

#[contract]
struct LegacyNebulaGen;

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn derive(seed: u64, index: u32, salt: u64) -> u64 {
    splitmix64(seed ^ splitmix64(u64::from(index).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt))
}

fn read_u64(seed: &BytesN<32>, offset: u32) -> u64 {
    let mut val = 0u64;
    for i in 0..8u32 {
        val |= u64::from(seed.get(offset + i).unwrap_or(0)) << (i * 8);
    }
    val
}

fn anomaly_type(v: u64) -> AnomalyType {
    match v % 5 {
        0 => AnomalyType::DustCloud,
        1 => AnomalyType::IonStorm,
        2 => AnomalyType::CrystalFormation,
        3 => AnomalyType::PlasmaVent,
        _ => AnomalyType::DarkMatterPocket,
    }
}

fn class(rarity: u64) -> ResourceClass {
    if rarity <= 33 {
        ResourceClass::Sparse
    } else if rarity <= 66 {
        ResourceClass::Moderate
    } else {
        ResourceClass::Abundant
    }
}

#[contractimpl]
impl LegacyNebulaGen {
    /// Mirrors the old `generate_validated_nebula_layout` hot path:
    /// 32 per-byte seed reads, one `push_back` per anomaly, a byte-by-byte
    /// hash, and storage keys rebuilt for each call.
    pub fn generate(env: Env, ship_id: u64, region_id: u64, seed: BytesN<32>, size: u32) -> u32 {
        let seed_u64 =
            read_u64(&seed, 0) ^ read_u64(&seed, 8) ^ read_u64(&seed, 16) ^ read_u64(&seed, 24);
        let master = splitmix64(seed_u64)
            ^ splitmix64(u64::from(env.ledger().sequence()))
            ^ splitmix64(env.ledger().timestamp())
            ^ splitmix64(ship_id)
            ^ splitmix64(region_id);

        let mut anomalies: Vec<Anomaly> = Vec::new(&env);
        for i in 0..size {
            let rarity = derive(master, i, 0xbf58_476d_1ce4_e5b9) % 101;
            anomalies.push_back(Anomaly {
                x: derive(master, i, 0x9e37_79b9_7f4a_7c15) % 1000,
                y: derive(master, i, 0x6c62_272e_07bb_0142) % 1000,
                rarity,
                anomaly_type: anomaly_type(derive(master, i, 0x94d0_49bb_1331_11eb)),
                resource_class: class(rarity),
            });
        }

        let mut arr = [0u8; 32];
        let lanes = [
            splitmix64(master),
            splitmix64(master ^ 0xdead_cafe_1234_5678),
            splitmix64(master.wrapping_add(0x1234_5678_dead_beef)),
            splitmix64(master.wrapping_mul(0x0101_0101_0101_0101).wrapping_add(1)),
        ];
        for (p, lane) in lanes.iter().enumerate() {
            arr[p * 8..p * 8 + 8].copy_from_slice(&lane.to_le_bytes());
        }
        let _hash = BytesN::from_array(&env, &arr);

        env.storage()
            .persistent()
            .set(&LegacyKey::Layout(ship_id), &anomalies);
        env.storage()
            .persistent()
            .extend_ttl(&LegacyKey::Layout(ship_id), 20, 20);
        anomalies.len()
    }
}

// ─── Harness ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Cost {
    cpu: u64,
    mem: u64,
}

fn measure<F: FnOnce()>(env: &Env, f: F) -> Cost {
    let budget = env.cost_estimate().budget();
    budget.reset_unlimited();
    f();
    Cost {
        cpu: budget.cpu_instruction_cost(),
        mem: budget.memory_bytes_cost(),
    }
}

fn pct_saved(before: u64, after: u64) -> f64 {
    if before == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let saved = (before as f64 - after as f64) / before as f64 * 100.0;
    saved
}

fn report(name: &str, before: Cost, after: Cost) {
    println!(
        "{name:<42} cpu {:>10} -> {:>10} ({:>6.2}% saved) | mem {:>9} -> {:>9} ({:>6.2}% saved)",
        before.cpu,
        after.cpu,
        pct_saved(before.cpu, after.cpu),
        before.mem,
        after.mem,
        pct_saved(before.mem, after.mem),
    );
}

fn fresh_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 10,
        timestamp: 1_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 1_000_000,
    });
    env
}

// ─── Issue #438: nebula generation ────────────────────────────────────────

fn bench_nebula_generation() {
    println!("\n== Issue #438: nebula generation (legacy vs optimised) ==");
    for size in [8u32, 16, 32, 64] {
        let env = fresh_env();
        let admin = Address::generate(&env);
        let seed = BytesN::from_array(&env, &[0x5Au8; 32]);

        let new_id = env.register(NebulaGen, ());
        let new_client = NebulaGenClient::new(&env, &new_id);
        new_client.init(&admin, &size, &1u32, &64u32, &100u64);

        let legacy_id = env.register(LegacyNebulaGen, ());
        let legacy_client = LegacyNebulaGenClient::new(&env, &legacy_id);

        let before = measure(&env, || {
            legacy_client.generate(&1u64, &1u64, &seed, &size);
        });
        let after = measure(&env, || {
            new_client.generate_validated_nebula_layout(&admin, &1u64, &1u64, &seed);
        });
        report(&format!("generate layout (size={size})"), before, after);
        assert!(
            after.cpu <= before.cpu,
            "optimised generation regressed at size {size}"
        );
    }
}

// ─── Issue #437: storage access ───────────────────────────────────────────

fn bench_batch_reads() {
    println!("\n== Issue #437: storage reads ==");
    let env = fresh_env();
    let host = env.register(NebulaGen, ());
    let admin = Address::generate(&env);

    let keys: [Symbol; 8] = [
        symbol_short!("k0"),
        symbol_short!("k1"),
        symbol_short!("k2"),
        symbol_short!("k3"),
        symbol_short!("k4"),
        symbol_short!("k5"),
        symbol_short!("k6"),
        symbol_short!("k7"),
    ];

    env.as_contract(&host, || {
        initialize_bump_config(&env, &admin);
        for k in &keys {
            store_with_bump(&env, k.clone(), BytesN::from_array(&env, &[1u8; 64])).unwrap();
        }
    });

    // N single reads: one burst-counter read + write per entry.
    let before = measure(&env, || {
        env.as_contract(&host, || {
            reset_burst_counter(&env);
            for k in &keys {
                get_optimized_entry(&env, k.clone()).unwrap();
            }
        });
    });

    // One batch read: a single burst-counter read + write for all entries.
    let after = measure(&env, || {
        env.as_contract(&host, || {
            reset_burst_counter(&env);
            get_optimized_entries(&env, Vec::from_array(&env, keys.clone())).unwrap();
        });
    });
    report("read 8 entries (single vs batch)", before, after);
}

fn bench_cached_entry() {
    let env = fresh_env();
    let host = env.register(NebulaGen, ());
    let key = symbol_short!("counter");

    env.as_contract(&host, || env.storage().instance().set(&key, &0u64));

    // Naive pattern: read-modify-write on every step.
    let before = measure(&env, || {
        env.as_contract(&host, || {
            for _ in 0..10 {
                let v: u64 = env.storage().instance().get(&key).unwrap_or(0);
                env.storage().instance().set(&key, &(v + 1));
            }
        });
    });

    // CachedEntry: one read, in-memory updates, one write on flush.
    let after = measure(&env, || {
        env.as_contract(&host, || {
            let mut entry = CachedEntry::<Symbol, u64>::new(StorageTier::Instance, key.clone());
            for _ in 0..10 {
                let v = entry.get_or(&env, 0);
                entry.set(v + 1);
            }
            entry.flush(&env);
        });
    });
    report("10x read-modify-write (naive vs cached)", before, after);
    assert!(after.cpu < before.cpu, "CachedEntry should reduce CPU cost");
}

fn main() {
    bench_nebula_generation();
    bench_batch_reads();
    bench_cached_entry();
    println!();
}
