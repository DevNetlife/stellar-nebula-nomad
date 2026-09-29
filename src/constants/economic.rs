//! Economic and marketplace constants.

/// Base crafting fee percentage applied to non-guild members.
/// Rationale: Creates a 5% sink on economic transactions to prevent hyper-inflation.
/// Unit: Basis Points (bps, where 100 bps = 1%).
pub const BASE_CRAFTING_FEE_BPS: u32 = 500;

/// Default efficiency retention factor during recycling operations (70%).
/// Rationale: Imposes a 30% resource destruction loss on recycling actions.
/// Unit: Percentage (0..100).
pub const RECYCLING_EFFICIENCY_PERCENT: u32 = 70;

/// Default recycling loss factor (30%).
/// Rationale: Net destroyed percentage when breaking down items.
/// Unit: Percentage (0..100).
pub const RECYCLING_LOSS_PERCENT: u32 = 30;

/// Base repair cost multiplier per ship level.
/// Rationale: Ensures higher-tier ships require proportionally more resources for maintenance.
/// Unit: Resource units per level.
pub const BASE_REPAIR_COST_PER_LEVEL: u64 = 50;

/// Default tournament entry fee in base resource units.
/// Rationale: Resource sink for competitive player vs player events.
/// Unit: Resource tokens.
pub const TOURNAMENT_BASE_ENTRY_FEE: u128 = 1000;
