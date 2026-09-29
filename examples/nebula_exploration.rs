//! # Example: Nebula exploration (main contract)
//!
//! Module: `nebula_explorer` via `NebulaNomadContract`
//!
//! Shows the core exploration loop:
//!   1. generate a procedural 16x16 nebula from a 32-byte seed,
//!   2. score its rarity,
//!   3. do both in one call with `scan_nebula`.
//!
//! Run:
//! ```text
//! cargo run --example nebula_exploration
//! ```

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient};

fn main() {
    // ── Step 1: Local Soroban environment ────────────────────────────────
    // `Env::default()` is an in-memory ledger. `mock_all_auths()` makes every
    // `require_auth()` pass so we can focus on game logic. On a real network
    // the player's wallet signs instead.
    let env = Env::default();
    env.mock_all_auths();

    // ── Step 2: Deploy the contract and create a typed client ────────────
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);

    let player = Address::generate(&env);

    // ── Step 3: Pick a seed ──────────────────────────────────────────────
    // Any 32 bytes work. The contract mixes in the ledger sequence and
    // timestamp, so the same seed gives a different nebula on each ledger.
    // dApps typically use `crypto.getRandomValues`.
    let seed = BytesN::from_array(&env, &[7u8; 32]);

    // ── Step 4: Generate a layout ────────────────────────────────────────
    let layout = client.generate_nebula_layout(&seed, &player);
    println!("Generated nebula:");
    println!("  size          : {}x{}", layout.width, layout.height);
    println!("  cells         : {}", layout.cells.len());
    println!("  total energy  : {}", layout.total_energy);
    println!("  generated at  : {}", layout.timestamp);

    // ── Step 5: Score its rarity ─────────────────────────────────────────
    // Read-only helper: rare cells * 10 + energy density -> tier.
    let rarity = client.calculate_rarity_tier(&layout);
    println!("  rarity tier   : {rarity:?}");

    // ── Step 6: One-shot scan (generate + score + event) ─────────────────
    // `scan_nebula` is what a game client normally calls. It also emits the
    // `NebulaScanned` event that indexers and the frontend listen for.
    let (scanned, scanned_rarity) = client.scan_nebula(&seed, &player);
    println!();
    println!("scan_nebula:");
    println!(
        "  energy={} rarity={scanned_rarity:?}",
        scanned.total_energy
    );

    // ── Step 7: Determinism ─────────────────────────────────────────────
    // Same seed + same ledger = same layout, so anyone can verify a scan.
    let again = client.generate_nebula_layout(&seed, &player);
    assert_eq!(again.total_energy, layout.total_energy);
    println!();
    println!("Determinism check passed: identical inputs -> identical layout.");
}
