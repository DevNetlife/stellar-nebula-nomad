//! # Revenue Attribution (#369)
//!
//! Analytics backend that tracks which acquisition/marketing channel a
//! revenue-generating event (purchase, content sale, etc.) originated
//! from, and computes per-channel totals and ROI given a recorded spend.
//!
//! This is distinct from `content_tools`' creator/platform revenue *split*
//! (Issue #192), which divides a single purchase between two parties.
//! Revenue attribution instead answers "which channel drove this
//! revenue?" across the whole game economy, for business-intelligence
//! reporting.

use soroban_sdk::{contracterror, contracttype, Env, Symbol, Vec};

use crate::error_standard::{ErrorDescriptor, ErrorKind, StandardContractError};

// ── Error ───────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AttributionError {
    /// Revenue amount must be greater than zero.
    ZeroAmount = 1,
}

impl StandardContractError for AttributionError {
    fn descriptor(self) -> ErrorDescriptor {
        ErrorDescriptor {
            module: "revenue_attribution",
            code: self as u32,
            kind: ErrorKind::Validation,
            retryable: false,
        }
    }
}

// ── Storage Keys ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum AttributionKey {
    /// Cumulative revenue attributed to a channel.
    ChannelRevenue(Symbol),
    /// Cumulative recorded marketing spend for a channel.
    ChannelSpend(Symbol),
    /// Number of revenue events attributed to a channel.
    ChannelEventCount(Symbol),
    /// List of known channel symbols (for iteration).
    KnownChannels,
}

// ── Data Types ──────────────────────────────────────────────────────────

/// Per-channel attribution summary.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ChannelAttribution {
    pub channel: Symbol,
    pub revenue: u64,
    pub spend: u64,
    pub event_count: u64,
    /// ROI in basis points: (revenue - spend) / spend * 10_000.
    /// `None` when spend is zero (ROI undefined).
    pub roi_bps: Option<i64>,
}

// ── Recording ───────────────────────────────────────────────────────────

/// Record a revenue event of `amount` attributed to `channel` (e.g.
/// `symbol_short!("organic")`, `symbol_short!("referral")`,
/// `symbol_short!("ad_camp")`).
pub fn record_revenue(env: &Env, channel: Symbol, amount: u64) -> Result<(), AttributionError> {
    if amount == 0 {
        return Err(AttributionError::ZeroAmount);
    }

    register_channel(env, &channel);

    let prev: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelRevenue(channel.clone()))
        .unwrap_or(0);
    env.storage().persistent().set(
        &AttributionKey::ChannelRevenue(channel.clone()),
        &prev.saturating_add(amount),
    );

    let count: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelEventCount(channel.clone()))
        .unwrap_or(0);
    env.storage().persistent().set(
        &AttributionKey::ChannelEventCount(channel.clone()),
        &count.saturating_add(1),
    );

    Ok(())
}

/// Record marketing/acquisition spend for `channel` (used for ROI calc).
pub fn record_spend(env: &Env, channel: Symbol, amount: u64) {
    register_channel(env, &channel);

    let prev: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelSpend(channel.clone()))
        .unwrap_or(0);
    env.storage().persistent().set(
        &AttributionKey::ChannelSpend(channel.clone()),
        &prev.saturating_add(amount),
    );
}

fn register_channel(env: &Env, channel: &Symbol) {
    let mut channels: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&AttributionKey::KnownChannels)
        .unwrap_or_else(|| Vec::new(env));
    if !channels.iter().any(|c| c == *channel) {
        channels.push_back(channel.clone());
        env.storage()
            .persistent()
            .set(&AttributionKey::KnownChannels, &channels);
    }
}

// ── Queries ─────────────────────────────────────────────────────────────

/// Get the full attribution record (revenue, spend, ROI) for `channel`.
pub fn get_channel_attribution(env: &Env, channel: Symbol) -> ChannelAttribution {
    let revenue: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelRevenue(channel.clone()))
        .unwrap_or(0);
    let spend: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelSpend(channel.clone()))
        .unwrap_or(0);
    let event_count: u64 = env
        .storage()
        .persistent()
        .get(&AttributionKey::ChannelEventCount(channel.clone()))
        .unwrap_or(0);

    let roi_bps = if spend == 0 {
        None
    } else {
        let net = revenue as i64 - spend as i64;
        Some(net.saturating_mul(10_000) / spend as i64)
    };

    ChannelAttribution {
        channel,
        revenue,
        spend,
        event_count,
        roi_bps,
    }
}

/// Return attribution summaries for every channel that has recorded
/// revenue or spend so far.
pub fn all_channel_attributions(env: &Env) -> Vec<ChannelAttribution> {
    let channels: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&AttributionKey::KnownChannels)
        .unwrap_or_else(|| Vec::new(env));

    let mut result: Vec<ChannelAttribution> = Vec::new(env);
    for channel in channels.iter() {
        result.push_back(get_channel_attribution(env, channel));
    }
    result
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
    fn test_record_revenue_accumulates() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let channel = symbol_short!("organic");
            record_revenue(&env, channel.clone(), 100).unwrap();
            record_revenue(&env, channel.clone(), 50).unwrap();

            let attr = get_channel_attribution(&env, channel);
            assert_eq!(attr.revenue, 150);
            assert_eq!(attr.event_count, 2);
        });
    }

    #[test]
    fn test_zero_amount_rejected() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let channel = symbol_short!("organic");
            let err = record_revenue(&env, channel, 0).unwrap_err();
            assert_eq!(err, AttributionError::ZeroAmount);
        });
    }

    #[test]
    fn test_roi_calculation() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let channel = symbol_short!("ad_camp");
            record_spend(&env, channel.clone(), 100);
            record_revenue(&env, channel.clone(), 150).unwrap();

            let attr = get_channel_attribution(&env, channel);
            // (150 - 100) / 100 * 10000 = 5000 bps = 50% ROI
            assert_eq!(attr.roi_bps, Some(5000));
        });
    }

    #[test]
    fn test_roi_none_without_spend() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let channel = symbol_short!("organic");
            record_revenue(&env, channel.clone(), 150).unwrap();

            let attr = get_channel_attribution(&env, channel);
            assert_eq!(attr.roi_bps, None);
        });
    }

    #[test]
    fn test_all_channel_attributions_lists_every_channel() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            record_revenue(&env, symbol_short!("organic"), 10).unwrap();
            record_revenue(&env, symbol_short!("referral"), 20).unwrap();
            record_spend(&env, symbol_short!("ad_camp"), 5);

            let all = all_channel_attributions(&env);
            assert_eq!(all.len(), 3);
        });
    }

    #[test]
    fn test_negative_roi_when_spend_exceeds_revenue() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let channel = symbol_short!("ad_camp");
            record_spend(&env, channel.clone(), 200);
            record_revenue(&env, channel.clone(), 100).unwrap();

            let attr = get_channel_attribution(&env, channel);
            // (100 - 200) / 200 * 10000 = -5000 bps
            assert_eq!(attr.roi_bps, Some(-5000));
        });
    }
}
