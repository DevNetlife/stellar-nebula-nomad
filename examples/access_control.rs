//! # Example: Role-based access control (RBAC)
//!
//! Module: `access_control` via `NebulaNomadContract`
//!
//! Shows how to set up an admin, grant and revoke roles (optionally with
//! an expiry ledger), attach permissions to roles, and hand over admin.
//!
//! Run:
//! ```text
//! cargo run --example access_control
//! ```

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use stellar_nebula_nomad::{AccessControlError, NebulaNomadContract, NebulaNomadContractClient};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let client = NebulaNomadContractClient::new(&env, &env.register(NebulaNomadContract, ()));

    let admin = Address::generate(&env);
    let indexer = Address::generate(&env);
    let moderator = Address::generate(&env);

    // ── Step 1: Bootstrap ────────────────────────────────────────────────
    // Call once after deployment. The admin can then grant roles.
    client.init_rbac(&admin);
    println!("RBAC initialised");

    // ── Step 2: Grant roles ──────────────────────────────────────────────
    // Built-in roles: "admin", "nomad", "indexer". Custom roles are
    // registered automatically the first time they are granted.
    let indexer_role = symbol_short!("indexer");
    let mod_role = symbol_short!("moderatr");
    client.grant_role(&admin, &indexer_role, &indexer, &None);
    client.grant_role(&admin, &mod_role, &moderator, &None);
    println!(
        "indexer has role: {} | moderator has role: {}",
        client.has_role(&indexer_role, &indexer),
        client.has_role(&mod_role, &moderator)
    );

    // ── Step 3: Permissions ──────────────────────────────────────────────
    // Map an action symbol to a role; contract code then checks
    // `check_permission(&env, &caller, action)`.
    let action = symbol_short!("hide_post");
    client.grant_permission(&admin, &mod_role, &action);
    println!(
        "moderator can hide posts: {}",
        client.has_permission(&mod_role, &action)
    );

    // ── Step 4: Only the admin can grant ─────────────────────────────────
    let result = client.try_grant_role(&moderator, &mod_role, &indexer, &None);
    assert_eq!(result, Err(Ok(AccessControlError::AdminRequired)));
    println!(
        "Non-admin grant rejected: {:?}",
        AccessControlError::AdminRequired
    );

    // Expiry must be a future ledger sequence.
    let past = env.ledger().sequence();
    assert_eq!(
        client.try_grant_role(&admin, &mod_role, &indexer, &Some(past)),
        Err(Ok(AccessControlError::InvalidExpiry))
    );

    // ── Step 5: Revoke ───────────────────────────────────────────────────
    client.revoke_role(&admin, &mod_role, &moderator);
    println!(
        "moderator has role after revoke: {}",
        client.has_role(&mod_role, &moderator)
    );

    // ── Step 6: Hand over admin ──────────────────────────────────────────
    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    println!("Admin transferred");
}
