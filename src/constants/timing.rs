//! Timing, cooldown, and duration constants.

/// Cooldown duration between special ability activations.
/// Rationale: Prevents ability spamming in single combat encounters.
/// Unit: Seconds.
pub const SPECIAL_ABILITY_COOLDOWN_SECS: u64 = 3600;

/// Window duration for rate limiting tracking.
/// Rationale: Time interval over which request counts are evaluated.
/// Unit: Seconds.
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;

/// Default session timeout duration for active player sessions.
/// Rationale: Automatically expires inactive session permissions.
/// Unit: Seconds.
pub const SESSION_TIMEOUT_SECS: u64 = 86400; // 24 hours
