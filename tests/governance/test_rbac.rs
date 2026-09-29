#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Env, Vec};
use stellar_nebula_nomad::access_control::{self, AccessControlError, BATCH_GRANT_LIMIT};

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

// ── Initialization Tests ──

#[test]
fn test_init_rbac_creates_admin_role() {
    let (env, admin) = setup_env();
    let result = access_control::init_roles(&env, admin.clone());
    assert!(result.is_ok(), "init_roles should succeed");
    assert!(
        access_control::has_role(&env, &symbol_short!("admin"), &admin),
        "Admin should hold admin role after init"
    );
}

#[test]
fn test_init_rbac_idempotent_fails_on_retry() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let result = access_control::init_roles(&env, admin);
    assert_eq!(
        result,
        Err(AccessControlError::InitializationFailed),
        "Second init should fail with InitializationFailed"
    );
}

// ── Role Storage and Retrieval ──

#[test]
fn test_has_role_false_for_ungranted() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let player = Address::generate(&env);
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_has_role_true_after_grant() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    access_control::grant_role(&env, admin, symbol_short!("nomad"), player.clone(), None).unwrap();
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_has_role_false_after_revoke() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        None,
    )
    .unwrap();
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
    access_control::revoke_role(&env, admin, symbol_short!("nomad"), player.clone()).unwrap();
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_has_role_false_after_expiry() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let expiry = env.ledger().sequence() + 10;
    access_control::grant_role(
        &env,
        admin,
        symbol_short!("nomad"),
        player.clone(),
        Some(expiry),
    )
    .unwrap();
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
    advance_ledger(&env, 10);
    assert!(
        !access_control::has_role(&env, &symbol_short!("nomad"), &player),
        "Role should be false at expiry ledger"
    );
}

#[test]
fn test_has_role_true_before_expiry() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let expiry = env.ledger().sequence() + 50;
    access_control::grant_role(
        &env,
        admin,
        symbol_short!("nomad"),
        player.clone(),
        Some(expiry),
    )
    .unwrap();
    advance_ledger(&env, 10);
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_has_role_true_forever_without_expiry() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    access_control::grant_role(&env, admin, symbol_short!("nomad"), player.clone(), None).unwrap();
    advance_ledger(&env, 1000);
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

// ── Permission Storage and Retrieval ──

#[test]
fn test_has_permission_false_for_undefined() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    assert!(!access_control::has_permission(
        &env,
        &symbol_short!("nomad"),
        &symbol_short!("scan")
    ));
}

#[test]
fn test_has_permission_true_after_grant() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    access_control::grant_permission(&env, admin, symbol_short!("nomad"), symbol_short!("scan"))
        .unwrap();
    assert!(access_control::has_permission(
        &env,
        &symbol_short!("nomad"),
        &symbol_short!("scan")
    ));
}

#[test]
fn test_has_permission_false_after_revoke() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let action = symbol_short!("scan");
    access_control::grant_permission(&env, admin.clone(), symbol_short!("nomad"), action.clone())
        .unwrap();
    assert!(access_control::has_permission(
        &env,
        &symbol_short!("nomad"),
        &action
    ));
    access_control::revoke_permission(&env, admin, symbol_short!("nomad"), action.clone()).unwrap();
    assert!(!access_control::has_permission(
        &env,
        &symbol_short!("nomad"),
        &action
    ));
}

// ── grant_role Tests ──

#[test]
fn test_grant_role_admin_success() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let result =
        access_control::grant_role(&env, admin, symbol_short!("nomad"), player.clone(), None);
    assert!(result.is_ok());
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_grant_role_non_admin_fails() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let non_admin = Address::generate(&env);
    let player = Address::generate(&env);
    let result = access_control::grant_role(
        &env,
        non_admin,
        symbol_short!("nomad"),
        player.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::AdminRequired));
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_grant_role_past_expiry_fails() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let past = env.ledger().sequence() - 1;
    let result =
        access_control::grant_role(&env, admin, symbol_short!("nomad"), player, Some(past));
    assert_eq!(result, Err(AccessControlError::InvalidExpiry));
}

// ── grant_role_batch Tests ──

#[test]
fn test_batch_grant_1_to_5_addresses() {
    for batch_size in 1..=5 {
        let (env, admin) = setup_env();
        access_control::init_roles(&env, admin.clone()).unwrap();
        let mut grantees = Vec::new(&env);
        for _ in 0..batch_size {
            grantees.push_back(Address::generate(&env));
        }
        let result = access_control::grant_role_batch(
            &env,
            admin,
            symbol_short!("nomad"),
            grantees.clone(),
            None,
        );
        assert!(
            result.is_ok(),
            "Batch grant of {} should succeed",
            batch_size
        );
        for i in 0..grantees.len() {
            if let Some(g) = grantees.get(i) {
                assert!(access_control::has_role(&env, &symbol_short!("nomad"), &g));
            }
        }
    }
}

#[test]
fn test_batch_grant_6_addresses_fails() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let mut grantees = Vec::new(&env);
    for _ in 0..6 {
        grantees.push_back(Address::generate(&env));
    }
    let result = access_control::grant_role_batch(
        &env,
        admin,
        symbol_short!("nomad"),
        grantees.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::BatchLimitExceeded));
    // Verify no roles were granted
    for i in 0..grantees.len() {
        if let Some(g) = grantees.get(i) {
            assert!(
                !access_control::has_role(&env, &symbol_short!("nomad"), &g),
                "No roles should be granted when batch exceeds limit"
            );
        }
    }
}

// ── check_permission Tests ──

#[test]
fn test_check_permission_succeeds_with_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");

    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        None,
    )
    .unwrap();
    access_control::grant_permission(&env, admin, symbol_short!("nomad"), action.clone()).unwrap();

    let result = access_control::check_permission(&env, &player, &action);
    assert!(
        result.is_ok(),
        "check_permission should succeed when role permits"
    );
}

#[test]
fn test_check_permission_fails_no_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");

    access_control::grant_permission(&env, admin, symbol_short!("nomad"), action.clone()).unwrap();

    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

#[test]
fn test_check_permission_fails_no_permission() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");

    access_control::grant_role(&env, admin, symbol_short!("nomad"), player.clone(), None).unwrap();

    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

#[test]
fn test_check_permission_fails_expired_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");
    let expiry = env.ledger().sequence() + 5;

    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        Some(expiry),
    )
    .unwrap();
    access_control::grant_permission(&env, admin, symbol_short!("nomad"), action.clone()).unwrap();

    advance_ledger(&env, 5);

    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

#[test]
fn test_check_permission_fails_revoked_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");

    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        None,
    )
    .unwrap();
    access_control::grant_permission(&env, admin.clone(), symbol_short!("nomad"), action.clone())
        .unwrap();

    assert!(access_control::check_permission(&env, &player, &action).is_ok());

    access_control::revoke_role(&env, admin, symbol_short!("nomad"), player.clone()).unwrap();

    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

#[test]
fn test_check_permission_multiple_roles_any_permits() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("scan");

    // Grant player two roles
    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        None,
    )
    .unwrap();
    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("indexer"),
        player.clone(),
        None,
    )
    .unwrap();

    // Grant permission to only second role
    access_control::grant_permission(&env, admin, symbol_short!("indexer"), action.clone())
        .unwrap();

    // Should succeed because player holds indexer role which has permission
    let result = access_control::check_permission(&env, &player, &action);
    assert!(result.is_ok(), "Should succeed if any role has permission");
}

// ── transfer_admin Tests ──

#[test]
fn test_transfer_admin_succeeds() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let new_admin = Address::generate(&env);

    let result = access_control::transfer_admin(&env, admin.clone(), new_admin.clone());
    assert!(result.is_ok());

    assert!(access_control::has_role(
        &env,
        &symbol_short!("admin"),
        &new_admin
    ));
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("admin"),
        &admin
    ));
}

#[test]
fn test_transfer_admin_non_admin_fails() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let non_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let result = access_control::transfer_admin(&env, non_admin, new_admin);
    assert_eq!(result, Err(AccessControlError::AdminRequired));
}

#[test]
fn test_new_admin_can_perform_admin_actions() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let new_admin = Address::generate(&env);
    let player = Address::generate(&env);

    access_control::transfer_admin(&env, admin, new_admin.clone()).unwrap();

    let result = access_control::grant_role(
        &env,
        new_admin,
        symbol_short!("nomad"),
        player.clone(),
        None,
    );
    assert!(result.is_ok());
    assert!(access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_old_admin_loses_privileges_after_transfer() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let new_admin = Address::generate(&env);
    let player = Address::generate(&env);

    access_control::transfer_admin(&env, admin.clone(), new_admin).unwrap();

    let result = access_control::grant_role(&env, admin, symbol_short!("nomad"), player, None);
    assert_eq!(result, Err(AccessControlError::AdminRequired));
}

// ── Privilege Escalation Attack Simulations ──

#[test]
fn test_non_admin_cannot_grant_admin_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let attacker = Address::generate(&env);

    let result = access_control::grant_role(
        &env,
        attacker.clone(),
        symbol_short!("admin"),
        attacker.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::AdminRequired));
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("admin"),
        &attacker
    ));
}

#[test]
fn test_non_admin_cannot_grant_arbitrary_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let result = access_control::grant_role(
        &env,
        attacker.clone(),
        symbol_short!("nomad"),
        target.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::AdminRequired));
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &target
    ));
}

// ── Role Expiry Bypass Attack Simulation ──

#[test]
fn test_cannot_use_expired_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("sensitive");

    let expiry = env.ledger().sequence() + 1;
    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        Some(expiry),
    )
    .unwrap();
    access_control::grant_permission(&env, admin, symbol_short!("nomad"), action.clone()).unwrap();

    // At expiry, should fail
    advance_ledger(&env, 1);
    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

// ── Revoked Role Bypass Attack Simulation ──

#[test]
fn test_cannot_use_revoked_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);
    let action = symbol_short!("sensitive");

    access_control::grant_role(
        &env,
        admin.clone(),
        symbol_short!("nomad"),
        player.clone(),
        None,
    )
    .unwrap();
    access_control::grant_permission(&env, admin.clone(), symbol_short!("nomad"), action.clone())
        .unwrap();

    // Should work initially
    assert!(access_control::check_permission(&env, &player, &action).is_ok());

    // Revoke and try again
    access_control::revoke_role(&env, admin, symbol_short!("nomad"), player.clone()).unwrap();
    let result = access_control::check_permission(&env, &player, &action);
    assert_eq!(result, Err(AccessControlError::UnauthorizedRole));
}

// ── Storage Key Collision Test ──

#[test]
fn test_role_membership_distinct_from_permissions() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);

    // Try to create collision: same string used as both role and action
    let shared_name = symbol_short!("admin");

    // Grant roleAS a role to player
    access_control::grant_role(
        &env,
        admin.clone(),
        shared_name.clone(),
        player.clone(),
        None,
    )
    .unwrap();

    // Grant permission for action "admin" to role "nomad" (should not affect role membership)
    access_control::grant_permission(&env, admin, symbol_short!("nomad"), shared_name.clone())
        .unwrap();

    // Player should have the "admin" role
    assert!(access_control::has_role(&env, &shared_name, &player));

    // But nomad should have permission for "admin" action, not affected by role grant
    assert!(access_control::has_permission(
        &env,
        &symbol_short!("nomad"),
        &shared_name
    ));
}

// ── Admin Transfer Race Condition Test ──

#[test]
fn test_old_admin_cannot_perform_admin_actions_after_transfer() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let new_admin = Address::generate(&env);
    let player = Address::generate(&env);

    access_control::transfer_admin(&env, admin.clone(), new_admin).unwrap();

    // Old admin tries to revoke role (admin-only operation)
    let result = access_control::revoke_role(&env, admin, symbol_short!("nomad"), player);
    assert_eq!(
        result,
        Err(AccessControlError::AdminRequired),
        "Old admin should not be able to revoke roles"
    );
}

// ── Idempotency and Vacuousness Checks ──

#[test]
fn test_failed_grant_role_leaves_no_state_change() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin).unwrap();
    let non_admin = Address::generate(&env);
    let player = Address::generate(&env);

    // Attempt grant by non-admin (should fail)
    let result = access_control::grant_role(
        &env,
        non_admin,
        symbol_short!("nomad"),
        player.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::AdminRequired));

    // Verify no role was granted
    assert!(!access_control::has_role(
        &env,
        &symbol_short!("nomad"),
        &player
    ));
}

#[test]
fn test_failed_batch_grant_granular_check() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let mut grantees = Vec::new(&env);
    for _ in 0..6 {
        grantees.push_back(Address::generate(&env));
    }

    let result = access_control::grant_role_batch(
        &env,
        admin,
        symbol_short!("nomad"),
        grantees.clone(),
        None,
    );
    assert_eq!(result, Err(AccessControlError::BatchLimitExceeded));

    // Verify NONE of the 6 addresses got the role (all-or-nothing)
    for i in 0..grantees.len() {
        if let Some(g) = grantees.get(i) {
            assert!(
                !access_control::has_role(&env, &symbol_short!("nomad"), &g),
                "No address should hold role when batch exceeds limit"
            );
        }
    }
}

#[test]
fn test_revoke_idempotent_success_on_non_held_role() {
    let (env, admin) = setup_env();
    access_control::init_roles(&env, admin.clone()).unwrap();
    let player = Address::generate(&env);

    // Revoke a role the player never had
    let result = access_control::revoke_role(&env, admin, symbol_short!("nomad"), player.clone());
    assert!(
        result.is_ok(),
        "Revoke should succeed even if role was never held"
    );
}
