//! System limits, page sizes, and rate limiting thresholds.

/// Maximum results returned per page in leaderboard queries.
/// Rationale: Optimizes gas usage for Soroban RPC call responses.
/// Unit: Item count.
pub const LEADERBOARD_PAGE_SIZE: u32 = 50;

/// Rate limit maximum requests per account per window.
/// Rationale: Prevents denial-of-service spam on contract state endpoints.
/// Unit: Request count.
pub const RATE_LIMIT_MAX_REQUESTS: u32 = 100;

/// Maximum sliding history size for price oracle points.
/// Rationale: Bounds TWAP array storage in contract instance memory.
/// Unit: History record count.
pub const MAX_PRICE_HISTORY_POINTS: u32 = 100;

/// Maximum hourly price adjustment percentage limit.
/// Rationale: Prevents volatility flash loan manipulation in dynamic pricing.
/// Unit: Percentage (0..100).
pub const MAX_HOURLY_PRICE_CHANGE_PERCENT: u32 = 5;

/// Circuit breaker threshold percentage for extreme price jumps.
/// Rationale: Triggers automatic freeze if price swings exceed this limit in a single block.
/// Unit: Percentage (0..100).
pub const CIRCUIT_BREAKER_THRESHOLD_PERCENT: u32 = 20;
