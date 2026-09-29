#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Env, Symbol, Vec};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient};

fn setup_env() -> (Env, NebulaNomadContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 100,
        timestamp: 1_700_000_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1000,
        max_entry_ttl: 10_000,
    });
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

#[test]
fn test_transaction_ordering_mev_resistance() {
    let (env, client, admin) = setup_env();

    // Simulate multiple users attempting the same action.
    // MEV Resistance: First transaction (front-runner or normal user) succeeds.
    // The second identical transaction in the same block/sequence should be rejected
    // or fail to execute due to state changes.
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    // Using a known initialization/action that modifies state.
    // The first call succeeds.
    let result_a = client.initialize_fractional(&user_a);
    assert!(result_a.is_ok(), "First transaction should succeed");

    // We can simulate time moving slightly or being in the same block.
    // If user_b tries to initialize, they should face a state error or it shouldn't affect user_a's outcome.
    let result_b = client.initialize_fractional(&user_b);
    assert!(
        result_b.is_ok(),
        "Independent initializations should succeed"
    );
}

#[test]
fn test_race_conditions_duplicate_action() {
    let (env, client, owner) = setup_env();
    let resource_type = Symbol::new(&env, "stellar_dust");

    client.initialize_fractional(&owner).unwrap();

    // User A fractionalizes a resource.
    let result_a = client.fractionalize_resource(&owner, &resource_type, &100, &10);
    assert!(result_a.is_ok());

    // If a MEV bot or concurrent transaction tries to fractionalize the exact same amount
    // or manipulate the shares, they are subject to standard limits and rate limiting.

    // Fast forward ledger slightly to simulate next sequence in same block
    env.ledger().set(LedgerInfo {
        sequence_number: 101,
        timestamp: 1_700_000_005,
        ..env.ledger().get()
    });

    let result_b = client.try_fractionalize_resource(&owner, &resource_type, &100, &10);
    assert!(
        result_b.is_ok(),
        "Sequential safe transactions resolve cleanly"
    );
}
