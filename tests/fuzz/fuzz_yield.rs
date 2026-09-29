//! Fuzz testing for yield farming and arithmetic operations

#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient};

fn setup() -> (Env, NebulaNomadContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 100,
        timestamp: 1_700_000_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    });
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    let player = Address::generate(&env);
    (env, client, player)
}

fn advance_ledgers(env: &Env, count: u32) {
    let current = env.ledger().sequence();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: current + count,
        timestamp: env.ledger().timestamp() + (count as u64 * 5),
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    });
}

proptest! {
    #[test]
    fn prop_difficulty_weights_sum_100(level in 1u32..=100u32) {
        let (_, client, _) = setup();
        let result = client.calculate_difficulty(&level);
        let w = &result.rarity_weights;
        prop_assert_eq!(w.common + w.uncommon + w.rare + w.epic + w.legendary, 100u32);
    }

    #[test]
    fn prop_ship_mint_valid_types(idx in 0usize..=2usize) {
        let (env, client, player) = setup();
        let metadata = Bytes::from_array(&env, &[0u8; 4]);
        let stype = match idx {
            0 => symbol_short!("fighter"),
            1 => symbol_short!("explorer"),
            _ => symbol_short!("hauler"),
        };
        let result = client.try_mint_ship(&player, &stype, &metadata);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_vault_deposit_positive(amount in 1u64..10000u64) {
        let (env, client, player) = setup();
        let metadata = Bytes::from_array(&env, &[0u8; 4]);
        let ship = client.mint_ship(&player, &symbol_short!("hauler"), &metadata);
        let result = client.try_deposit_treasure(&player, &ship.id, &amount);
        prop_assert!(result.is_ok());
    }
}

#[test]
fn fuzz_zero_vault_deposit() {
    let (env, client, player) = setup();
    let metadata = Bytes::from_array(&env, &[0u8; 4]);
    let ship = client.mint_ship(&player, &symbol_short!("fighter"), &metadata);
    let result = client.try_deposit_treasure(&player, &ship.id, &0u64);
    assert!(result.is_err());
}

#[test]
fn fuzz_nebula_layout_determinism() {
    let (env, client, player) = setup();
    let seed = BytesN::from_array(&env, &[42u8; 32]);

    let layout1 = client.generate_nebula_layout(&seed, &player);
    let layout2 = client.generate_nebula_layout(&seed, &player);

    assert_eq!(layout1.cells.len(), layout2.cells.len());
    assert_eq!(layout1.total_energy, layout2.total_energy);
}

#[test]
fn fuzz_rarity_calculation() {
    let (env, client, player) = setup();
    let seed = BytesN::from_array(&env, &[1u8; 32]);
    let layout = client.generate_nebula_layout(&seed, &player);
    let rarity = client.calculate_rarity_tier(&layout);

    // Rarity should be one of the valid enum values
    // This test just ensures no panic occurs
    let _ = rarity;
}

#[test]
fn fuzz_batch_ship_mint() {
    let (env, client, player) = setup();
    let metadata = Bytes::from_array(&env, &[0u8; 4]);

    for _ in 0..5 {
        let result = client.try_mint_ship(&player, &symbol_short!("explorer"), &metadata);
        assert!(result.is_ok());
    }

    let ships = client.get_ships_by_owner(&player);
    assert_eq!(ships.len(), 5);
}
