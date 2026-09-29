//! # Smart Alerts for Anomalies (#373)
//!
//! Threshold-configurable alerting layered on top of the existing
//! `health_monitor`/`anomaly_classifier` metrics infrastructure. Operators
//! register per-metric thresholds; each time a metric sample is reported,
//! this module evaluates it against the configured threshold and, on
//! breach, raises an alert with a severity that escalates the more times
//! the same metric keeps breaching without being acknowledged.
//!
//! ## Escalation Policy
//!
//! - 1st unacknowledged breach → `Warning`
//! - 2nd/3rd unacknowledged breach → `Critical`
//! - 4th+ unacknowledged breach → `PageOps` (highest severity)
//!
//! Acknowledging an alert (`acknowledge`) resets the breach streak for that
//! metric back to `Warning` on the next breach.

use soroban_sdk::{contracterror, contracttype, Env, Symbol, Vec};

use crate::error_standard::{ErrorDescriptor, ErrorKind, StandardContractError};

// ── Error ───────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AlertError {
    /// No threshold has been configured for this metric.
    ThresholdNotConfigured = 1,
    /// Referenced alert does not exist.
    AlertNotFound = 2,
}

impl StandardContractError for AlertError {
    fn descriptor(self) -> ErrorDescriptor {
        ErrorDescriptor {
            module: "smart_alerts",
            code: self as u32,
            kind: match self {
                AlertError::ThresholdNotConfigured => ErrorKind::Validation,
                AlertError::AlertNotFound => ErrorKind::NotFound,
            },
            retryable: false,
        }
    }
}

// ── Storage Keys ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum AlertKey {
    /// Configured threshold for a metric.
    Threshold(Symbol),
    /// Escalation state (consecutive unacknowledged breaches) for a metric.
    EscalationState(Symbol),
    /// Active alert log entries, most-recent-last, capped at `MAX_ALERTS`.
    ActiveAlerts,
    /// Monotonic alert id counter.
    AlertCounter,
}

// ── Constants ───────────────────────────────────────────────────────────

/// Maximum number of active alerts retained in the on-chain log.
const MAX_ALERTS: u32 = 30;
/// Breach streak thresholds for escalation.
const CRITICAL_AT_STREAK: u32 = 2;
const PAGE_OPS_AT_STREAK: u32 = 4;

// ── Data Types ──────────────────────────────────────────────────────────

/// Threshold configuration for a single metric.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct AlertThreshold {
    pub metric: Symbol,
    /// Alert fires when the sampled value is >= this value.
    pub max_value: u64,
}

/// Alert severity, escalating with repeated unacknowledged breaches.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum AlertSeverity {
    Warning,
    Critical,
    PageOps,
}

/// A raised alert record.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct AlertRecord {
    pub alert_id: u64,
    pub metric: Symbol,
    pub value: u64,
    pub threshold: u64,
    pub severity: AlertSeverity,
    pub timestamp: u64,
    pub acknowledged: bool,
}

// ── Configuration ───────────────────────────────────────────────────────

/// Configure (or replace) the threshold for `metric`.
pub fn configure_threshold(env: &Env, metric: Symbol, max_value: u64) {
    env.storage().persistent().set(
        &AlertKey::Threshold(metric.clone()),
        &AlertThreshold { metric, max_value },
    );
}

/// Fetch the configured threshold for `metric`, if any.
pub fn get_threshold(env: &Env, metric: &Symbol) -> Option<AlertThreshold> {
    env.storage()
        .persistent()
        .get(&AlertKey::Threshold(metric.clone()))
}

// ── Core Evaluation ─────────────────────────────────────────────────────

/// Evaluate a metric sample against its configured threshold. If it
/// breaches, raises and stores an escalating alert and returns it.
/// Returns `Ok(None)` if the metric is within bounds, and
/// `Err(ThresholdNotConfigured)` if no threshold was set for this metric.
pub fn evaluate_sample(
    env: &Env,
    metric: Symbol,
    value: u64,
) -> Result<Option<AlertRecord>, AlertError> {
    let threshold = get_threshold(env, &metric).ok_or(AlertError::ThresholdNotConfigured)?;

    if value < threshold.max_value {
        // Sample is healthy: reset escalation streak.
        env.storage()
            .temporary()
            .set(&AlertKey::EscalationState(metric.clone()), &0u32);
        return Ok(None);
    }

    let streak: u32 = env
        .storage()
        .temporary()
        .get(&AlertKey::EscalationState(metric.clone()))
        .unwrap_or(0)
        + 1;
    env.storage()
        .temporary()
        .set(&AlertKey::EscalationState(metric.clone()), &streak);

    let severity = if streak >= PAGE_OPS_AT_STREAK {
        AlertSeverity::PageOps
    } else if streak >= CRITICAL_AT_STREAK {
        AlertSeverity::Critical
    } else {
        AlertSeverity::Warning
    };

    let alert_id: u64 = env
        .storage()
        .persistent()
        .get(&AlertKey::AlertCounter)
        .unwrap_or(0)
        + 1;
    env.storage()
        .persistent()
        .set(&AlertKey::AlertCounter, &alert_id);

    let record = AlertRecord {
        alert_id,
        metric,
        value,
        threshold: threshold.max_value,
        severity,
        timestamp: env.ledger().timestamp(),
        acknowledged: false,
    };

    let mut active: Vec<AlertRecord> = env
        .storage()
        .persistent()
        .get(&AlertKey::ActiveAlerts)
        .unwrap_or_else(|| Vec::new(env));
    if active.len() >= MAX_ALERTS {
        active.remove(0);
    }
    active.push_back(record.clone());
    env.storage()
        .persistent()
        .set(&AlertKey::ActiveAlerts, &active);

    Ok(Some(record))
}

/// Acknowledge the alert with `alert_id`, resetting its metric's escalation
/// streak so the next breach starts back at `Warning`.
pub fn acknowledge(env: &Env, alert_id: u64) -> Result<(), AlertError> {
    let mut active: Vec<AlertRecord> = env
        .storage()
        .persistent()
        .get(&AlertKey::ActiveAlerts)
        .unwrap_or_else(|| Vec::new(env));

    for i in 0..active.len() {
        let mut record = active.get(i).unwrap();
        if record.alert_id == alert_id {
            record.acknowledged = true;
            let metric = record.metric.clone();
            active.set(i, record);
            env.storage()
                .persistent()
                .set(&AlertKey::ActiveAlerts, &active);
            env.storage()
                .temporary()
                .set(&AlertKey::EscalationState(metric), &0u32);
            return Ok(());
        }
    }
    Err(AlertError::AlertNotFound)
}

/// List currently stored active alerts (most-recent-last).
pub fn list_active_alerts(env: &Env) -> Vec<AlertRecord> {
    env.storage()
        .persistent()
        .get(&AlertKey::ActiveAlerts)
        .unwrap_or_else(|| Vec::new(env))
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short, Env};

    #[contract]
    struct Stub;
    #[contractimpl]
    impl Stub {}

    fn make_env() -> (Env, soroban_sdk::Address) {
        let env = Env::default();
        let id = env.register(Stub, ());
        (env, id)
    }

    #[test]
    fn test_no_alert_below_threshold() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let metric = symbol_short!("latency");
            configure_threshold(&env, metric.clone(), 100);
            let result = evaluate_sample(&env, metric, 50).unwrap();
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_missing_threshold_errors() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let metric = symbol_short!("unknown");
            let err = evaluate_sample(&env, metric, 50).unwrap_err();
            assert_eq!(err, AlertError::ThresholdNotConfigured);
        });
    }

    #[test]
    fn test_breach_escalates_severity() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let metric = symbol_short!("errs");
            configure_threshold(&env, metric.clone(), 10);

            let a1 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a1.severity, AlertSeverity::Warning);

            let a2 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a2.severity, AlertSeverity::Critical);

            let a3 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a3.severity, AlertSeverity::Critical);

            let a4 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a4.severity, AlertSeverity::PageOps);
        });
    }

    #[test]
    fn test_acknowledge_resets_streak() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let metric = symbol_short!("errs");
            configure_threshold(&env, metric.clone(), 10);

            let a1 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            let a2 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a2.severity, AlertSeverity::Critical);

            acknowledge(&env, a1.alert_id).unwrap();

            let a3 = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(a3.severity, AlertSeverity::Warning);
        });
    }

    #[test]
    fn test_acknowledge_unknown_alert_errors() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let err = acknowledge(&env, 999).unwrap_err();
            assert_eq!(err, AlertError::AlertNotFound);
        });
    }

    #[test]
    fn test_healthy_sample_resets_streak() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let metric = symbol_short!("errs");
            configure_threshold(&env, metric.clone(), 10);

            evaluate_sample(&env, metric.clone(), 15).unwrap();
            evaluate_sample(&env, metric.clone(), 15).unwrap();
            // Healthy sample resets streak.
            let healthy = evaluate_sample(&env, metric.clone(), 1).unwrap();
            assert!(healthy.is_none());

            let next = evaluate_sample(&env, metric.clone(), 15).unwrap().unwrap();
            assert_eq!(next.severity, AlertSeverity::Warning);
        });
    }
}
