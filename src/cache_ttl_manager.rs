//! Time-to-live policy, invalidation and automatic pruning for cached
//! contract data.
//!
//! Every namespace keeps an index of the keys cached under it, so expired
//! entries can be found and deleted instead of lingering (and accruing rent)
//! forever. Pruning runs in two ways:
//!
//! - **Automatically**, piggy-backed on every [`cache_with_ttl`] write: up to
//!   [`AUTO_PRUNE_BATCH`] expired entries of the same namespace are removed,
//!   so the cost is amortised across writers and bounded per call.
//! - **Explicitly**, via [`prune_expired_entries`] / [`clear_stale_entries`],
//!   for a keeper to sweep a namespace that is no longer written to.
//!
//! An entry is prunable when its age exceeds its TTL or it was invalidated;
//! live entries are never touched. [`invalidate_namespace`] deletes the whole
//! namespace immediately.
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec};

// ─── Cache TTL Management System ────────────────────────────────────────────
//
// This module provides consistent TTL enforcement across all cached data,
// preventing stale financial data from being served and protecting the
// gaming economy from exploits.

// ─── Configuration ───────────────────────────────────────────────────────

/// Default cache TTL in seconds (5 minutes).
pub const DEFAULT_CACHE_TTL: u64 = 300;

/// Yield forecast cache TTL (15 minutes - market data updates).
pub const YIELD_FORECAST_TTL: u64 = 900;

/// Market oracle price cache TTL (10 minutes).
pub const MARKET_ORACLE_TTL: u64 = 600;

/// State snapshot cache TTL (30 minutes).
pub const STATE_SNAPSHOT_TTL: u64 = 1800;

/// Leaderboard cache TTL (1 hour).
pub const LEADERBOARD_TTL: u64 = 3600;

/// Player profile cache TTL (15 minutes).
pub const PLAYER_PROFILE_TTL: u64 = 900;

/// Analytics cache TTL (5 minutes - hot data).
pub const ANALYTICS_TTL: u64 = 300;

/// Maximum number of keys tracked per namespace. Bounds the size of the
/// namespace index and therefore the cost of a full sweep: at this size
/// [`invalidate_namespace`] stays well inside the per-transaction ledger
/// write limit (200 entries).
pub const MAX_KEYS_PER_NAMESPACE: u32 = 100;

/// Expired entries removed opportunistically on each [`cache_with_ttl`] call.
pub const AUTO_PRUNE_BATCH: u32 = 5;

/// Upper bound on entries removed by a single explicit prune call.
pub const MAX_PRUNE_BATCH: u32 = 50;

/// Keys examined per entry a prune call may remove. Scanning resumes where
/// the previous call stopped, so repeated calls cycle through the namespace
/// while each one reads a bounded number of entries.
const PRUNE_SCAN_FACTOR: u32 = 4;

// ─── Storage Keys ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum CacheKey {
    /// Cache entry with TTL: `CacheEntry(namespace, key)`.
    CacheEntry(Symbol, Symbol),
    /// Timestamp of last cache invalidation.
    LastInvalidation(Symbol),
    /// TTL configuration per cache type.
    TtlConfig(Symbol),
    /// Stale entry detection flag.
    IsStale(Symbol, Symbol),
    /// Keys currently cached under a namespace, in insertion order.
    NamespaceIndex(Symbol),
    /// Index position where the next prune of a namespace starts scanning.
    PruneCursor(Symbol),
}

// ─── Error Types ─────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CacheTtlError {
    /// Cache entry has expired.
    CacheExpired = 1,
    /// Cache entry not found.
    EntryNotFound = 2,
    /// Invalid TTL value.
    InvalidTtl = 3,
    /// Unauthorized cache operation.
    Unauthorized = 4,
    /// Cache validation failed.
    ValidationFailed = 5,
    /// The namespace already tracks [`MAX_KEYS_PER_NAMESPACE`] live keys.
    NamespaceFull = 6,
}

impl crate::error_standard::StandardContractError for CacheTtlError {
    fn descriptor(self) -> crate::error_standard::ErrorDescriptor {
        use crate::error_standard::ErrorKind;
        let (kind, retryable) = match self {
            Self::CacheExpired => (ErrorKind::Conflict, false),
            Self::EntryNotFound => (ErrorKind::NotFound, false),
            Self::InvalidTtl => (ErrorKind::Validation, false),
            Self::Unauthorized => (ErrorKind::Authorization, false),
            Self::ValidationFailed => (ErrorKind::Internal, false),
            Self::NamespaceFull => (ErrorKind::ResourceLimit, true),
        };
        crate::error_standard::ErrorDescriptor {
            module: "cache_ttl_manager",
            code: self as u32,
            kind,
            retryable,
        }
    }
}

// ─── Data Structures ─────────────────────────────────────────────────────

/// Cached data with TTL metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct CachedData {
    pub namespace: Symbol,
    pub key: Symbol,
    pub value: Bytes,
    pub cached_at: u64,
    pub ttl_seconds: u64,
    pub is_valid: bool,
}

/// Cache invalidation event.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct InvalidationEvent {
    pub namespace: Symbol,
    pub reason: Symbol,
    pub invalidated_at: u64,
    pub affected_keys: u32,
}

/// TTL configuration per cache type.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct TtlConfig {
    pub namespace: Symbol,
    pub ttl_seconds: u64,
    pub auto_refresh: bool,
    pub max_age_for_refresh: u64,
}

// ─── Cache Operations ───────────────────────────────────────────────────

/// Store data with TTL enforcement.
///
/// Also prunes up to [`AUTO_PRUNE_BATCH`] expired entries of `namespace`, so
/// a namespace that keeps being written to never accumulates dead entries.
///
/// # Errors
/// - [`CacheTtlError::InvalidTtl`] if `ttl_seconds` is zero.
/// - [`CacheTtlError::NamespaceFull`] if `key` is new and the namespace
///   already tracks [`MAX_KEYS_PER_NAMESPACE`] live keys.
pub fn cache_with_ttl(
    env: &Env,
    namespace: Symbol,
    key: Symbol,
    value: Bytes,
    ttl_seconds: u64,
) -> Result<(), CacheTtlError> {
    if ttl_seconds == 0 {
        return Err(CacheTtlError::InvalidTtl);
    }

    prune_expired_entries(env, namespace.clone(), AUTO_PRUNE_BATCH);

    let mut index = namespace_index(env, &namespace);
    if !index.contains(&key) {
        if index.len() >= MAX_KEYS_PER_NAMESPACE {
            return Err(CacheTtlError::NamespaceFull);
        }
        index.push_back(key.clone());
        env.storage()
            .persistent()
            .set(&CacheKey::NamespaceIndex(namespace.clone()), &index);
    }

    let cached = CachedData {
        namespace: namespace.clone(),
        key: key.clone(),
        value,
        cached_at: env.ledger().timestamp(),
        ttl_seconds,
        is_valid: true,
    };

    env.storage().persistent().set(
        &CacheKey::CacheEntry(namespace.clone(), key.clone()),
        &cached,
    );

    env.storage()
        .instance()
        .set(&CacheKey::IsStale(namespace.clone(), key), &false);

    env.events().publish(
        (symbol_short!("cache"), symbol_short!("stored")),
        (namespace, ttl_seconds, env.ledger().timestamp()),
    );

    Ok(())
}

/// Retrieve cached data with expiry validation.
pub fn get_cached_with_ttl(
    env: &Env,
    namespace: Symbol,
    key: Symbol,
) -> Result<Bytes, CacheTtlError> {
    let cached: Option<CachedData> = env
        .storage()
        .persistent()
        .get(&CacheKey::CacheEntry(namespace.clone(), key.clone()));

    match cached {
        None => Err(CacheTtlError::EntryNotFound),
        Some(entry) => {
            let current_time = env.ledger().timestamp();
            let age = current_time.saturating_sub(entry.cached_at);
            let invalidated: bool = env
                .storage()
                .instance()
                .get(&CacheKey::IsStale(namespace.clone(), key.clone()))
                .unwrap_or(false);

            if invalidated || !entry.is_valid || age > entry.ttl_seconds {
                // Mark as stale and emit event.
                env.storage()
                    .instance()
                    .set(&CacheKey::IsStale(namespace.clone(), key.clone()), &true);

                env.events().publish(
                    (symbol_short!("cache"), symbol_short!("expired")),
                    (namespace, key, current_time),
                );

                Err(CacheTtlError::CacheExpired)
            } else {
                Ok(entry.value)
            }
        }
    }
}

/// Check if cached entry is still valid without retrieving it.
pub fn is_cache_valid(env: &Env, namespace: Symbol, key: Symbol) -> bool {
    let cached: Option<CachedData> = env
        .storage()
        .persistent()
        .get(&CacheKey::CacheEntry(namespace.clone(), key.clone()));

    match cached {
        None => false,
        Some(entry) => {
            let current_time = env.ledger().timestamp();
            let age = current_time.saturating_sub(entry.cached_at);
            let invalidated: bool = env
                .storage()
                .instance()
                .get(&CacheKey::IsStale(namespace, key))
                .unwrap_or(false);
            !invalidated && entry.is_valid && age <= entry.ttl_seconds
        }
    }
}

/// Get remaining TTL for a cache entry.
pub fn get_remaining_ttl(env: &Env, namespace: Symbol, key: Symbol) -> Result<u64, CacheTtlError> {
    let cached: Option<CachedData> = env
        .storage()
        .persistent()
        .get(&CacheKey::CacheEntry(namespace, key));

    match cached {
        None => Err(CacheTtlError::EntryNotFound),
        Some(entry) => {
            let current_time = env.ledger().timestamp();
            let age = current_time.saturating_sub(entry.cached_at);

            if age >= entry.ttl_seconds {
                Ok(0)
            } else {
                Ok(entry.ttl_seconds - age)
            }
        }
    }
}

// ─── Cache Invalidation ──────────────────────────────────────────────────

/// Invalidate a specific cache entry.
pub fn invalidate_cache_entry(env: &Env, namespace: Symbol, key: Symbol, reason: Symbol) {
    env.storage()
        .instance()
        .set(&CacheKey::IsStale(namespace.clone(), key.clone()), &true);

    env.events().publish(
        (symbol_short!("cache"), symbol_short!("inv")),
        (namespace, key, reason, env.ledger().timestamp()),
    );
}

/// Invalidate all cache entries in a namespace.
///
/// Invalidated entries can never be served again, so they are deleted right
/// away rather than flagged and left for a later prune. Returns how many
/// entries were removed.
pub fn invalidate_namespace(env: &Env, namespace: Symbol, reason: Symbol) -> u32 {
    env.storage().instance().set(
        &CacheKey::LastInvalidation(namespace.clone()),
        &env.ledger().timestamp(),
    );

    let index = namespace_index(env, &namespace);
    for key in index.iter() {
        remove_entry(env, &namespace, key);
    }
    env.storage()
        .persistent()
        .remove(&CacheKey::NamespaceIndex(namespace.clone()));
    env.storage()
        .instance()
        .remove(&CacheKey::PruneCursor(namespace.clone()));

    env.events().publish(
        (symbol_short!("cache"), symbol_short!("ns_clr")),
        (namespace, reason, env.ledger().timestamp()),
    );

    index.len()
}

/// Remove every expired entry of `namespace`, up to [`MAX_PRUNE_BATCH`].
///
/// Intended to be called periodically by a keeper. Returns the number of
/// entries removed; call again while it returns [`MAX_PRUNE_BATCH`].
pub fn clear_stale_entries(env: &Env, namespace: Symbol) -> u32 {
    prune_expired_entries(env, namespace, MAX_PRUNE_BATCH)
}

/// Remove up to `max_entries` expired entries of `namespace`.
///
/// Deletes the cached value, its stale flag and its slot in the namespace
/// index. Live entries are left untouched. `max_entries` is clamped to
/// [`MAX_PRUNE_BATCH`], and at most `max_entries * 4` keys are examined,
/// starting where the previous call stopped, so one call has a bounded cost.
///
/// Emits `cache/cleanup` with `(namespace, removed, timestamp)` when anything
/// was removed.
pub fn prune_expired_entries(env: &Env, namespace: Symbol, max_entries: u32) -> u32 {
    let limit = max_entries.min(MAX_PRUNE_BATCH);
    let index = namespace_index(env, &namespace);
    let len = index.len();
    if limit == 0 || len == 0 {
        return 0;
    }

    let cursor_key = CacheKey::PruneCursor(namespace.clone());
    let start = env
        .storage()
        .instance()
        .get::<_, u32>(&cursor_key)
        .unwrap_or(0)
        % len;
    let scan = len.min(limit.saturating_mul(PRUNE_SCAN_FACTOR));

    let mut pruned: Vec<Symbol> = Vec::new(env);
    let mut scanned = 0u32;
    while scanned < scan && pruned.len() < limit {
        let key = index.get_unchecked((start + scanned) % len);
        if is_prunable(env, &namespace, &key) {
            remove_entry(env, &namespace, key.clone());
            pruned.push_back(key);
        }
        scanned += 1;
    }

    let removed = pruned.len();
    let remaining = len - removed;
    if remaining == 0 {
        env.storage().instance().remove(&cursor_key);
    } else {
        // Entries removed before the resume point shift it left; an
        // approximate position is fine because scanning wraps around anyway.
        env.storage()
            .instance()
            .set(&cursor_key, &((start + scanned - removed) % remaining));
    }

    if removed == 0 {
        return 0;
    }

    let index_key = CacheKey::NamespaceIndex(namespace.clone());
    if remaining == 0 {
        env.storage().persistent().remove(&index_key);
    } else {
        let mut kept: Vec<Symbol> = Vec::new(env);
        for key in index.iter() {
            if !pruned.contains(&key) {
                kept.push_back(key);
            }
        }
        env.storage().persistent().set(&index_key, &kept);
    }

    env.events().publish(
        (symbol_short!("cache"), symbol_short!("cleanup")),
        (namespace, removed, env.ledger().timestamp()),
    );

    removed
}

fn namespace_index(env: &Env, namespace: &Symbol) -> Vec<Symbol> {
    env.storage()
        .persistent()
        .get(&CacheKey::NamespaceIndex(namespace.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

/// Delete a cached value and its stale flag. The caller owns the index.
fn remove_entry(env: &Env, namespace: &Symbol, key: Symbol) {
    env.storage()
        .persistent()
        .remove(&CacheKey::CacheEntry(namespace.clone(), key.clone()));
    env.storage()
        .instance()
        .remove(&CacheKey::IsStale(namespace.clone(), key));
}

/// `true` if the entry under `(namespace, key)` is missing, expired or
/// invalidated, i.e. nothing would ever be served from it again.
fn is_prunable(env: &Env, namespace: &Symbol, key: &Symbol) -> bool {
    let Some(entry) = env
        .storage()
        .persistent()
        .get::<_, CachedData>(&CacheKey::CacheEntry(namespace.clone(), key.clone()))
    else {
        return true;
    };
    let age = env.ledger().timestamp().saturating_sub(entry.cached_at);
    let flagged: bool = env
        .storage()
        .instance()
        .get(&CacheKey::IsStale(namespace.clone(), key.clone()))
        .unwrap_or(false);
    flagged || !entry.is_valid || age > entry.ttl_seconds
}

// ─── TTL Configuration ──────────────────────────────────────────────────

/// Set TTL configuration for a cache type.
pub fn configure_ttl(
    env: &Env,
    admin: &Address,
    namespace: Symbol,
    ttl_seconds: u64,
    auto_refresh: bool,
) -> Result<(), CacheTtlError> {
    admin.require_auth();

    if ttl_seconds == 0 {
        return Err(CacheTtlError::InvalidTtl);
    }

    let config = TtlConfig {
        namespace: namespace.clone(),
        ttl_seconds,
        auto_refresh,
        max_age_for_refresh: ttl_seconds / 2,
    };

    env.storage()
        .instance()
        .set(&CacheKey::TtlConfig(namespace.clone()), &config);

    env.events().publish(
        (symbol_short!("cache"), symbol_short!("cfg")),
        (
            namespace,
            ttl_seconds,
            auto_refresh,
            env.ledger().timestamp(),
        ),
    );

    Ok(())
}

/// Get TTL configuration for a namespace.
pub fn get_ttl_config(env: &Env, namespace: Symbol) -> TtlConfig {
    env.storage()
        .instance()
        .get(&CacheKey::TtlConfig(namespace.clone()))
        .unwrap_or(TtlConfig {
            namespace,
            ttl_seconds: DEFAULT_CACHE_TTL,
            auto_refresh: true,
            max_age_for_refresh: DEFAULT_CACHE_TTL / 2,
        })
}

// ─── Health Checks ──────────────────────────────────────────────────────

/// Detect stale data and flag for invalidation.
pub fn detect_stale_data(env: &Env, namespace: Symbol, key: Symbol) -> bool {
    env.storage()
        .instance()
        .get(&CacheKey::IsStale(namespace, key))
        .unwrap_or(false)
}

/// Get cache statistics (for monitoring).
///
/// Returns `(total_entries, stale_entries)` for `namespace`, where
/// `stale_entries` is how many of them the next prune would remove.
pub fn get_cache_stats(env: &Env, namespace: Symbol) -> (u32, u32) {
    let index = namespace_index(env, &namespace);
    let mut stale = 0u32;
    for key in index.iter() {
        if is_prunable(env, &namespace, &key) {
            stale += 1;
        }
    }
    (index.len(), stale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{contract, contractimpl};

    #[contract]
    struct Stub;
    #[contractimpl]
    impl Stub {}

    fn setup() -> (Env, Address) {
        let env = Env::default();
        env.ledger().set_timestamp(1_000);
        let id = env.register(Stub, ());
        (env, id)
    }

    fn ns() -> Symbol {
        symbol_short!("prices")
    }

    fn value(env: &Env, byte: u8) -> Bytes {
        Bytes::from_array(env, &[byte; 4])
    }

    fn advance(env: &Env, secs: u64) {
        env.ledger().with_mut(|li| li.timestamp += secs);
    }

    #[test]
    fn prune_removes_only_expired_entries() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("short"), value(&env, 1), 10).unwrap();
            cache_with_ttl(&env, ns(), symbol_short!("long"), value(&env, 2), 1_000).unwrap();

            advance(&env, 11);
            assert_eq!(get_cache_stats(&env, ns()), (2, 1));

            assert_eq!(prune_expired_entries(&env, ns(), MAX_PRUNE_BATCH), 1);
            assert_eq!(get_cache_stats(&env, ns()), (1, 0));

            // The expired entry is gone from storage, not just flagged.
            assert_eq!(
                get_cached_with_ttl(&env, ns(), symbol_short!("short")),
                Err(CacheTtlError::EntryNotFound)
            );
            // The live entry is intact, byte for byte.
            assert_eq!(
                get_cached_with_ttl(&env, ns(), symbol_short!("long")),
                Ok(value(&env, 2))
            );
        });
    }

    #[test]
    fn prune_is_a_noop_when_nothing_expired() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 1), 100).unwrap();
            assert_eq!(clear_stale_entries(&env, ns()), 0);
            assert_eq!(get_cache_stats(&env, ns()), (1, 0));
            assert!(is_cache_valid(&env, ns(), symbol_short!("a")));
        });
    }

    #[test]
    fn prune_removes_individually_invalidated_entries() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 1), 1_000).unwrap();
            invalidate_cache_entry(&env, ns(), symbol_short!("a"), symbol_short!("manual"));
            assert_eq!(get_cache_stats(&env, ns()), (1, 1));
            assert_eq!(clear_stale_entries(&env, ns()), 1);
            assert!(!detect_stale_data(&env, ns(), symbol_short!("a")));
            assert_eq!(get_cache_stats(&env, ns()), (0, 0));
        });
    }

    #[test]
    fn writes_automatically_prune_expired_entries() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("old"), value(&env, 1), 5).unwrap();
            advance(&env, 6);

            // No explicit prune: the next write to the namespace cleans up.
            cache_with_ttl(&env, ns(), symbol_short!("new"), value(&env, 2), 100).unwrap();
            assert_eq!(get_cache_stats(&env, ns()), (1, 0));
            assert_eq!(
                get_cached_with_ttl(&env, ns(), symbol_short!("old")),
                Err(CacheTtlError::EntryNotFound)
            );
        });
    }

    #[test]
    fn automatic_pruning_is_bounded_per_write() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let keys = [
                symbol_short!("k0"),
                symbol_short!("k1"),
                symbol_short!("k2"),
                symbol_short!("k3"),
                symbol_short!("k4"),
                symbol_short!("k5"),
                symbol_short!("k6"),
            ];
            for key in &keys {
                cache_with_ttl(&env, ns(), key.clone(), value(&env, 0), 1).unwrap();
            }
            advance(&env, 2);

            cache_with_ttl(&env, ns(), symbol_short!("fresh"), value(&env, 9), 100).unwrap();
            let (total, stale) = get_cache_stats(&env, ns());
            assert_eq!(stale, 7 - AUTO_PRUNE_BATCH);
            assert_eq!(total, stale + 1);
        });
    }

    #[test]
    fn prune_respects_the_requested_batch_size() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            for key in [symbol_short!("a"), symbol_short!("b"), symbol_short!("c")] {
                cache_with_ttl(&env, ns(), key, value(&env, 0), 1).unwrap();
            }
            advance(&env, 2);
            assert_eq!(prune_expired_entries(&env, ns(), 2), 2);
            assert_eq!(prune_expired_entries(&env, ns(), 2), 1);
            assert_eq!(prune_expired_entries(&env, ns(), 2), 0);
            assert_eq!(get_cache_stats(&env, ns()), (0, 0));
        });
    }

    #[test]
    fn rewriting_a_key_does_not_duplicate_it_in_the_index() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 1), 10).unwrap();
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 2), 10).unwrap();
            assert_eq!(get_cache_stats(&env, ns()), (1, 0));
            assert_eq!(
                get_cached_with_ttl(&env, ns(), symbol_short!("a")),
                Ok(value(&env, 2))
            );
        });
    }

    #[test]
    fn namespaces_are_pruned_independently() {
        let (env, id) = setup();
        let other = symbol_short!("boards");
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 1), 1).unwrap();
            cache_with_ttl(&env, other.clone(), symbol_short!("a"), value(&env, 2), 1).unwrap();
            advance(&env, 2);

            assert_eq!(clear_stale_entries(&env, ns()), 1);
            assert_eq!(get_cache_stats(&env, other.clone()), (1, 1));
        });
    }

    #[test]
    fn invalidate_namespace_deletes_every_entry() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 1), 100).unwrap();
            cache_with_ttl(&env, ns(), symbol_short!("b"), value(&env, 2), 100).unwrap();

            assert_eq!(invalidate_namespace(&env, ns(), symbol_short!("reset")), 2);
            assert_eq!(get_cache_stats(&env, ns()), (0, 0));
            assert!(!is_cache_valid(&env, ns(), symbol_short!("a")));

            // The namespace is usable again straight away, in the same ledger.
            cache_with_ttl(&env, ns(), symbol_short!("a"), value(&env, 3), 100).unwrap();
            assert_eq!(
                get_cached_with_ttl(&env, ns(), symbol_short!("a")),
                Ok(value(&env, 3))
            );
        });
    }

    #[test]
    fn full_namespace_rejects_new_keys_but_accepts_updates() {
        let (env, id) = setup();
        // Each write is its own invocation, as it would be on-chain.
        let key = |i: u32| Symbol::new(&env, &std::format!("k{i}"));
        for i in 0..MAX_KEYS_PER_NAMESPACE {
            env.as_contract(&id, || {
                cache_with_ttl(&env, ns(), key(i), value(&env, 0), 1_000).unwrap();
            });
        }
        env.as_contract(&id, || {
            assert_eq!(
                cache_with_ttl(&env, ns(), symbol_short!("extra"), value(&env, 0), 1_000),
                Err(CacheTtlError::NamespaceFull)
            );
            // An existing key can still be refreshed.
            cache_with_ttl(&env, ns(), key(0), value(&env, 1), 1_000).unwrap();
        });

        // Once entries expire, the next write reclaims their slots.
        advance(&env, 1_001);
        env.as_contract(&id, || {
            cache_with_ttl(&env, ns(), symbol_short!("extra"), value(&env, 0), 1_000).unwrap();
            let (total, stale) = get_cache_stats(&env, ns());
            assert_eq!(total, MAX_KEYS_PER_NAMESPACE + 1 - AUTO_PRUNE_BATCH);
            assert_eq!(stale, total - 1);
        });
    }
}
