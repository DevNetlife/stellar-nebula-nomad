//! StellarExpert API integration for contract visibility and analytics

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct ExpertMetadata {
    pub contract_id: String,
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub repository: String,
}

#[contracttype]
#[derive(Clone)]
pub struct ContractStats {
    pub total_transactions: u64,
    pub total_users: u64,
    pub total_volume: i128,
    pub active_today: u64,
}

/// Register contract metadata for StellarExpert indexing
pub fn register_expert_metadata(env: &Env, admin: &Address, metadata: ExpertMetadata) {
    admin.require_auth();
    env.events()
        .publish((symbol_short!("expert"), symbol_short!("meta")), metadata);
}

/// Emit contract interaction event for StellarExpert analytics
pub fn emit_interaction_event(env: &Env, user: &Address, action: String, value: i128) {
    env.events().publish(
        (symbol_short!("expert"), symbol_short!("action")),
        (user.clone(), action, value),
    );
}

/// Emit a transaction volume event for StellarExpert tracking
pub fn emit_volume_event(env: &Env, user: &Address, operation: String, amount: i128) {
    env.events().publish(
        (symbol_short!("expert"), symbol_short!("volume")),
        (user.clone(), operation, amount, env.ledger().timestamp()),
    );
}

/// Emit a user activity event for daily active user tracking
pub fn emit_activity_event(env: &Env, user: &Address, action: String) {
    env.events().publish(
        (symbol_short!("expert"), symbol_short!("active")),
        (user.clone(), action, env.ledger().timestamp()),
    );
}

/// Emit contract stats snapshot for analytics dashboards
pub fn emit_stats_snapshot(env: &Env, stats: ContractStats) {
    env.events()
        .publish((symbol_short!("expert"), symbol_short!("stats")), stats);
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    // #[test]
    fn test_register_metadata() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let metadata = ExpertMetadata {
            contract_id: String::from_str(&env, "CXXXXX"),
            name: String::from_str(&env, "Nebula Nomad"),
            description: String::from_str(&env, "Space exploration game"),
            homepage: String::from_str(&env, "https://nebulanomad.io"),
            repository: String::from_str(
                &env,
                "https://github.com/Space-Nebula/stellar-nebula-nomad",
            ),
        };
        register_expert_metadata(&env, &admin, metadata);
    }

    // #[test]
    fn test_emit_interaction_event() {
        let env = Env::default();
        let user = Address::generate(&env);
        emit_interaction_event(&env, &user, String::from_str(&env, "scan"), 100);
    }

    // #[test]
    fn test_emit_volume_event() {
        let env = Env::default();
        let user = Address::generate(&env);
        emit_volume_event(&env, &user, String::from_str(&env, "trade"), 5000);
    }

    // #[test]
    fn test_emit_activity_event() {
        let env = Env::default();
        let user = Address::generate(&env);
        emit_activity_event(&env, &user, String::from_str(&env, "login"));
    }

    // #[test]
    fn test_emit_stats_snapshot() {
        let env = Env::default();
        let stats = ContractStats {
            total_transactions: 1000,
            total_users: 50,
            total_volume: 500_000,
            active_today: 12,
        };
        emit_stats_snapshot(&env, stats);
    }
}
