//! # Example: Validated nebula generator with layout lifecycle
//!
//! Module: `nebula_gen` (`NebulaGen` contract)
//!
//! Shows:
//!   * one-time admin initialisation (size bounds and layout TTL),
//!   * validated generation (ship_id / region_id / seed checks),
//!   * querying anomalies from the active layout,
//!   * layout expiry and admin cleanup,
//!   * handling `NebulaError` codes with the `try_*` client.
//!
//! Run:
//! ```text
//! cargo run --example nebula_generator
//! ```

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Vec,
};
use stellar_nebula_nomad::nebula_gen::{NebulaError, NebulaGen, NebulaGenClient, MAX_REGION_ID};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    // ── Step 1: Deploy and initialise ────────────────────────────────────
    // default_size = 21 anomalies, bounds [1, 64], layouts live 600 s.
    let id = env.register(NebulaGen, ());
    let nebula = NebulaGenClient::new(&env, &id);
    let admin = Address::generate(&env);
    nebula.init(&admin, &21u32, &1u32, &64u32, &600u64);

    // Initialising twice is rejected (code 2).
    assert_eq!(
        nebula.try_init(&admin, &21u32, &1u32, &64u32, &600u64),
        Err(Ok(NebulaError::AlreadyInitialized))
    );

    // ── Step 2: Generate a layout for a ship ─────────────────────────────
    let pilot = Address::generate(&env);
    let ship_id = 42u64;
    let region_id = 7u64;
    let seed = BytesN::from_array(&env, &[0xA5; 32]);

    let layout = nebula.generate_validated_nebula_layout(&pilot, &ship_id, &region_id, &seed);
    println!("Layout for ship {ship_id} in region {region_id}:");
    println!("  anomalies : {}", layout.size);
    println!("  hash      : {:?}", layout.layout_hash.to_array());

    // ── Step 3: Inspect anomalies ────────────────────────────────────────
    for i in 0..3u32 {
        let a = nebula.query_anomaly(&ship_id, &i);
        println!(
            "  #{i}: ({:>3},{:>3}) rarity={:>3} {:?} / {:?}",
            a.x, a.y, a.rarity, a.anomaly_type, a.resource_class
        );
    }
    assert!(nebula.has_anomaly(&ship_id, &0u32));

    // ── Step 4: Input validation errors ──────────────────────────────────
    // Each bad input maps to its own error code (see docs/ERROR_CODES.md).
    let zero_seed = BytesN::from_array(&env, &[0u8; 32]);
    let cases = [
        (
            "ship_id = 0",
            nebula.try_generate_validated_nebula_layout(&pilot, &0, &region_id, &seed),
        ),
        (
            "region_id > MAX",
            nebula.try_generate_validated_nebula_layout(
                &pilot,
                &ship_id,
                &(MAX_REGION_ID + 1),
                &seed,
            ),
        ),
        (
            "all-zero seed",
            nebula.try_generate_validated_nebula_layout(&pilot, &ship_id, &region_id, &zero_seed),
        ),
    ];
    println!();
    println!("Validation:");
    for (label, result) in cases {
        match result {
            Err(Ok(e)) => println!("  {label:<16} -> {e:?} (code {})", e as u32),
            other => println!("  {label:<16} -> unexpected {other:?}"),
        }
    }

    // Out-of-range anomaly index.
    assert_eq!(
        nebula.try_has_anomaly(&ship_id, &layout.size),
        Err(Ok(NebulaError::AnomalyOutOfBounds))
    );

    // ── Step 5: Expiry ───────────────────────────────────────────────────
    // After the TTL, reads treat the layout as gone (and remove it).
    env.ledger().set_timestamp(1_000 + 601);
    assert!(nebula.get_layout(&ship_id).is_none());
    assert_eq!(
        nebula.try_query_anomaly(&ship_id, &0u32),
        Err(Ok(NebulaError::LayoutNotFound))
    );
    println!();
    println!("Layout expired after TTL -> LayoutNotFound. Regenerate to continue.");

    // ── Step 6: Admin maintenance ────────────────────────────────────────
    // Generate a few layouts, let them expire, sweep them in one call.
    env.ledger().set_timestamp(2_000);
    let mut ships = Vec::new(&env);
    for s in 100u64..105 {
        nebula.generate_validated_nebula_layout(&pilot, &s, &region_id, &seed);
        ships.push_back(s);
    }
    env.ledger().set_timestamp(2_000 + 601);
    let removed = nebula.clean_expired_layouts(&ships);
    println!("Admin sweep removed {removed} expired layouts.");

    // Extend the TTL for future layouts (admin only, must be > 0).
    nebula.update_layout_ttl(&86_400u64);
    assert_eq!(
        nebula.try_update_layout_ttl(&0u64),
        Err(Ok(NebulaError::InvalidTtl))
    );
    println!("Layout TTL updated to 24h.");
}
