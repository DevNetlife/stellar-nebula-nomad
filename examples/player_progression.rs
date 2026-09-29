//! # Example: Player profile, onboarding tutorial and sessions
//!
//! Modules: `player_profile`, `onboarding_tutorial`, `session_manager`
//! (all via `NebulaNomadContract`)
//!
//! A new player's first minutes:
//!   1. create an on-chain profile,
//!   2. go through the onboarding tutorial (earning starter rewards),
//!   3. mint a ship and open a play session,
//!   4. record progress after scanning.
//!
//! Run:
//! ```text
//! cargo run --example player_progression
//! ```

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env};
use stellar_nebula_nomad::{
    NebulaNomadContract, NebulaNomadContractClient, ProfileError, TOTAL_STEPS,
};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let client = NebulaNomadContractClient::new(&env, &env.register(NebulaNomadContract, ()));

    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    // ── Step 1: Profile ──────────────────────────────────────────────────
    // One profile per wallet. The returned id is used for progress updates.
    let profile_id = client.initialize_profile(&player);
    println!("Profile #{profile_id} created");

    // A second profile for the same wallet is rejected.
    assert_eq!(
        client.try_initialize_profile(&player),
        Err(Ok(ProfileError::ProfileAlreadyExists))
    );

    // ── Step 2: Onboarding tutorial ──────────────────────────────────────
    // Admin configures onboarding once; players then walk through steps.
    client.init_onboarding(&admin);
    client.create_profile_onboarding(&player);
    client.start_tutorial(&player);

    let mut earned: i128 = 0;
    for step in 0..TOTAL_STEPS {
        // `try_` lets a UI show a friendly message instead of failing,
        // e.g. when a tutorial path requires steps in a set order.
        match client.try_complete_tutorial_step(&player, &step) {
            Ok(Ok(reward)) => {
                earned += reward;
                println!("  step {step} complete: +{reward}");
            }
            Err(Ok(e)) => println!("  step {step} skipped: {e:?}"),
            other => println!("  step {step}: {other:?}"),
        }
    }
    println!("Tutorial rewards earned: {earned}");
    if let Some(progress) = client.get_tutorial_progress(&player) {
        println!(
            "Steps completed: {}/{}",
            progress.completed_count, TOTAL_STEPS
        );
    }

    // ── Step 3: Ship and session ─────────────────────────────────────────
    let ship = client.mint_ship(&player, &symbol_short!("explorer"), &Bytes::new(&env));
    let session_id = client.start_session(&player, &ship.id);
    let session = client.get_session(&session_id);
    println!("Session #{session_id} opened for ship #{}", session.ship_id);

    // ── Step 4: Record progress ──────────────────────────────────────────
    // After a scan, add the scan count and essence earned. Only the profile
    // owner may update it.
    client.update_progress(&player, &profile_id, &3u32, &150i128);
    let profile = client.get_profile(&profile_id);
    println!(
        "Profile #{}: scans={} essence={}",
        profile.id, profile.total_scans, profile.essence_earned
    );

    let stranger = Address::generate(&env);
    assert_eq!(
        client.try_update_progress(&stranger, &profile_id, &1u32, &1i128),
        Err(Ok(ProfileError::Unauthorized))
    );

    // ── Step 5: Close the session ────────────────────────────────────────
    match client.try_expire_session(&player, &session_id) {
        Ok(Ok(())) => println!("Session #{session_id} closed"),
        Err(Ok(e)) => println!("Session not closed yet: {e:?}"),
        other => println!("Unexpected: {other:?}"),
    }
}
