#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient, GRID_SIZE};

#[test]
fn test_season_scheduling() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let title = String::from_str(&env, "Season 1: Deep Space");

    // Initialize season
    let season_id = client.init_season(&admin, &title);
    assert_eq!(season_id, 1u64);

    let current = client.get_current_season();
    assert_eq!(current.id, 1u64);
    assert_eq!(current.title, title);

    // Advance time by 31 days
    env.ledger().set_timestamp(31 * 24 * 60 * 60);

    // Reset season
    let new_title = String::from_str(&env, "Season 2: Nebula Nomad");
    let next_id = client.reset_season(&admin, &new_title);
    assert_eq!(next_id, 2u64);

    let current_new = client.get_current_season();
    assert_eq!(current_new.id, 2u64);
    assert_eq!(current_new.title, new_title);
}

#[test]
fn test_battle_pass_progression() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    // Setup
    client.init_onboarding(&admin);
    client.create_profile(&player);
    client.init_season(&admin, &String::from_str(&env, "Season 1"));
    client.init_battle_pass_rewards(&admin);

    let profile = client.get_profile_by_owner(&player);

    // Execute scan
    let seed = BytesN::from_array(&env, &[0u8; 32]);
    client.scan_nebula(&seed, &player);

    // Verify XP (scan grants XP)
    let bp_state = client.get_battle_pass_state(&profile.id);
    assert!(bp_state.xp > 0);
    assert_eq!(bp_state.season_id, 1u64);
}

#[test]
fn test_battle_pass_reward_claiming() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    // Setup
    client.init_onboarding(&admin);
    client.create_profile(&player);
    client.init_season(&admin, &String::from_str(&env, "Season 1"));
    client.init_battle_pass_rewards(&admin);

    let profile = client.get_profile_by_owner(&player);

    // Grant XP manually (mocking multiple scans)
    // XP needed for tier 1 is 100.
    // 10 scans @ 10 XP each = 100 XP.
    let seed_base = [0u8; 32];
    for i in 0..10 {
        let mut seed = seed_base.clone();
        seed[0] = i as u8;
        client.scan_nebula(&BytesN::from_array(&env, &seed), &player);
    }

    let bp_state = client.get_battle_pass_state(&profile.id);
    assert!(bp_state.xp >= 100);

    // Claim reward
    let reward = client.claim_bp_reward(&player, &profile.id, &1u32);
    assert_eq!(reward, 50i128);

    // Try to claim again (should fail)
    let result = client.try_claim_bp_reward(&player, &profile.id, &1u32);
    assert!(result.is_err());
}
