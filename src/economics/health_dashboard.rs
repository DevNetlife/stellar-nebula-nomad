//! Real-time economy health dashboard with supply/demand tracking and alert thresholds.
//!
//! Monitors economic indicators including supply metrics, inflation rates, and resource balance
//! to enable proactive management of game economy.

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

// ── Data Types ────────────────────────────────────────────────────────────────

/// Supply and demand metrics for the game economy.
#[contracttype]
#[derive(Clone)]
pub struct SupplyDemandMetrics {
    /// Current total circulating supply.
    pub total_supply: i128,
    /// Supply available on market.
    pub market_supply: i128,
    /// Demand indicator (buy orders).
    pub market_demand: i128,
    /// Supply/demand ratio (in basis points, target = 10000 = 1:1).
    pub supply_demand_ratio: i32,
    /// Timestamp of last update.
    pub last_update: u64,
}

/// Inflation and deflation tracking.
#[contracttype]
#[derive(Clone)]
pub struct InflationMetrics {
    /// Current annual inflation rate in basis points.
    pub annual_inflation_bps: u32,
    /// Cumulative inflation this epoch.
    pub epoch_inflation: i128,
    /// Total burn amount this epoch.
    pub epoch_deflation: i128,
    /// Net change in supply this epoch.
    pub net_supply_change: i128,
    /// Timestamp of last update.
    pub last_update: u64,
}

/// Alert configuration and thresholds.
#[contracttype]
#[derive(Clone)]
pub struct AlertThreshold {
    /// Threshold name.
    pub alert_name: Symbol,
    /// Upper bound threshold.
    pub upper_bound: i128,
    /// Lower bound threshold.
    pub lower_bound: i128,
    /// Is this alert active?
    pub is_active: bool,
    /// Metric type this monitors.
    pub metric_type: Symbol,
}

/// Alert event triggered when threshold is exceeded.
#[contracttype]
#[derive(Clone)]
pub struct AlertEvent {
    pub alert_name: Symbol,
    pub metric_value: i128,
    pub threshold: i128,
    pub alert_type: Symbol, // "upper" or "lower"
    pub timestamp: u64,
}

/// Health status of the economy.
#[contracttype]
#[derive(Clone, Copy, Debug)]
pub enum EconomyHealth {
    /// All metrics healthy.
    Healthy = 1,
    /// Some warnings but no critical issues.
    Warning = 2,
    /// Critical issues requiring intervention.
    Critical = 3,
}

/// Overall dashboard snapshot.
#[contracttype]
#[derive(Clone)]
pub struct EconomyHealthDashboard {
    pub supply_demand: SupplyDemandMetrics,
    pub inflation: InflationMetrics,
    pub health_status: EconomyHealth,
    pub active_alerts: u32,
    pub last_calculated: u64,
}

// ── Storage Keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DashboardKey {
    SupplyDemand,
    InflationMetrics,
    EconomyHealth,
    AlertThreshold(Symbol),
    AlertHistory,
    HealthCheckInterval,
}

// ── Write Helpers ─────────────────────────────────────────────────────────────

/// Initialize the economy health dashboard with default thresholds.
pub fn initialize_dashboard(env: &Env, admin: &Address) {
    admin.require_auth();

    let metrics = SupplyDemandMetrics {
        total_supply: 0,
        market_supply: 0,
        market_demand: 0,
        supply_demand_ratio: 10000,
        last_update: env.ledger().timestamp(),
    };

    let inflation = InflationMetrics {
        annual_inflation_bps: 500, // 5% annual
        epoch_inflation: 0,
        epoch_deflation: 0,
        net_supply_change: 0,
        last_update: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DashboardKey::SupplyDemand, &metrics);
    env.storage()
        .persistent()
        .set(&DashboardKey::InflationMetrics, &inflation);

    // Set default alert thresholds
    set_alert_threshold(
        env,
        admin,
        symbol_short!("inflation"),
        1000, // Upper: 10% inflation
        0,    // Lower: 0%
    );

    set_alert_threshold(
        env,
        admin,
        symbol_short!("deflation"),
        0,    // No lower bound
        -500, // Lower: -5% deflation
    );

    set_alert_threshold(
        env,
        admin,
        symbol_short!("supply"),
        0,             // No upper bound
        1_000_000_000, // Lower: 1B minimum supply
    );

    env.events()
        .publish((symbol_short!("dash"), symbol_short!("init")), ());
}

/// Update supply and demand metrics and check thresholds.
pub fn update_supply_demand(
    env: &Env,
    total_supply: i128,
    market_supply: i128,
    market_demand: i128,
) {
    let supply_demand_ratio = if market_supply > 0 {
        ((market_demand as i128 * 10000) / market_supply as i128) as i32
    } else {
        10000
    };

    let metrics = SupplyDemandMetrics {
        total_supply,
        market_supply,
        market_demand,
        supply_demand_ratio,
        last_update: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DashboardKey::SupplyDemand, &metrics);

    // Check supply thresholds
    check_alert_threshold(env, symbol_short!("supply"), total_supply);

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("supply")),
        (total_supply, market_supply, market_demand),
    );
}

/// Update inflation/deflation metrics and check thresholds.
pub fn update_inflation_metrics(env: &Env, minted_amount: i128, burned_amount: i128) {
    let mut inflation: InflationMetrics = env
        .storage()
        .persistent()
        .get(&DashboardKey::InflationMetrics)
        .unwrap_or(InflationMetrics {
            annual_inflation_bps: 500,
            epoch_inflation: 0,
            epoch_deflation: 0,
            net_supply_change: 0,
            last_update: 0,
        });

    inflation.epoch_inflation = inflation.epoch_inflation.saturating_add(minted_amount);
    inflation.epoch_deflation = inflation.epoch_deflation.saturating_add(burned_amount);
    inflation.net_supply_change = minted_amount.saturating_sub(burned_amount);
    inflation.last_update = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DashboardKey::InflationMetrics, &inflation);

    // Check inflation thresholds
    check_alert_threshold(env, symbol_short!("inflation"), minted_amount);

    check_alert_threshold(
        env,
        symbol_short!("deflation"),
        burned_amount.wrapping_neg(),
    );

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("inflation")),
        (minted_amount, burned_amount),
    );
}

/// Set or update an alert threshold.
pub fn set_alert_threshold(
    env: &Env,
    admin: &Address,
    alert_name: Symbol,
    upper_bound: i128,
    lower_bound: i128,
) {
    admin.require_auth();

    let threshold = AlertThreshold {
        alert_name: alert_name.clone(),
        upper_bound,
        lower_bound,
        is_active: true,
        metric_type: alert_name.clone(),
    };

    env.storage().persistent().set(
        &DashboardKey::AlertThreshold(alert_name.clone()),
        &threshold,
    );

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("threshold")),
        (alert_name, upper_bound, lower_bound),
    );
}

/// Deactivate an alert threshold.
pub fn deactivate_alert(env: &Env, admin: &Address, alert_name: Symbol) {
    admin.require_auth();

    let mut threshold: AlertThreshold = env
        .storage()
        .persistent()
        .get(&DashboardKey::AlertThreshold(alert_name.clone()))
        .unwrap_or(AlertThreshold {
            alert_name: alert_name.clone(),
            upper_bound: 0,
            lower_bound: 0,
            is_active: false,
            metric_type: alert_name.clone(),
        });

    threshold.is_active = false;

    env.storage().persistent().set(
        &DashboardKey::AlertThreshold(alert_name.clone()),
        &threshold,
    );

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("deactiv8")),
        (alert_name,),
    );
}

/// Check if a metric value triggers any alerts.
fn check_alert_threshold(env: &Env, alert_name: Symbol, metric_value: i128) {
    let threshold: AlertThreshold = match env
        .storage()
        .persistent()
        .get(&DashboardKey::AlertThreshold(alert_name.clone()))
    {
        Some(t) => t,
        None => return,
    };

    if !threshold.is_active {
        return;
    }

    // Check upper bound
    if threshold.upper_bound > 0 && metric_value > threshold.upper_bound {
        let alert = AlertEvent {
            alert_name: alert_name.clone(),
            metric_value,
            threshold: threshold.upper_bound,
            alert_type: symbol_short!("upper"),
            timestamp: env.ledger().timestamp(),
        };

        record_alert(env, alert);
    }

    // Check lower bound
    if threshold.lower_bound > 0 && metric_value < threshold.lower_bound {
        let alert = AlertEvent {
            alert_name,
            metric_value,
            threshold: threshold.lower_bound,
            alert_type: symbol_short!("lower"),
            timestamp: env.ledger().timestamp(),
        };

        record_alert(env, alert);
    }
}

/// Record an alert event.
fn record_alert(env: &Env, alert: AlertEvent) {
    let mut history: Vec<AlertEvent> = env
        .storage()
        .persistent()
        .get(&DashboardKey::AlertHistory)
        .unwrap_or_else(|| Vec::new(env));

    // Keep last 100 alerts
    if history.len() >= 100 {
        let mut new_history: Vec<AlertEvent> = Vec::new(env);
        for i in 1..history.len() {
            if let Some(a) = history.get(i as u32) {
                new_history.push_back(a.clone());
            }
        }
        history = new_history;
    }

    history.push_back(alert.clone());
    env.storage()
        .persistent()
        .set(&DashboardKey::AlertHistory, &history);

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("alert")),
        (alert.alert_name, alert.metric_value, alert.alert_type),
    );
}

/// Recalculate overall economy health status.
pub fn recalculate_health(env: &Env) {
    let supply_demand: SupplyDemandMetrics = env
        .storage()
        .persistent()
        .get(&DashboardKey::SupplyDemand)
        .unwrap_or(SupplyDemandMetrics {
            total_supply: 0,
            market_supply: 0,
            market_demand: 0,
            supply_demand_ratio: 10000,
            last_update: 0,
        });

    let inflation: InflationMetrics = env
        .storage()
        .persistent()
        .get(&DashboardKey::InflationMetrics)
        .unwrap_or(InflationMetrics {
            annual_inflation_bps: 500,
            epoch_inflation: 0,
            epoch_deflation: 0,
            net_supply_change: 0,
            last_update: 0,
        });

    let alerts: Vec<AlertEvent> = env
        .storage()
        .persistent()
        .get(&DashboardKey::AlertHistory)
        .unwrap_or_else(|| Vec::new(env));

    // Calculate health status based on recent alerts and metrics
    let health_status = if alerts.len() > 3 {
        EconomyHealth::Critical
    } else if alerts.len() > 1 {
        EconomyHealth::Warning
    } else {
        EconomyHealth::Healthy
    };

    let dashboard = EconomyHealthDashboard {
        supply_demand,
        inflation,
        health_status,
        active_alerts: alerts.len() as u32,
        last_calculated: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DashboardKey::EconomyHealth, &dashboard);

    env.events().publish(
        (symbol_short!("dash"), symbol_short!("health")),
        (health_status as u32, alerts.len()),
    );
}

// ── Read-Only View Functions ──────────────────────────────────────────────────

/// Get current supply and demand metrics.
pub fn get_supply_demand(env: &Env) -> SupplyDemandMetrics {
    env.storage()
        .persistent()
        .get(&DashboardKey::SupplyDemand)
        .unwrap_or(SupplyDemandMetrics {
            total_supply: 0,
            market_supply: 0,
            market_demand: 0,
            supply_demand_ratio: 10000,
            last_update: 0,
        })
}

/// Get current inflation metrics.
pub fn get_inflation_metrics(env: &Env) -> InflationMetrics {
    env.storage()
        .persistent()
        .get(&DashboardKey::InflationMetrics)
        .unwrap_or(InflationMetrics {
            annual_inflation_bps: 500,
            epoch_inflation: 0,
            epoch_deflation: 0,
            net_supply_change: 0,
            last_update: 0,
        })
}

/// Get overall economy health dashboard.
pub fn get_health_dashboard(env: &Env) -> EconomyHealthDashboard {
    env.storage()
        .persistent()
        .get(&DashboardKey::EconomyHealth)
        .unwrap_or(EconomyHealthDashboard {
            supply_demand: SupplyDemandMetrics {
                total_supply: 0,
                market_supply: 0,
                market_demand: 0,
                supply_demand_ratio: 10000,
                last_update: 0,
            },
            inflation: InflationMetrics {
                annual_inflation_bps: 500,
                epoch_inflation: 0,
                epoch_deflation: 0,
                net_supply_change: 0,
                last_update: 0,
            },
            health_status: EconomyHealth::Healthy,
            active_alerts: 0,
            last_calculated: 0,
        })
}

/// Get recent alert events (limited to last 20).
pub fn get_recent_alerts(env: &Env, limit: u32) -> Vec<AlertEvent> {
    let all_alerts: Vec<AlertEvent> = env
        .storage()
        .persistent()
        .get(&DashboardKey::AlertHistory)
        .unwrap_or_else(|| Vec::new(env));

    let mut result: Vec<AlertEvent> = Vec::new(env);
    let start_idx = if (limit as usize) < all_alerts.len() as usize {
        all_alerts.len() as usize - (limit as usize)
    } else {
        0
    };

    for i in start_idx..all_alerts.len() as usize {
        if let Some(alert) = all_alerts.get(i as u32) {
            result.push_back(alert.clone());
        }
    }

    result
}

/// Get a specific alert threshold configuration.
pub fn get_alert_threshold(env: &Env, alert_name: Symbol) -> Option<AlertThreshold> {
    env.storage()
        .persistent()
        .get(&DashboardKey::AlertThreshold(alert_name))
}
