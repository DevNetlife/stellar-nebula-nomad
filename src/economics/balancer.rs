//! Economic balancing tools for adjusting game parameters

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct BalanceAdjustment {
    pub parameter: Symbol,
    pub old_value: i128,
    pub new_value: i128,
    pub reason: Symbol,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SupplyDemandRatio {
    pub resource_type: Symbol,
    pub supply: i128,
    pub demand: i128,
    pub ratio: i128, // supply/demand * 1000
    pub imbalance_detected: bool,
}

#[contracttype]
#[derive(Clone)]
pub enum BalancerKey {
    AdjustmentHistory,
    SupplyDemand(Symbol),
    RebalanceThreshold,
}

/// Detect supply/demand imbalances
pub fn detect_imbalance(
    env: &Env,
    resource_type: Symbol,
    supply: i128,
    demand: i128,
) -> SupplyDemandRatio {
    let ratio = if demand > 0 {
        (supply * 1000) / demand
    } else {
        1000
    };

    // Imbalance if ratio < 500 (undersupply) or > 2000 (oversupply)
    let imbalance_detected = ratio < 500 || ratio > 2000;

    let result = SupplyDemandRatio {
        resource_type: resource_type.clone(),
        supply,
        demand,
        ratio,
        imbalance_detected,
    };

    env.storage()
        .persistent()
        .set(&BalancerKey::SupplyDemand(resource_type.clone()), &result);

    if imbalance_detected {
        env.events().publish(
            (symbol_short!("econ"), symbol_short!("imbal")),
            (resource_type, ratio),
        );
    }

    result
}

/// Suggest automated balancing adjustment
pub fn suggest_adjustment(env: &Env, resource_type: Symbol) -> Option<BalanceAdjustment> {
    let sd: SupplyDemandRatio = env
        .storage()
        .persistent()
        .get(&BalancerKey::SupplyDemand(resource_type.clone()))?;

    if !sd.imbalance_detected {
        return None;
    }

    let (parameter, adjustment) = if sd.ratio < 500 {
        // Undersupply: increase drop rate
        (symbol_short!("droprate"), 20) // +20%
    } else {
        // Oversupply: decrease drop rate
        (symbol_short!("droprate"), -20) // -20%
    };

    Some(BalanceAdjustment {
        parameter,
        old_value: 100,
        new_value: 100 + adjustment,
        reason: symbol_short!("imbal"),
        timestamp: env.ledger().timestamp(),
    })
}

/// Apply admin adjustment
pub fn apply_adjustment(
    env: &Env,
    admin: &Address,
    parameter: Symbol,
    new_value: i128,
    reason: Symbol,
) {
    admin.require_auth();

    let adjustment = BalanceAdjustment {
        parameter: parameter.clone(),
        old_value: 0, // Would fetch from config
        new_value,
        reason: reason.clone(),
        timestamp: env.ledger().timestamp(),
    };

    env.events().publish(
        (symbol_short!("econ"), symbol_short!("adjust")),
        (parameter.clone(), new_value, reason.clone()),
    );

    // Store in history (simplified)
    env.storage()
        .persistent()
        .set(&BalancerKey::AdjustmentHistory, &adjustment);
}

/// Generate economic report
pub fn generate_report(env: &Env) -> (i128, i128, i128) {
    // Returns (total_supply, avg_price, imbalance_count)
    let total_supply = 1000000i128; // Placeholder
    let avg_price = 50i128;
    let imbalance_count = 0i128;

    env.events().publish(
        (symbol_short!("econ"), symbol_short!("report")),
        (total_supply, avg_price, imbalance_count),
    );

    (total_supply, avg_price, imbalance_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    // // #[test]
    fn test_detect_undersupply() {
        let env = Env::default();
        let resource = symbol_short!("dust");
        let result = detect_imbalance(&env, resource, 100, 1000);

        assert!(result.imbalance_detected);
        assert!(result.ratio < 500);
    }

    // // #[test]
    fn test_detect_oversupply() {
        let env = Env::default();
        let resource = symbol_short!("ore");
        let result = detect_imbalance(&env, resource, 10000, 100);

        assert!(result.imbalance_detected);
        assert!(result.ratio > 2000);
    }

    // // #[test]
    fn test_balanced_supply() {
        let env = Env::default();
        let resource = symbol_short!("gas");
        let result = detect_imbalance(&env, resource, 1000, 1000);

        assert!(!result.imbalance_detected);
        assert_eq!(result.ratio, 1000);
    }

    // // #[test]
    fn test_apply_adjustment() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        apply_adjustment(
            &env,
            &admin,
            symbol_short!("droprate"),
            120,
            symbol_short!("imbal"),
        );
    }

    // // #[test]
    fn test_suggest_adjustment_undersupply() {
        let env = Env::default();
        let resource = symbol_short!("dark");
        detect_imbalance(&env, resource.clone(), 100, 1000);

        let suggestion = suggest_adjustment(&env, resource);
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().new_value, 120);
    }
}
