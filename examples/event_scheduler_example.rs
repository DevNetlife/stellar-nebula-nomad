#![cfg(test)]

//! Event Scheduler Usage Examples
//!
//! This file demonstrates various ways to use the event scheduler
//! for community engagement and automated event management.

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use stellar_nebula_nomad::{EventError, WEEKLY_FESTIVAL_INTERVAL};

/// Example 1: Schedule a Weekly Nebula Festival
///
/// This is the most common use case - scheduling recurring weekly festivals
/// that bring the community together with large reward pools.
#[test]
fn example_weekly_festival() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    // Initialize the scheduler
    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    // Schedule a weekly festival with 100,000 reward pool
    let event_id = stellar_nebula_nomad::NebulaNomadContract::schedule_weekly_festival(
        env.clone(),
        admin.clone(),
        100_000i128,
    )
    .unwrap();

    println!("✅ Weekly festival scheduled with ID: {}", event_id);

    let event =
        stellar_nebula_nomad::NebulaNomadContract::get_event(env.clone(), event_id).unwrap();
    println!("   Event Type: {:?}", event.event_type);
    println!("   Start Time: {}", event.start_time);
    println!("   Reward Pool: {}", event.reward_pool);
}

/// Example 2: Schedule Multiple Event Types
///
/// Demonstrates scheduling different event types throughout the week
/// to maintain player engagement.
#[test]
fn example_weekly_event_calendar() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();
    let one_day = 24 * 60 * 60u64;

    // Monday: PvP Tournament
    let pvp_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("pvp"),
        current_time + one_day,
        50_000i128,
    )
    .unwrap();

    // Wednesday: Raid Boss Event
    let raid_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("raid"),
        current_time + (3 * one_day),
        75_000i128,
    )
    .unwrap();

    // Friday: Harvest Competition
    let harvest_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("harvest"),
        current_time + (5 * one_day),
        40_000i128,
    )
    .unwrap();

    // Sunday: Exploration Challenge
    let explore_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("explore"),
        current_time + (7 * one_day),
        30_000i128,
    )
    .unwrap();

    println!("✅ Weekly event calendar created:");
    println!("   Monday (PvP): Event #{}", pvp_id);
    println!("   Wednesday (Raid): Event #{}", raid_id);
    println!("   Friday (Harvest): Event #{}", harvest_id);
    println!("   Sunday (Explore): Event #{}", explore_id);

    let active = stellar_nebula_nomad::NebulaNomadContract::get_active_events(env.clone());
    assert_eq!(active.len(), 4);
}

/// Example 3: Event Lifecycle with Participant Tracking
///
/// Shows the complete lifecycle of an event from scheduling to execution,
/// including participant tracking.
#[test]
fn example_event_lifecycle_with_participants() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();
    let event_start = current_time + 3600; // 1 hour from now

    // 1. Schedule the event
    let event_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("raid"),
        event_start,
        50_000i128,
    )
    .unwrap();

    println!("✅ Event scheduled: #{}", event_id);

    // 2. Simulate players joining over time
    // After 10 minutes: 25 players
    stellar_nebula_nomad::NebulaNomadContract::update_event_participants(env.clone(), event_id, 25)
        .unwrap();
    println!("   📊 Participants: 25");

    // After 30 minutes: 75 players
    stellar_nebula_nomad::NebulaNomadContract::update_event_participants(env.clone(), event_id, 75)
        .unwrap();
    println!("   📊 Participants: 75");

    // After 50 minutes: 150 players
    stellar_nebula_nomad::NebulaNomadContract::update_event_participants(
        env.clone(),
        event_id,
        150,
    )
    .unwrap();
    println!("   📊 Participants: 150");

    // 3. Fast forward to event start time
    env.ledger().with_mut(|li| {
        li.timestamp = event_start + 1;
    });

    // 4. Trigger the event
    let result =
        stellar_nebula_nomad::NebulaNomadContract::trigger_scheduled_event(env.clone(), event_id)
            .unwrap();

    println!("✅ Event executed!");
    println!("   💰 Rewards Distributed: {}", result.rewards_distributed);
    println!("   👥 Final Participants: {}", result.participants);
    println!("   ⏰ Executed At: {}", result.executed_at);
}

/// Example 4: Managing Active Events
///
/// Demonstrates querying and managing multiple active events.
#[test]
fn example_managing_active_events() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();

    // Schedule 3 events
    for i in 1..=3 {
        stellar_nebula_nomad::NebulaNomadContract::schedule_event(
            env.clone(),
            admin.clone(),
            symbol_short!("festival"),
            current_time + (i * 1000),
            (i as i128) * 10_000,
        )
        .unwrap();
    }

    // Get all active events
    let active = stellar_nebula_nomad::NebulaNomadContract::get_active_events(env.clone());
    println!("✅ Active Events: {}", active.len());

    // Display details for each event
    for i in 0..active.len() {
        let event_id = active.get(i).unwrap();
        let event =
            stellar_nebula_nomad::NebulaNomadContract::get_event(env.clone(), event_id).unwrap();

        println!(
            "   Event #{}: {:?} at {} with {} rewards",
            event.event_id, event.event_type, event.start_time, event.reward_pool
        );
    }

    // Cancel the second event
    stellar_nebula_nomad::NebulaNomadContract::cancel_event(env.clone(), admin.clone(), 2).unwrap();

    let active_after = stellar_nebula_nomad::NebulaNomadContract::get_active_events(env.clone());
    println!(
        "✅ Active Events After Cancellation: {}",
        active_after.len()
    );
}

/// Example 5: Seasonal Event Series
///
/// Shows how to schedule a series of themed events for a special season.
#[test]
fn example_seasonal_event_series() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();
    let one_day = 24 * 60 * 60u64;

    println!("🎉 Scheduling 'Cosmic Convergence' Event Series:");

    // Week 1: Opening Festival
    let week1 = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("festival"),
        current_time + one_day,
        200_000i128,
    )
    .unwrap();
    println!("   Week 1 - Opening Festival: Event #{}", week1);

    // Week 2: Raid Marathon
    let week2 = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("raid"),
        current_time + (7 * one_day),
        150_000i128,
    )
    .unwrap();
    println!("   Week 2 - Raid Marathon: Event #{}", week2);

    // Week 3: PvP Championship
    let week3 = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("pvp"),
        current_time + (14 * one_day),
        250_000i128,
    )
    .unwrap();
    println!("   Week 3 - PvP Championship: Event #{}", week3);

    // Week 4: Grand Finale
    let week4 = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("festival"),
        current_time + (21 * one_day),
        500_000i128,
    )
    .unwrap();
    println!("   Week 4 - Grand Finale: Event #{}", week4);

    let total_rewards = 200_000 + 150_000 + 250_000 + 500_000;
    println!("   💰 Total Season Rewards: {}", total_rewards);
}

/// Example 6: Error Handling
///
/// Demonstrates proper error handling when scheduling events.
#[test]
fn example_error_handling() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();

    // Try to schedule event in the past
    let past_result = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("festival"),
        current_time - 1000,
        10_000i128,
    );

    match past_result {
        Err(EventError::EventAlreadyPassed) => {
            println!("✅ Correctly rejected past event");
        }
        _ => panic!("Should have rejected past event"),
    }

    // Try to schedule with invalid event type
    let invalid_result = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("invalid"),
        current_time + 1000,
        10_000i128,
    );

    match invalid_result {
        Err(EventError::InvalidEventType) => {
            println!("✅ Correctly rejected invalid event type");
        }
        _ => panic!("Should have rejected invalid type"),
    }

    // Try to trigger event too early
    let event_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
        env.clone(),
        admin.clone(),
        symbol_short!("raid"),
        current_time + 10000,
        10_000i128,
    )
    .unwrap();

    let early_result =
        stellar_nebula_nomad::NebulaNomadContract::trigger_scheduled_event(env.clone(), event_id);

    match early_result {
        Err(EventError::EventNotReady) => {
            println!("✅ Correctly rejected early trigger");
        }
        _ => panic!("Should have rejected early trigger"),
    }
}

/// Example 7: High-Frequency Event Schedule
///
/// Demonstrates scheduling multiple events in quick succession
/// for high-engagement periods.
#[test]
fn example_high_frequency_schedule() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    stellar_nebula_nomad::NebulaNomadContract::initialize_scheduler(env.clone(), admin.clone());

    let current_time = env.ledger().timestamp();
    let one_hour = 60 * 60u64;

    println!("⚡ Scheduling High-Frequency Event Day:");

    let event_types = [
        symbol_short!("harvest"),
        symbol_short!("pvp"),
        symbol_short!("raid"),
        symbol_short!("explore"),
        symbol_short!("harvest"),
        symbol_short!("pvp"),
    ];

    for (i, event_type) in event_types.iter().enumerate() {
        let event_id = stellar_nebula_nomad::NebulaNomadContract::schedule_event(
            env.clone(),
            admin.clone(),
            event_type.clone(),
            current_time + ((i as u64 + 1) * one_hour * 2),
            20_000i128,
        )
        .unwrap();

        println!(
            "   Hour {}: {:?} - Event #{}",
            (i + 1) * 2,
            event_type,
            event_id
        );
    }

    let active = stellar_nebula_nomad::NebulaNomadContract::get_active_events(env.clone());
    println!("   📅 Total Events Scheduled: {}", active.len());
}
