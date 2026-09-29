//! # Example: Sponsoring a new player's first scan
//!
//! Module: `gas_sponsor` (library helpers called from contract functions)
//!
//! New players often have no XLM for fees. The sponsorship pool pays for
//! their first scan, within global and per-player caps.
//!
//! Run:
//! ```text
//! cargo run --example gas_sponsorship
//! ```

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_nebula_nomad::{
    get_config, get_daily_count, get_fund_balance, get_remaining_daily_slots, has_been_sponsored,
    initialize_sponsorship, mark_profile_verified, sponsor_first_scan, update_config,
    NebulaNomadContract, SponsorError,
};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let host = env.register(NebulaNomadContract, ());

    let admin = Address::generate(&env);
    let newcomer = Address::generate(&env);

    env.as_contract(&host, || {
        // ── Step 1: Fund the pool (admin) ────────────────────────────────
        // Amounts are in stroops (1 XLM = 10_000_000 stroops).
        initialize_sponsorship(&env, &admin, 50_000_000).unwrap();
        let cfg = get_config(&env).unwrap();
        println!(
            "Pool funded: {} stroops, {} per scan, daily cap {}",
            get_fund_balance(&env),
            cfg.sponsor_amount,
            cfg.daily_cap
        );

        // ── Step 2: Tune the caps (admin, optional) ──────────────────────
        // min_threshold, sponsor_amount, daily_cap, per_user_cap, per_user_daily_cap
        update_config(&env, &admin, 10_000_000, 100_000, 500, 1_000_000, 3).unwrap();

        // ── Step 3: Player must have a verified profile ──────────────────
        // In the full game, `player_profile` does this when the profile is
        // created. Here we mark it directly.
        mark_profile_verified(&env, &newcomer);

        // ── Step 4: Sponsor the first scan ───────────────────────────────
        let paid = sponsor_first_scan(&env, &newcomer).unwrap();
        println!("Sponsored first scan: {paid} stroops");
        println!(
            "Pool now {} | today {} used, {} left",
            get_fund_balance(&env),
            get_daily_count(&env),
            get_remaining_daily_slots(&env)
        );
        assert!(has_been_sponsored(&env, &newcomer));

        // ── Step 5: One-time only ────────────────────────────────────────
        // The UI should hide the "free scan" button once this is true.
        assert_eq!(
            sponsor_first_scan(&env, &newcomer),
            Err(SponsorError::AlreadySponsored)
        );
        println!("Second request rejected: AlreadySponsored (code 1)");
    });
}
