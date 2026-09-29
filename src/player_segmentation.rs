//! Player behavioral segmentation for targeting and personalization.
//!
//! Segments players based on activity patterns, spending, and engagement levels
//! to enable targeted marketing campaigns and personalized experiences.

use crate::error_standard::{ErrorDescriptor, ErrorKind, StandardContractError};
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ── Error ─────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SegmentationError {
    /// Invalid segment ID provided.
    InvalidSegment = 1,
    /// Player not found in segment mapping.
    PlayerNotFound = 2,
}

impl StandardContractError for SegmentationError {
    fn descriptor(self) -> ErrorDescriptor {
        ErrorDescriptor {
            module: "player_segmentation",
            code: self as u32,
            kind: ErrorKind::Validation,
            retryable: false,
        }
    }
}

// ── Storage Keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum SegmentKey {
    /// Player's current behavioral segment.
    PlayerSegment(Address),
    /// List of players in a given segment.
    SegmentMembers(Symbol),
    /// Segment engagement metrics.
    SegmentMetrics(Symbol),
    /// Player engagement history for segmentation analysis.
    PlayerEngagement(Address),
}

// ── Data Types ────────────────────────────────────────────────────────────────

/// Behavioral segment categories.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerSegment {
    /// Highly active, high-value players.
    VIP = 1,
    /// Regular, consistent players.
    Core = 2,
    /// Casual, low-frequency players.
    Casual = 3,
    /// At-risk players showing low engagement.
    AtRisk = 4,
    /// Recently churned players.
    Churned = 5,
}

/// Player engagement metrics for segmentation.
#[contracttype]
#[derive(Clone)]
pub struct PlayerEngagementMetrics {
    /// Lifetime value in essence.
    pub lifetime_essence: u64,
    /// Number of active sessions in last 7 days.
    pub sessions_last_7d: u32,
    /// Total purchases or transactions.
    pub total_transactions: u32,
    /// Average session duration in seconds.
    pub avg_session_duration: u64,
    /// Days since last activity.
    pub days_inactive: u32,
    /// Current segment.
    pub segment: PlayerSegment,
    /// Last segment update timestamp.
    pub last_segmentation_update: u64,
}

/// Segment-level metrics and targeting data.
#[contracttype]
#[derive(Clone)]
pub struct SegmentMetrics {
    pub segment_name: Symbol,
    pub member_count: u32,
    pub avg_lifetime_value: u64,
    pub retention_rate: u32,   // basis points (0-10000)
    pub churn_rate: u32,       // basis points (0-10000)
    pub engagement_score: u32, // 0-100
}

// ── Write Helpers ─────────────────────────────────────────────────────────────

/// Update a player's engagement metrics and recalculate segment.
pub fn update_player_engagement(
    env: &Env,
    player: &Address,
    essence_earned: u64,
    sessions: u32,
    transactions: u32,
    session_duration: u64,
) {
    let mut metrics: PlayerEngagementMetrics = env
        .storage()
        .persistent()
        .get(&SegmentKey::PlayerEngagement(player.clone()))
        .unwrap_or(PlayerEngagementMetrics {
            lifetime_essence: 0,
            sessions_last_7d: 0,
            total_transactions: 0,
            avg_session_duration: 0,
            days_inactive: 0,
            segment: PlayerSegment::Casual,
            last_segmentation_update: 0,
        });

    metrics.lifetime_essence = metrics.lifetime_essence.saturating_add(essence_earned);
    metrics.sessions_last_7d = metrics.sessions_last_7d.saturating_add(sessions);
    metrics.total_transactions = metrics.total_transactions.saturating_add(transactions);

    if metrics.sessions_last_7d > 0 {
        metrics.avg_session_duration = (metrics.avg_session_duration
            * (metrics.sessions_last_7d - sessions) as u64
            + session_duration * sessions as u64)
            / metrics.sessions_last_7d as u64;
    } else {
        metrics.avg_session_duration = session_duration;
    }

    metrics.days_inactive = 0; // Reset inactivity counter on engagement
    metrics.last_segmentation_update = env.ledger().timestamp();

    // Recalculate segment based on updated metrics
    let old_segment = metrics.segment;
    metrics.segment = calculate_segment(&metrics);

    // If segment changed, update segment membership
    if old_segment != metrics.segment {
        remove_from_segment(env, player, old_segment);
        add_to_segment(env, player, metrics.segment);
    }

    env.storage()
        .persistent()
        .set(&SegmentKey::PlayerEngagement(player.clone()), &metrics);

    env.storage()
        .persistent()
        .set(&SegmentKey::PlayerSegment(player.clone()), &metrics.segment);

    env.events().publish(
        (symbol_short!("seg"), symbol_short!("update")),
        (player.clone(), metrics.segment as u32),
    );
}

/// Manually transition a player to a specific segment.
pub fn set_player_segment(env: &Env, admin: &Address, player: &Address, segment: PlayerSegment) {
    admin.require_auth();

    let old_segment: PlayerSegment = env
        .storage()
        .persistent()
        .get(&SegmentKey::PlayerSegment(player.clone()))
        .unwrap_or(PlayerSegment::Casual);

    if old_segment != segment {
        remove_from_segment(env, player, old_segment);
        add_to_segment(env, player, segment);
    }

    env.storage()
        .persistent()
        .set(&SegmentKey::PlayerSegment(player.clone()), &segment);

    env.events().publish(
        (symbol_short!("seg"), symbol_short!("manual")),
        (player.clone(), segment as u32),
    );
}

/// Add a player to a segment's member list.
fn add_to_segment(env: &Env, player: &Address, segment: PlayerSegment) {
    let segment_name = segment_to_symbol(segment);
    let mut members: Vec<Address> = env
        .storage()
        .persistent()
        .get(&SegmentKey::SegmentMembers(segment_name.clone()))
        .unwrap_or_else(|| Vec::new(env));

    // Avoid duplicates
    for i in 0..members.len() {
        if let Some(p) = members.get(i) {
            if p == *player {
                return;
            }
        }
    }

    members.push_back(player.clone());
    env.storage()
        .persistent()
        .set(&SegmentKey::SegmentMembers(segment_name), &members);
}

/// Remove a player from a segment's member list.
fn remove_from_segment(env: &Env, player: &Address, segment: PlayerSegment) {
    let segment_name = segment_to_symbol(segment);
    let members: Vec<Address> = env
        .storage()
        .persistent()
        .get(&SegmentKey::SegmentMembers(segment_name.clone()))
        .unwrap_or_else(|| Vec::new(env));

    let mut new_members: Vec<Address> = Vec::new(env);
    for i in 0..members.len() {
        if let Some(p) = members.get(i) {
            if p != *player {
                new_members.push_back(p.clone());
            }
        }
    }

    if new_members.len() > 0 {
        env.storage()
            .persistent()
            .set(&SegmentKey::SegmentMembers(segment_name), &new_members);
    }
}

/// Calculate the appropriate segment based on engagement metrics.
fn calculate_segment(metrics: &PlayerEngagementMetrics) -> PlayerSegment {
    // VIP: high essence + frequent transactions
    if metrics.lifetime_essence > 100_000 && metrics.total_transactions > 50 {
        return PlayerSegment::VIP;
    }

    // Core: consistent engagement with good retention
    if metrics.sessions_last_7d >= 5 && metrics.total_transactions > 20 {
        return PlayerSegment::Core;
    }

    // AtRisk: was active but now inactive
    if metrics.days_inactive > 14 && metrics.total_transactions > 5 {
        return PlayerSegment::AtRisk;
    }

    // Churned: very inactive after some engagement
    if metrics.days_inactive > 30 && metrics.total_transactions > 0 {
        return PlayerSegment::Churned;
    }

    // Default: Casual players
    PlayerSegment::Casual
}

/// Convert segment enum to symbol for storage.
fn segment_to_symbol(segment: PlayerSegment) -> Symbol {
    match segment {
        PlayerSegment::VIP => symbol_short!("vip"),
        PlayerSegment::Core => symbol_short!("core"),
        PlayerSegment::Casual => symbol_short!("casual"),
        PlayerSegment::AtRisk => symbol_short!("atrisk"),
        PlayerSegment::Churned => symbol_short!("churned"),
    }
}

// ── Read-Only View Functions ──────────────────────────────────────────────────

/// Get a player's current segment.
pub fn get_player_segment(env: &Env, player: &Address) -> PlayerSegment {
    env.storage()
        .persistent()
        .get(&SegmentKey::PlayerSegment(player.clone()))
        .unwrap_or(PlayerSegment::Casual)
}

/// Get a player's full engagement metrics.
pub fn get_player_engagement(env: &Env, player: &Address) -> PlayerEngagementMetrics {
    env.storage()
        .persistent()
        .get(&SegmentKey::PlayerEngagement(player.clone()))
        .unwrap_or(PlayerEngagementMetrics {
            lifetime_essence: 0,
            sessions_last_7d: 0,
            total_transactions: 0,
            avg_session_duration: 0,
            days_inactive: 0,
            segment: PlayerSegment::Casual,
            last_segmentation_update: 0,
        })
}

/// Get all members of a specific segment (limited to avoid unbounded iteration).
pub fn get_segment_members(env: &Env, segment: PlayerSegment, limit: u32) -> Vec<Address> {
    let segment_name = segment_to_symbol(segment);
    let all_members: Vec<Address> = env
        .storage()
        .persistent()
        .get(&SegmentKey::SegmentMembers(segment_name))
        .unwrap_or_else(|| Vec::new(env));

    let mut result: Vec<Address> = Vec::new(env);
    let count = if (limit as usize) < all_members.len() as usize {
        limit as usize
    } else {
        all_members.len() as usize
    };

    for i in 0..count {
        if let Some(member) = all_members.get(i as u32) {
            result.push_back(member.clone());
        }
    }

    result
}

/// Get aggregated metrics for a segment.
pub fn get_segment_metrics(env: &Env, segment: PlayerSegment) -> SegmentMetrics {
    let segment_name = segment_to_symbol(segment);

    env.storage()
        .persistent()
        .get(&SegmentKey::SegmentMetrics(segment_name.clone()))
        .unwrap_or(SegmentMetrics {
            segment_name,
            member_count: 0,
            avg_lifetime_value: 0,
            retention_rate: 0,
            churn_rate: 0,
            engagement_score: 0,
        })
}

/// Update segment-level aggregate metrics.
pub fn update_segment_metrics(
    env: &Env,
    segment: PlayerSegment,
    member_count: u32,
    avg_ltv: u64,
    retention: u32,
    churn: u32,
    engagement: u32,
) {
    let segment_name = segment_to_symbol(segment);
    let metrics = SegmentMetrics {
        segment_name,
        member_count,
        avg_lifetime_value: avg_ltv,
        retention_rate: retention,
        churn_rate: churn,
        engagement_score: engagement,
    };

    env.storage().persistent().set(
        &SegmentKey::SegmentMetrics(segment_to_symbol(segment)),
        &metrics,
    );

    env.events().publish(
        (symbol_short!("seg"), symbol_short!("metrics")),
        (member_count, avg_ltv, engagement),
    );
}
