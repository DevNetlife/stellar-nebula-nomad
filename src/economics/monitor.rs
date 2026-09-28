//! Economic monitoring system for token supply and inflation tracking

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct EconomicMetrics {
    pub total_supply: i128,
    pub circulating_supply: i128,
    pub staked_supply: i128,
    pub inflation_rate_bps: u32,
    pub last_update: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ResourceMetrics {
    pub resource_type: Symbol,
    pub total_minted: i128,
    pub total_burned: i128,
    pub avg_price: i128,
    pub price_change_24h: i32,
}

#[contracttype]
#[derive(Clone)]
pub enum EconKey {
    Metrics,
    ResourceMetrics(Symbol),
    SupplyHistory,
}

/// Initialize economic monitoring
pub fn initialize_monitor(env: &Env, admin: &Address) {
    admin.require_auth();
    let metrics = EconomicMetrics {
        total_supply: 0,
        circulating_supply: 0,
        staked_supply: 0,
        inflation_rate_bps: 500, // 5% default
        last_update: env.ledger().timestamp(),
    };
    env.storage().persistent().set(&EconKey::Metrics, &metrics);
}

/// Update token supply metrics
pub fn update_supply_metrics(
    env: &Env,
    admin: &Address,
    total_supply: i128,
    circulating_supply: i128,
    staked_supply: i128,
) {
    admin.require_auth();
    
    let mut metrics: EconomicMetrics = env
        .storage()
        .persistent()
        .get(&EconKey::Metrics)
        .unwrap_or(EconomicMetrics {
            total_supply: 0,
            circulating_supply: 0,
            staked_supply: 0,
            inflation_rate_bps: 500,
            last_update: 0,
        });
    
    metrics.total_supply = total_supply;
    metrics.circulating_supply = circulating_supply;
    metrics.staked_supply = staked_supply;
    metrics.last_update = env.ledger().timestamp();
    
    env.storage().persistent().set(&EconKey::Metrics, &metrics);
    
    env.events().publish(
        (symbol_short!("econ"), symbol_short!("supply")),
        (total_supply, circulating_supply, staked_supply),
    );
}

/// Track resource minting/burning
pub fn track_resource_activity(
    env: &Env,
    resource_type: Symbol,
    minted: i128,
    burned: i128,
    avg_price: i128,
) {
    let key = EconKey::ResourceMetrics(resource_type.clone());
    let mut metrics: ResourceMetrics = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(ResourceMetrics {
            resource_type: resource_type.clone(),
            total_minted: 0,
            total_burned: 0,
            avg_price: 0,
            price_change_24h: 0,
        });
    
    metrics.total_minted += minted;
    metrics.total_burned += burned;
    metrics.avg_price = avg_price;
    
    env.storage().persistent().set(&key, &metrics);
}

/// Get current economic metrics
pub fn get_metrics(env: &Env) -> EconomicMetrics {
    env.storage()
        .persistent()
        .get(&EconKey::Metrics)
        .unwrap_or(EconomicMetrics {
            total_supply: 0,
            circulating_supply: 0,
            staked_supply: 0,
            inflation_rate_bps: 500,
            last_update: 0,
        })
}

/// Get resource-specific metrics
pub fn get_resource_metrics(env: &Env, resource_type: Symbol) -> ResourceMetrics {
    env.storage()
        .persistent()
        .get(&EconKey::ResourceMetrics(resource_type.clone()))
        .unwrap_or(ResourceMetrics {
            resource_type,
            total_minted: 0,
            total_burned: 0,
            avg_price: 0,
            price_change_24h: 0,
        })
}

/// Calculate inflation rate based on supply growth
pub fn calculate_inflation_rate(_env: &Env, old_supply: i128, new_supply: i128) -> u32 {
    if old_supply == 0 {
        return 0;
    }
    let growth = new_supply - old_supply;
    let rate = (growth * 10000) / old_supply;
    rate.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize_and_update() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        
        initialize_monitor(&env, &admin);
        update_supply_metrics(&env, &admin, 1000000, 800000, 200000);
        
        let metrics = get_metrics(&env);
        assert_eq!(metrics.total_supply, 1000000);
        assert_eq!(metrics.circulating_supply, 800000);
        assert_eq!(metrics.staked_supply, 200000);
    }

    #[test]
    fn test_inflation_calculation() {
        let env = Env::default();
        let rate = calculate_inflation_rate(&env, 1000000, 1050000);
        assert_eq!(rate, 500); // 5%
    }

    #[test]
    fn test_resource_tracking() {
        let env = Env::default();
        let resource = symbol_short!("dust");
        
        track_resource_activity(&env, resource.clone(), 1000, 100, 50);
        let metrics = get_resource_metrics(&env, resource);
        
        assert_eq!(metrics.total_minted, 1000);
        assert_eq!(metrics.total_burned, 100);
        assert_eq!(metrics.avg_price, 50);
    }
}
