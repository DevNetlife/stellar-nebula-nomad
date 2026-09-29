#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
use stellar_nebula_nomad::{BindingError, SoulBinding};

#[test]
fn test_bind_ship_to_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let ship_id = 1u64;

    let binding = stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner.clone(),
        ship_id,
    )
    .unwrap();

    assert_eq!(binding.ship_id, ship_id);
    assert_eq!(binding.bound_to, owner);
    assert!(binding.bound_at > 0);
}

#[test]
fn test_check_binding_status() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let ship_id = 2u64;

    let status =
        stellar_nebula_nomad::NebulaNomadContract::check_binding_status(env.clone(), ship_id);
    assert!(status.is_none());

    stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner.clone(),
        ship_id,
    )
    .unwrap();

    let status =
        stellar_nebula_nomad::NebulaNomadContract::check_binding_status(env.clone(), ship_id);
    assert!(status.is_some());
    assert_eq!(status.unwrap().bound_to, owner);
}

#[test]
fn test_already_bound_error() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let ship_id = 3u64;

    stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner.clone(),
        ship_id,
    )
    .unwrap();

    let result = stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner.clone(),
        ship_id,
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BindingError::AlreadyBound);
}

#[test]
fn test_is_bound_to() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let ship_id = 4u64;

    let is_bound =
        stellar_nebula_nomad::NebulaNomadContract::is_bound_to(env.clone(), ship_id, owner.clone());
    assert!(!is_bound);

    stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner.clone(),
        ship_id,
    )
    .unwrap();

    let is_bound_owner =
        stellar_nebula_nomad::NebulaNomadContract::is_bound_to(env.clone(), ship_id, owner.clone());
    assert!(is_bound_owner);

    let is_bound_other =
        stellar_nebula_nomad::NebulaNomadContract::is_bound_to(env.clone(), ship_id, other);
    assert!(!is_bound_other);
}

#[test]
fn test_batch_bind_ships() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let mut ship_ids = Vec::new(&env);
    ship_ids.push_back(10);
    ship_ids.push_back(11);
    ship_ids.push_back(12);

    let bindings = stellar_nebula_nomad::NebulaNomadContract::batch_bind_ships(
        env.clone(),
        owner.clone(),
        ship_ids.clone(),
    )
    .unwrap();

    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings.get(0).unwrap().ship_id, 10);
    assert_eq!(bindings.get(1).unwrap().ship_id, 11);
    assert_eq!(bindings.get(2).unwrap().ship_id, 12);

    for i in 0..3 {
        let ship_id = ship_ids.get(i).unwrap();
        let is_bound = stellar_nebula_nomad::NebulaNomadContract::is_bound_to(
            env.clone(),
            ship_id,
            owner.clone(),
        );
        assert!(is_bound);
    }
}

#[test]
fn test_batch_bind_exceeds_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let mut ship_ids = Vec::new(&env);
    for i in 0..4 {
        ship_ids.push_back(20 + i);
    }

    let result =
        stellar_nebula_nomad::NebulaNomadContract::batch_bind_ships(env.clone(), owner, ship_ids);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BindingError::BurstLimitExceeded);
}

#[test]
fn test_binding_is_immutable() {
    let env = Env::default();
    env.mock_all_auths();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let ship_id = 5u64;

    stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner1.clone(),
        ship_id,
    )
    .unwrap();

    let result = stellar_nebula_nomad::NebulaNomadContract::bind_ship_to_owner(
        env.clone(),
        owner2.clone(),
        ship_id,
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BindingError::AlreadyBound);

    let is_bound_owner1 =
        stellar_nebula_nomad::NebulaNomadContract::is_bound_to(env.clone(), ship_id, owner1);
    assert!(is_bound_owner1);
}
