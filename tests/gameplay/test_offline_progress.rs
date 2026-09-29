#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Vec,
};
use stellar_nebula_nomad::{OfflineError, OfflineProgress, OfflineYieldClaim};

#[test]
fn test_record_last_active() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    let progress =
        stellar_nebula_nomad::NebulaNomadContract::get_offline_progress(env.clone(), player);

    assert!(progress.is_some());
    let p = progress.unwrap();
    assert_eq!(p.last_active, env.ledger().timestamp());
    assert_eq!(p.total_accrued, 0);
}

#[test]
fn test_claim_offline_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 7200; // 2 hours
    });

    let claim =
        stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player.clone())
            .unwrap();

    assert_eq!(claim.offline_duration, 7200);
    assert_eq!(claim.yield_amount, 200); // 2 hours * 100 per hour

    let progress =
        stellar_nebula_nomad::NebulaNomadContract::get_offline_progress(env.clone(), player)
            .unwrap();
    assert_eq!(progress.total_accrued, 200);
}

#[test]
fn test_no_accrual_available() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    let result =
        stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), OfflineError::NoAccrualAvailable);
}

#[test]
fn test_offline_yield_capped_at_48_hours() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + (72 * 3600); // 72 hours
    });

    let claim = stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player)
        .unwrap();

    assert_eq!(claim.offline_duration, 48 * 3600); // Capped at 48 hours
    assert_eq!(claim.yield_amount, 4800); // 48 hours * 100 per hour
}

#[test]
fn test_calculate_pending_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 3600; // 1 hour
    });

    let pending =
        stellar_nebula_nomad::NebulaNomadContract::calculate_pending_yield(env.clone(), player)
            .unwrap();

    assert_eq!(pending, 100); // 1 hour * 100 per hour
}

#[test]
fn test_multiple_claims_accumulate() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 3600; // 1 hour
    });

    let claim1 =
        stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player.clone())
            .unwrap();
    assert_eq!(claim1.yield_amount, 100);

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 7200; // 2 more hours
    });

    let claim2 =
        stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player.clone())
            .unwrap();
    assert_eq!(claim2.yield_amount, 200);

    let progress =
        stellar_nebula_nomad::NebulaNomadContract::get_offline_progress(env.clone(), player)
            .unwrap();
    assert_eq!(progress.total_accrued, 300); // 100 + 200
}

#[test]
fn test_batch_claim_offline_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let mut players = Vec::new(&env);
    for _ in 0..3 {
        let player = Address::generate(&env);
        stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());
        players.push_back(player);
    }

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 3600; // 1 hour
    });

    let claims =
        stellar_nebula_nomad::NebulaNomadContract::batch_claim_offline_yield(env.clone(), players);

    assert_eq!(claims.len(), 3);
    for i in 0..3 {
        let claim = claims.get(i).unwrap();
        assert_eq!(claim.yield_amount, 100);
    }
}

#[test]
fn test_not_initialized_error() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    let result =
        stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), OfflineError::NotInitialized);
}

#[test]
fn test_partial_hour_accrual() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::record_last_active(env.clone(), player.clone());

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 5400; // 1.5 hours (5400 seconds)
    });

    let claim = stellar_nebula_nomad::NebulaNomadContract::claim_offline_yield(env.clone(), player)
        .unwrap();

    assert_eq!(claim.yield_amount, 100); // Only 1 complete hour counts
}
