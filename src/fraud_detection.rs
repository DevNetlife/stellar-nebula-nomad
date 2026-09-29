//! # Fraud Detection System (#372)
//!
//! Lightweight anomaly detection, bot identification, and abuse prevention
//! layered on top of player action history. This complements
//! `bot_detection.rs` (which focuses on action-timing/CAPTCHA flows) by
//! providing a general-purpose fraud score derived from statistical
//! deviation of a player's recent transaction volumes, plus a blocking
//! mechanism for confirmed abuse.
//!
//! ## How It Works
//!
//! Callers report per-player transaction "events" (e.g. trades, claims,
//! transfers) via [`record_event`]. The module keeps a rolling window of
//! recent event magnitudes per player and computes a simple z-score style
//! deviation against the player's own historical average. Players whose
//! deviation or event frequency crosses configured thresholds are flagged
//! as suspicious; repeated flags escalate them to blocked (abuse
//! prevention), which callers can check via [`is_blocked`] before allowing
//! sensitive actions.

use soroban_sdk::{contracterror, contracttype, Address, Env, Vec};

use crate::error_standard::{ErrorDescriptor, ErrorKind, StandardContractError};

// ── Error ───────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FraudError {
    /// Player is currently blocked from performing the requested action.
    PlayerBlocked = 1,
}

impl StandardContractError for FraudError {
    fn descriptor(self) -> ErrorDescriptor {
        ErrorDescriptor {
            module: "fraud_detection",
            code: self as u32,
            kind: ErrorKind::Authorization,
            retryable: false,
        }
    }
}

// ── Storage Keys ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum FraudKey {
    /// Rolling window of recent event magnitudes for a player.
    EventWindow(Address),
    /// Current fraud profile (score, flag count, blocked state) for a player.
    Profile(Address),
}

// ── Constants ───────────────────────────────────────────────────────────

/// Number of recent events retained per player for deviation analysis.
const WINDOW_SIZE: u32 = 20;
/// Minimum number of samples required before deviation scoring kicks in.
const MIN_SAMPLES: u32 = 4;
/// Deviation ratio (in percent of the player's own average) above which a
/// single event is considered anomalous. E.g. 300 = event is 3x the average.
const ANOMALY_RATIO_PCT: u64 = 300;
/// Fraud score added per anomalous event.
const ANOMALY_SCORE_INCREMENT: u32 = 25;
/// Fraud score above which a player is flagged as suspicious (bot-like).
const SUSPICION_THRESHOLD: u32 = 50;
/// Number of times a player must be flagged as suspicious before being
/// automatically blocked (escalating abuse prevention).
const FLAGS_BEFORE_BLOCK: u32 = 3;
/// Maximum fraud score.
const MAX_SCORE: u32 = 100;
/// Score decay applied each time an event is recorded without anomaly.
const SCORE_DECAY: u32 = 5;

// ── Data Types ──────────────────────────────────────────────────────────

/// Per-player fraud tracking profile.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct FraudProfile {
    pub score: u32,
    pub flag_count: u32,
    pub blocked: bool,
    pub total_events: u64,
}

impl Default for FraudProfile {
    fn default() -> Self {
        FraudProfile {
            score: 0,
            flag_count: 0,
            blocked: false,
            total_events: 0,
        }
    }
}

/// Outcome of recording a single event.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct FraudCheckResult {
    pub score: u32,
    pub anomalous: bool,
    pub blocked: bool,
    /// True if this event caused the player to become bot-like flagged
    /// (crossed `SUSPICION_THRESHOLD`) or is a repeat pattern.
    pub bot_suspected: bool,
}

// ── Core Logic ──────────────────────────────────────────────────────────

/// Record a transaction/action event of the given `magnitude` (e.g. token
/// amount, item count) for `player` and update their fraud profile.
///
/// Returns the resulting fraud check outcome. Does not itself block the
/// action being reported — callers should combine this with
/// [`enforce_not_blocked`] as needed.
pub fn record_event(env: &Env, player: &Address, magnitude: u64) -> FraudCheckResult {
    let mut window: Vec<u64> = env
        .storage()
        .temporary()
        .get(&FraudKey::EventWindow(player.clone()))
        .unwrap_or_else(|| Vec::new(env));

    let mut profile: FraudProfile = env
        .storage()
        .persistent()
        .get(&FraudKey::Profile(player.clone()))
        .unwrap_or_default();

    let sample_count = window.len();
    let anomalous = if sample_count >= MIN_SAMPLES {
        let sum: u64 = window.iter().sum();
        let avg = sum / sample_count as u64;
        if avg == 0 {
            false
        } else {
            // magnitude / avg * 100 >= ANOMALY_RATIO_PCT
            let ratio_pct = (magnitude.saturating_mul(100)) / avg;
            ratio_pct >= ANOMALY_RATIO_PCT
        }
    } else {
        false
    };

    if anomalous {
        profile.score = core::cmp::min(MAX_SCORE, profile.score + ANOMALY_SCORE_INCREMENT);
    } else {
        profile.score = profile.score.saturating_sub(SCORE_DECAY);
    }

    let mut bot_suspected = false;
    if profile.score >= SUSPICION_THRESHOLD {
        bot_suspected = true;
        profile.flag_count = profile.flag_count.saturating_add(1);
        profile.score = 0; // reset after flag so score reflects fresh behavior
        if profile.flag_count >= FLAGS_BEFORE_BLOCK {
            profile.blocked = true;
        }
    }

    profile.total_events = profile.total_events.saturating_add(1);

    // Push magnitude into the rolling window, evicting oldest if full.
    if window.len() >= WINDOW_SIZE {
        window.remove(0);
    }
    window.push_back(magnitude);

    env.storage()
        .temporary()
        .set(&FraudKey::EventWindow(player.clone()), &window);
    env.storage()
        .persistent()
        .set(&FraudKey::Profile(player.clone()), &profile);

    FraudCheckResult {
        score: profile.score,
        anomalous,
        blocked: profile.blocked,
        bot_suspected,
    }
}

/// Return the current fraud profile for `player` (defaults if unseen).
pub fn get_profile(env: &Env, player: &Address) -> FraudProfile {
    env.storage()
        .persistent()
        .get(&FraudKey::Profile(player.clone()))
        .unwrap_or_default()
}

/// True if `player` is currently blocked from sensitive actions.
pub fn is_blocked(env: &Env, player: &Address) -> bool {
    get_profile(env, player).blocked
}

/// Convenience guard: returns `Err(FraudError::PlayerBlocked)` if the player
/// is currently blocked, otherwise `Ok(())`. Intended to be called at the
/// top of sensitive contract entry points (trading, minting, withdrawals).
pub fn enforce_not_blocked(env: &Env, player: &Address) -> Result<(), FraudError> {
    if is_blocked(env, player) {
        return Err(FraudError::PlayerBlocked);
    }
    Ok(())
}

/// Administratively clear a player's blocked/flagged state (e.g. after
/// manual review clears them of abuse).
pub fn clear_block(env: &Env, player: &Address) {
    let mut profile = get_profile(env, player);
    profile.blocked = false;
    profile.flag_count = 0;
    profile.score = 0;
    env.storage()
        .persistent()
        .set(&FraudKey::Profile(player.clone()), &profile);
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Env};

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
    fn test_normal_events_do_not_flag() {
        let (env, contract_id) = make_env();
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            for _ in 0..10 {
                let result = record_event(&env, &player, 100);
                assert!(!result.anomalous);
                assert!(!result.blocked);
            }
            assert!(!is_blocked(&env, &player));
        });
    }

    #[test]
    fn test_anomalous_spike_flags_player() {
        let (env, contract_id) = make_env();
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            for _ in 0..5 {
                record_event(&env, &player, 100);
            }
            // A huge spike relative to average of 100.
            let result = record_event(&env, &player, 10_000);
            assert!(result.anomalous);
        });
    }

    #[test]
    fn test_repeated_flags_escalate_to_block() {
        let (env, contract_id) = make_env();
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            for _ in 0..5 {
                record_event(&env, &player, 100);
            }
            // Trigger multiple anomalous spikes to accumulate flags.
            for _ in 0..(FLAGS_BEFORE_BLOCK * 2) {
                record_event(&env, &player, 100_000);
            }
            assert!(is_blocked(&env, &player));
            assert!(enforce_not_blocked(&env, &player).is_err());
        });
    }

    #[test]
    fn test_clear_block_resets_state() {
        let (env, contract_id) = make_env();
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            for _ in 0..5 {
                record_event(&env, &player, 100);
            }
            for _ in 0..(FLAGS_BEFORE_BLOCK * 2) {
                record_event(&env, &player, 100_000);
            }
            assert!(is_blocked(&env, &player));

            clear_block(&env, &player);
            assert!(!is_blocked(&env, &player));
            assert!(enforce_not_blocked(&env, &player).is_ok());
        });
    }

    #[test]
    fn test_unseen_player_not_blocked() {
        let (env, contract_id) = make_env();
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            assert!(!is_blocked(&env, &player));
            let profile = get_profile(&env, &player);
            assert_eq!(profile.total_events, 0);
        });
    }
}
