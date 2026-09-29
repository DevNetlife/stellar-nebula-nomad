//! # Example: Ship NFTs
//!
//! Module: `ship_nft` via `NebulaNomadContract`
//!
//! Shows minting (single and batch), metadata, lookups by owner, transfers,
//! and the `ShipError` codes a dApp should handle.
//!
//! Run:
//! ```text
//! cargo run --example ship_nft
//! ```

use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Bytes, Env};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient, ShipError};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let client = NebulaNomadContractClient::new(&env, &env.register(NebulaNomadContract, ()));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // ── Step 1: Mint a ship ──────────────────────────────────────────────
    // Valid ship types: "fighter", "explorer", "hauler". Each type starts
    // with different hull / scanner stats. `metadata` is free-form bytes
    // stored on-chain (keep it small: storage is paid rent).
    let ship = client.mint_ship(&alice, &symbol_short!("explorer"), &Bytes::new(&env));
    println!(
        "Minted ship #{} ({:?}) hull={} scanner={} durability={}/{}",
        ship.id,
        ship.ship_type,
        ship.hull,
        ship.scanner_power,
        ship.durability,
        ship.max_durability
    );

    // ── Step 2: Batch mint (max 3 per transaction) ───────────────────────
    let fleet = client.batch_mint_ships(
        &alice,
        &vec![&env, symbol_short!("fighter"), symbol_short!("hauler")],
        &Bytes::new(&env),
    );
    println!("Batch minted {} ships", fleet.len());

    // More than 3 in one call is rejected with BatchLimitExceeded (#5).
    let too_many = vec![
        &env,
        symbol_short!("fighter"),
        symbol_short!("fighter"),
        symbol_short!("fighter"),
        symbol_short!("fighter"),
    ];
    assert_eq!(
        client.try_batch_mint_ships(&alice, &too_many, &Bytes::new(&env)),
        Err(Ok(ShipError::BatchLimitExceeded))
    );

    // ── Step 3: Attach marketplace metadata ──────────────────────────────
    // URIs must use a marketplace-compatible scheme (ipfs://, https://, ar://).
    let uri = Bytes::from_slice(
        &env,
        b"ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.set_metadata(&alice, &ship.id, &uri);
    println!(
        "Metadata URI set ({} bytes)",
        client.get_metadata(&ship.id).len()
    );

    let bad_uri = Bytes::from_slice(&env, b"ftp://example.com/ship.json");
    assert_eq!(
        client.try_set_metadata(&alice, &ship.id, &bad_uri),
        Err(Ok(ShipError::InvalidMetadataUri))
    );

    // ── Step 4: Query by owner ───────────────────────────────────────────
    let owned = client.get_ships_by_owner(&alice);
    println!(
        "Alice owns ship ids: {:?}",
        owned.iter().collect::<std::vec::Vec<_>>()
    );

    // ── Step 5: Transfer ─────────────────────────────────────────────────
    // `from` must sign and must be the current owner.
    let moved = client.transfer_ship(&ship.id, &alice, &bob);
    assert_eq!(moved.owner, bob);
    println!("Ship #{} transferred to Bob", moved.id);

    // Alice no longer owns it: NotOwner (#3).
    assert_eq!(
        client.try_transfer_ship(&ship.id, &alice, &bob),
        Err(Ok(ShipError::NotOwner))
    );
    // Unknown id: ShipNotFound (#2).
    assert_eq!(
        client.try_get_ship(&9_999u64),
        Err(Ok(ShipError::ShipNotFound))
    );

    println!("Bob owns {} ship(s)", client.get_ships_by_owner(&bob).len());
}
