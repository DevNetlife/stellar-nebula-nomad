#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Env};
use stellar_nebula_nomad::staking::{self, DelegationRecord, StakeRecord, StakingError};

fn setup_env() -> (Env, Address) {
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
    let admin = Address::generate(&env);
    (env, admin)
}

fn advance_ledger(env: &Env, count: u32) {
    let seq = env.ledger().sequence();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: seq + count,
        timestamp: env.ledger().timestamp() + (count as u64) * 5,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    });
}

#[test]
fn test_initialize_staking() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    let result = staking::initialize(
        env.clone(),
        admin.clone(),
        token.clone(),
        min_stake,
        lock_duration,
    );
    assert!(result.is_ok());

    // Verify initialization cannot happen twice.
    let result2 = staking::initialize(
        env.clone(),
        admin.clone(),
        token.clone(),
        min_stake,
        lock_duration,
    );
    assert_eq!(result2, Err(StakingError::AlreadyInitialized));
}

#[test]
fn test_stake_below_minimum_rejected() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let below_min = min_stake - 1;

    let result = staking::stake(env, staker, below_min);
    assert_eq!(result, Err(StakingError::InvalidAmount));
}

#[test]
fn test_stake_zero_rejected() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let result = staking::stake(env, staker, 0);
    assert_eq!(result, Err(StakingError::InvalidAmount));
}

#[test]
fn test_stake_successful() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    let result = staking::stake(env.clone(), staker.clone(), amount);
    assert!(result.is_ok());

    // Verify stake record was created.
    let stake = staking::get_stake(env.clone(), staker.clone()).unwrap();
    assert_eq!(stake.amount, amount);
    assert_eq!(stake.staker, staker);
    assert_eq!(stake.created_ledger, 100);
    assert_eq!(stake.unlock_ledger, 150);

    // Verify total staked updated.
    let total = staking::get_total_staked(env);
    assert_eq!(total, amount);
}

#[test]
fn test_voting_power_equals_stake() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker.clone(), amount).unwrap();

    // Advance at least 1 ledger so stake is old enough.
    advance_ledger(&env, 1);

    let power = staking::get_voting_power(env, staker);
    assert_eq!(power, amount);
}

#[test]
fn test_voting_power_zero_for_unstaked() {
    let (env, _admin) = setup_env();

    let unstaked = Address::generate(&env);
    let power = staking::get_voting_power(env, unstaked);
    assert_eq!(power, 0);
}

#[test]
fn test_stake_too_young_no_voting_power() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker.clone(), amount).unwrap();

    // Do NOT advance ledger. Stake is in same ledger, so should have 0 power.
    let power = staking::get_voting_power(env, staker);
    assert_eq!(power, 0);
}

#[test]
fn test_unstake_before_lock_rejected() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker.clone(), amount).unwrap();

    // Try unstake before unlock_ledger. Should fail.
    let result = staking::unstake(env, staker);
    assert_eq!(result, Err(StakingError::TimeLockActive));
}

#[test]
fn test_unstake_after_lock_successful() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker.clone(), amount).unwrap();

    // Advance past unlock_ledger.
    advance_ledger(&env, 51);

    let result = staking::unstake(env.clone(), staker.clone());
    assert_eq!(result, Ok(amount));

    // Verify stake record is deleted.
    let stake = staking::get_stake(env.clone(), staker.clone());
    assert!(stake.is_none());

    // Verify total staked is decreased.
    let total = staking::get_total_staked(env);
    assert_eq!(total, 0);
}

#[test]
fn test_unstake_no_stake_rejected() {
    let (env, _admin) = setup_env();

    let staker = Address::generate(&env);
    let result = staking::unstake(env, staker);
    assert_eq!(result, Err(StakingError::NoActiveStake));
}

#[test]
fn test_delegate_voting_power() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker_a = Address::generate(&env);
    let staker_b = Address::generate(&env);
    let amount = 5_000_000;

    // A stakes.
    staking::stake(env.clone(), staker_a.clone(), amount).unwrap();
    advance_ledger(&env, 1);

    // Verify A has voting power before delegation.
    let power_a = staking::get_voting_power(env.clone(), staker_a.clone());
    assert_eq!(power_a, amount);

    // A delegates to B.
    staking::delegate(env.clone(), staker_a.clone(), staker_b.clone()).unwrap();

    // Now A has 0 power, and B would have A's power (if B had staked, but B doesn't stake).
    let power_a_after = staking::get_voting_power(env.clone(), staker_a);
    assert_eq!(power_a_after, 0);
}

#[test]
fn test_self_delegation_rejected() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker.clone(), amount).unwrap();

    // Try to delegate to self.
    let result = staking::delegate(env, staker.clone(), staker.clone());
    assert_eq!(result, Err(StakingError::SelfDelegation));
}

#[test]
fn test_circular_delegation_rejected() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker_a = Address::generate(&env);
    let staker_b = Address::generate(&env);
    let amount = 5_000_000;

    // Both stake.
    staking::stake(env.clone(), staker_a.clone(), amount).unwrap();
    staking::stake(env.clone(), staker_b.clone(), amount).unwrap();

    // A delegates to B.
    staking::delegate(env.clone(), staker_a.clone(), staker_b.clone()).unwrap();

    // B tries to delegate to A (circular).
    let result = staking::delegate(env, staker_b, staker_a);
    assert_eq!(result, Err(StakingError::CircularDelegation));
}

#[test]
fn test_undelegate() {
    let (env, admin) = setup_env();
    let token = Address::generate(&env);
    let min_stake = 1_000_000;
    let lock_duration = 50;

    staking::initialize(env.clone(), admin, token, min_stake, lock_duration).unwrap();

    let staker_a = Address::generate(&env);
    let staker_b = Address::generate(&env);
    let amount = 5_000_000;

    staking::stake(env.clone(), staker_a.clone(), amount).unwrap();
    staking::stake(env.clone(), staker_b.clone(), amount).unwrap();

    // A delegates to B.
    staking::delegate(env.clone(), staker_a.clone(), staker_b.clone()).unwrap();

    // A's power is now 0.
    advance_ledger(&env, 1);
    let power = staking::get_voting_power(env.clone(), staker_a.clone());
    assert_eq!(power, 0);

    // A undelegates.
    staking::undelegate(env.clone(), staker_a.clone()).unwrap();

    // A's power is restored.
    let power_after = staking::get_voting_power(env, staker_a);
    assert_eq!(power_after, amount);
}

#[test]
fn test_undelegate_no_delegation_rejected() {
    let (env, _admin) = setup_env();

    let staker = Address::generate(&env);
    let result = staking::undelegate(env, staker);
    assert_eq!(result, Err(StakingError::NoActiveDelegation));
}
