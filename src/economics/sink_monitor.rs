//! Resource Sink Monitoring & Equilibrium Tracking Module.
//! Monitors total resources created vs destroyed across crafting, repairs, recycling, and tournaments.

use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EconomicSinkMetrics {
    pub total_created: u128,
    pub total_destroyed: u128,
    pub crafting_destroyed: u128,
    pub repair_destroyed: u128,
    pub recycling_destroyed: u128,
    pub tournament_destroyed: u128,
    pub guild_destroyed: u128,
    pub equilibrium_index_bps: u32, // Ratio in basis points (10000 = 1.00)
}

pub fn get_sink_metrics(env: &Env) -> EconomicSinkMetrics {
    let key = symbol_short!("SINK_MET");
    env.storage().instance().get(&key).unwrap_or(EconomicSinkMetrics {
        total_created: 0,
        total_destroyed: 0,
        crafting_destroyed: 0,
        repair_destroyed: 0,
        recycling_destroyed: 0,
        tournament_destroyed: 0,
        guild_destroyed: 0,
        equilibrium_index_bps: 10000,
    })
}

pub fn record_resource_creation(env: &Env, amount: u128) {
    let mut metrics = get_sink_metrics(env);
    metrics.total_created = metrics.total_created.saturating_add(amount);
    update_equilibrium(&mut metrics);
    let key = symbol_short!("SINK_MET");
    env.storage().instance().set(&key, &metrics);
}

pub fn record_resource_destruction(env: &Env, category: Symbol, amount: u128) {
    let mut metrics = get_sink_metrics(env);
    metrics.total_destroyed = metrics.total_destroyed.saturating_add(amount);

    if category == symbol_short!("CRAFT") {
        metrics.crafting_destroyed = metrics.crafting_destroyed.saturating_add(amount);
    } else if category == symbol_short!("REPAIR") {
        metrics.repair_destroyed = metrics.repair_destroyed.saturating_add(amount);
    } else if category == symbol_short!("RECYCLE") {
        metrics.recycling_destroyed = metrics.recycling_destroyed.saturating_add(amount);
    } else if category == symbol_short!("TOURN") {
        metrics.tournament_destroyed = metrics.tournament_destroyed.saturating_add(amount);
    } else if category == symbol_short!("GUILD") {
        metrics.guild_destroyed = metrics.guild_destroyed.saturating_add(amount);
    }

    update_equilibrium(&mut metrics);
    let key = symbol_short!("SINK_MET");
    env.storage().instance().set(&key, &metrics);
}

fn update_equilibrium(metrics: &mut EconomicSinkMetrics) {
    if metrics.total_created == 0 {
        metrics.equilibrium_index_bps = 10000;
    } else {
        metrics.equilibrium_index_bps = ((metrics.total_destroyed * 10000) / metrics.total_created) as u32;
    }
}

pub fn is_equilibrium_healthy(env: &Env) -> bool {
    let metrics = get_sink_metrics(env);
    metrics.equilibrium_index_bps >= 9000 && metrics.equilibrium_index_bps <= 11000
}
