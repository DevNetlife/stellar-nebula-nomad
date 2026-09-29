//! Gas regression coverage for reducing event emissions / payload sizes.
//!
//! Every CPU-instruction figure below is a **baseline recorded before the
//! change** under the same conditions (measured inside a contract invocation
//! with `env.cost_estimate().budget().reset_default()`), so these assertions
//! fail if a future edit puts the event tax back on a read path.
//!
//! | flow | before | after | delta |
//! |---|---|---|---|
//! | `calculate_difficulty` (pure read) | 3606 | 0 | −100% |
//! | `check_permission` success | 54023 | 50359 | −6.8% |
//! | `get_cached_with_ttl` expired | 23802 | 20196 | −15.1% |
//! | `list_ship` (control, 1 event kept) | 50105 | 50105 | unchanged |
//! | `cache_with_ttl` (control, 1 event kept) | 23349 | 23349 | unchanged |
//! | `rollover_season` | 2 events | 1 event | −2.7% |
//! | `repair_ship` | 2 events | 1 event | −2% |
//! | `emit_standard_event` (framework) | 76118 | 44320 | −41.8% |
//! | `emit_standard_event_burst` (15 payloads) | 605759 / 15 events | 433705 / 1 event | −28.4%, −14 events |

use crate::nebula_gen::{NebulaGen, NebulaGenClient};
use crate::test_helpers::event_count;
use crate::{
    access_control, cache_ttl_manager, difficulty_scaler, event_framework, nft_marketplace, seasons,
};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env};

#[contract]
struct Stub;

#[contractimpl]
impl Stub {}

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract = env.register(Stub, ());
    (env, contract)
}

/// Run `f` inside the contract, returning `(cpu instructions, events emitted)`.
///
/// `Env::events().all()` reports the events of the *last* invocation, so the
/// count is read once, after `f` returns, rather than as a delta.
fn run(env: &Env, contract: &Address, name: &str, f: impl FnOnce()) -> (u64, usize) {
    let mut cost = 0u64;
    env.as_contract(contract, || {
        let mut budget = env.cost_estimate().budget();
        budget.reset_default();
        f();
        cost = budget.cpu_instruction_cost();
    });
    let events = event_count(env);
    std::println!("GAS {name}: {cost} instructions, {events} event(s)");
    (cost, events)
}

/// A pure calculation must not pay the event tax: the `diff/adjust`
/// emission *was* the entire host cost (3606 → 0; local arithmetic is free).
#[test]
fn calculate_difficulty_pays_no_event_tax() {
    let (env, contract) = setup();
    let (cost, events) = run(&env, &contract, "calculate_difficulty", || {
        let _ = difficulty_scaler::calculate_difficulty(&env, 40);
    });
    assert_eq!(events, 0, "a pure read must not emit");
    assert!(
        cost <= 3245,
        "calculate_difficulty cost {cost} > 90% of baseline 3606"
    );
}

/// Successful authorization used to emit `rbac/perm_ok` on every check; only
/// the denial path stays auditable (54023 → 50359, −6.8%).
#[test]
fn permission_success_emits_nothing_and_costs_less() {
    let (env, contract) = setup();
    let admin = Address::generate(&env);
    let action = symbol_short!("act");
    // One authorisation per contract frame: an address may authorise only
    // once inside a single `as_contract` block.
    env.as_contract(&contract, || {
        access_control::init_roles(&env, admin.clone()).unwrap()
    });
    env.as_contract(&contract, || {
        access_control::grant_permission(
            &env,
            admin.clone(),
            access_control::admin_role(),
            action.clone(),
        )
        .unwrap()
    });

    let (cost, events) = run(&env, &contract, "check_permission_ok", || {
        assert!(access_control::check_permission(&env, &admin, &action).is_ok());
    });
    assert_eq!(events, 0, "granted permission must not emit");
    assert!(
        cost <= 51321,
        "check_permission ok cost {cost} > 95% of baseline 54023"
    );

    // Denials keep their audit event.
    let stranger = Address::generate(&env);
    let (_, fail_events) = run(&env, &contract, "check_permission_fail", || {
        assert!(access_control::check_permission(&env, &stranger, &action).is_err());
    });
    assert_eq!(fail_events, 1, "denied permission must still emit");
}

/// An expired cache read marks the entry stale and returns `Err`; the event
/// that duplicated both signals is gone (23802 → 20196, −15.1%).
#[test]
fn cache_expiry_read_emits_nothing_and_costs_less() {
    let (env, contract) = setup();
    run(&env, &contract, "cache_with_ttl", || {
        cache_ttl_manager::cache_with_ttl(
            &env,
            symbol_short!("ns"),
            symbol_short!("key"),
            soroban_sdk::Bytes::from_slice(&env, &[1u8; 16]),
            60,
        )
        .unwrap();
    });
    env.ledger().with_mut(|l| l.timestamp += 10_000);

    let (cost, events) = run(&env, &contract, "get_cached_with_ttl_expired", || {
        assert!(cache_ttl_manager::get_cached_with_ttl(
            &env,
            symbol_short!("ns"),
            symbol_short!("key"),
        )
        .is_err());
    });
    assert_eq!(events, 0, "expiry must not emit on a read path");
    assert!(
        cost <= 22611,
        "expired cache read cost {cost} > 95% of baseline 23802"
    );
}

/// A season rollover archives the old season and starts the new one in the
/// same call: one `season/rolled` event replaces `ended` + `started`.
///
/// The removed event was worth ~2600 instructions (~2.7% of this flow), so
/// the emission count is the meaningful assertion here; the cost ceiling only
/// guards against regressions.
#[test]
fn season_rollover_emits_one_event_not_two() {
    let (env, contract) = setup();
    let admin = Address::generate(&env);
    env.as_contract(&contract, || {
        seasons::initialize_season(&env, &admin, soroban_sdk::String::from_str(&env, "S1"))
            .unwrap();
    });
    // Rollover is only legal once the current season has ended.
    env.ledger()
        .with_mut(|l| l.timestamp += seasons::SEASON_DURATION_SECS + 1);

    let (cost, events) = run(&env, &contract, "rollover_season", || {
        assert!(seasons::rollover_season(
            &env,
            &admin,
            soroban_sdk::String::from_str(&env, "S2"),
            &soroban_sdk::Vec::new(&env),
        )
        .is_ok());
    });
    assert_eq!(events, 1, "rollover must emit exactly one event");
    assert!(
        cost <= 95_000,
        "rollover_season cost {cost} regressed past 95000"
    );
}

/// Reading an expired layout removes it and returns `None`; the
/// `neb_gen/expired` notification was a third copy of that signal.
#[test]
fn expired_layout_read_emits_nothing() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| {
        l.sequence_number = 1;
        l.timestamp = 1_000;
    });
    let id = env.register(NebulaGen, ());
    let client = NebulaGenClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.init(&admin, &5u32, &1u32, &10u32, &100u64);
    let caller = Address::generate(&env);
    let seed = BytesN::from_array(&env, &[1u8; 32]);
    client.generate_validated_nebula_layout(&caller, &7u64, &1u64, &seed);
    env.ledger().with_mut(|l| l.timestamp += 101);

    let mut budget = env.cost_estimate().budget();
    budget.reset_default();
    assert!(client.get_layout(&7u64).is_none());
    let cost = budget.cpu_instruction_cost();

    std::println!("GAS get_layout_expired: {cost} instructions, 0 event(s)");
    assert_eq!(event_count(&env), 0, "expiry must not emit on a read path");
    assert!(
        cost < 94_789,
        "expired layout read cost {cost} ≥ baseline 94789"
    );
}

/// Control: a state-changing flow keeps emitting, and the belt-and-braces
/// changes must not have made it any more expensive than baseline 50105.
#[test]
fn marketplace_listing_still_emits_once() {
    let (env, contract) = setup();
    let seller = Address::generate(&env);
    let (cost, events) = run(&env, &contract, "list_ship", || {
        nft_marketplace::list_ship(&env, &seller, 1, 500).unwrap();
    });
    assert_eq!(events, 1, "listing must still emit");
    assert!(
        cost <= 50_105,
        "list_ship cost {cost} regressed past baseline 50105"
    );
}

/// A repaired ship carries payment + durability in one event instead of the
/// old `paid` + `durbl` pair.
#[test]
fn repair_ship_emits_one_combined_event() {
    use crate::resource_minter::ResourceKey;
    use crate::ship_nft::{DataKey, ShipNft};

    let (env, contract) = setup();
    let player = Address::generate(&env);
    let ship_id = 1u64;

    env.as_contract(&contract, || {
        let ship = ShipNft {
            id: ship_id,
            owner: player.clone(),
            ship_type: symbol_short!("explorer"),
            hull: 100,
            scanner_power: 50,
            durability: 60,
            max_durability: 100,
            metadata: soroban_sdk::Bytes::new(&env),
            metadata_uri: soroban_sdk::Bytes::new(&env),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Ship(ship_id), &ship);
        env.storage().instance().set(
            &ResourceKey::ResourceBalance(player.clone(), symbol_short!("dust")),
            &1_000u32,
        );
    });

    let (cost, events) = run(&env, &contract, "repair_ship", || {
        let receipt =
            crate::ship_repair::repair_ship(&env, &player, ship_id, symbol_short!("dust"), false)
                .unwrap();
        assert_eq!(receipt.durability_before, 60);
        assert_eq!(receipt.durability_after, 100);
    });
    assert_eq!(events, 1, "repair must emit exactly one combined event");
    assert!(cost > 0, "sanity");

    assert!(
        crate::test_helpers::has_event_topics(&env, &["ship_rep", "repaired"]),
        "expected a ship_rep/repaired event"
    );
}

// ─── standardized event framework ────────────────────────────────────────────

/// Seed the framework's default schemas in its own invocation, so the
/// measured emission below pays nothing for setup.
fn init_framework(env: &Env, contract: &Address) {
    let admin = Address::generate(env);
    env.as_contract(contract, || {
        event_framework::init_event_framework(env, &admin)
    });
}

/// One record, one event, one topic — and only the unavoidable work left.
///
/// Baseline (persistent tier for the schema lookup and for both emit
/// counters): **76,118** instructions. Schema versions and counters now live
/// in instance storage, so the record write and the publish are all that
/// remains of that cost.
#[test]
fn standard_event_emit_keeps_only_the_record_write_and_publish() {
    let (env, contract) = setup();
    init_framework(&env, &contract);
    let caller = Address::generate(&env);
    let payload = BytesN::from_array(&env, &[7u8; 256]);

    let (cost, events) = run(&env, &contract, "emit_standard_event", || {
        let _ = event_framework::emit_standard_event(
            &env,
            &caller,
            symbol_short!("system"),
            payload.clone(),
        );
    });

    assert_eq!(events, 1, "one record must publish exactly one event");
    let topics = crate::test_helpers::event_topics(&env, 0).expect("event topics");
    assert_eq!(
        topics,
        std::vec![std::string::String::from("system")],
        "standard events go through events::emit: one topic, no wrapper symbol"
    );
    assert!(
        cost <= 48_000,
        "emit_standard_event cost {cost} above ceiling 48000 (baseline 76118)"
    );
}

/// A burst stores every payload as its own record but publishes a **single**
/// batched event, and advances both counters with one instance write for the
/// whole batch. Baseline: **605,759** instructions across **15** events.
#[test]
fn burst_stores_every_payload_but_publishes_one_event() {
    let (env, contract) = setup();
    init_framework(&env, &contract);
    let caller = Address::generate(&env);
    let payload = BytesN::from_array(&env, &[7u8; 256]);
    let mut payloads = soroban_sdk::Vec::new(&env);
    for _ in 0..15 {
        payloads.push_back(payload.clone());
    }

    let (cost, events) = run(&env, &contract, "emit_standard_event_burst", || {
        let emitted = event_framework::emit_standard_event_burst(
            &env,
            &caller,
            symbol_short!("system"),
            payloads.clone(),
        )
        .ok();
        assert_eq!(emitted, Some(15));
    });

    assert_eq!(events, 1, "15 payloads are one operation: one event");

    // Every payload is still an individually addressable record, and the ids
    // stay consecutive so consumers can derive them from the batch's first id.
    env.as_contract(&contract, || {
        let records = event_framework::query_recent_events(&env, &symbol_short!("all"), 20);
        assert_eq!(records.len(), 15, "one record per payload");
        let newest = records.get(0).unwrap().id;
        let oldest = records.get(14).unwrap().id;
        assert_eq!(newest - oldest, 14, "ids stay consecutive across the batch");
        assert_eq!(records.get(0).unwrap().payload, payload);
    });

    assert!(
        cost <= 475_000,
        "burst cost {cost} above ceiling 475000 (baseline 605759)"
    );
}

/// An unregistered type fails before anything is written or published: no
/// record, no counter bump, no event.
#[test]
fn unknown_event_type_emits_nothing() {
    let (env, contract) = setup();
    init_framework(&env, &contract);
    let caller = Address::generate(&env);
    let payload = BytesN::from_array(&env, &[7u8; 256]);

    let (_, events) = run(&env, &contract, "emit_standard_event_unknown_type", || {
        assert!(event_framework::emit_standard_event(
            &env,
            &caller,
            symbol_short!("nope"),
            payload.clone(),
        )
        .is_err());
    });
    assert_eq!(events, 0, "a rejected emission must not publish");
}

/// Counters written by the previous persistent layout are carried over the
/// first time they are read, so record indices — and the ids already handed
/// out — do not restart at zero.
#[test]
fn legacy_persistent_counters_are_migrated_not_reset() {
    let (env, contract) = setup();
    let admin = Address::generate(&env);
    let caller = Address::generate(&env);
    let payload = BytesN::from_array(&env, &[7u8; 256]);

    // State as the old layout left it: 7 records, 3 of them in this ledger.
    let timestamp = env.as_contract(&contract, || {
        let ts = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&symbol_short!("ev_idx"), &7u64);
        env.storage()
            .persistent()
            .set(&(symbol_short!("ev_tsq"), ts), &3u64);
        ts
    });
    env.as_contract(&contract, || {
        event_framework::init_event_framework(&env, &admin)
    });

    let id = env
        .as_contract(&contract, || {
            event_framework::emit_standard_event(
                &env,
                &caller,
                symbol_short!("system"),
                payload.clone(),
            )
            .ok()
        })
        .expect("emit");
    assert_eq!(
        id,
        timestamp.saturating_mul(1_000_000).saturating_add(4),
        "the 8th record continues the legacy sequence instead of restarting"
    );

    env.as_contract(&contract, || {
        let records = event_framework::query_recent_events(&env, &symbol_short!("all"), 20);
        assert_eq!(records.len(), 1, "the record lands at index 8, not index 1");
        assert!(
            !env.storage().persistent().has(&symbol_short!("ev_idx")),
            "the superseded counter entry is dropped, not left paying rent"
        );
    });
}
