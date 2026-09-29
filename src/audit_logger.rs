//! Structured, append-only audit event recording with bounded retention.
//!
//! Entries are written with sequential IDs and never modified. Old entries are
//! pruned from the front of the log according to a [`RetentionPolicy`]: an
//! entry goes once it is older than `max_age_secs`, or once more than
//! `max_entries` newer entries exist. Pruning happens automatically — each
//! [`log_audit_event`] removes up to [`AUTO_PRUNE_BATCH`] expired entries —
//! and can be driven explicitly with [`prune_audit_logs`].
//!
//! Because pruning only ever removes the oldest entries, the retained log is
//! always the contiguous ID range `[oldest_audit_id, get_audit_count)`.
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec};

pub const MAX_QUERY_LIMIT: u32 = 1000;

/// Default retention window: 30 days.
pub const DEFAULT_AUDIT_RETENTION_SECS: u64 = 30 * 24 * 60 * 60;

/// Default cap on retained entries. Entries live in instance storage, which
/// is loaded on every invocation, so the cap keeps that entry small.
pub const DEFAULT_MAX_AUDIT_ENTRIES: u64 = 256;

/// Entries pruned opportunistically on each [`log_audit_event`] call. Larger
/// than one so a backlog (e.g. after tightening the policy) drains over time.
pub const AUTO_PRUNE_BATCH: u32 = 4;

/// Upper bound on entries removed by a single [`prune_audit_logs`] call.
pub const MAX_PRUNE_BATCH: u32 = 100;

#[derive(Clone)]
#[contracttype]
pub enum AuditLoggerKey {
    Counter,
    Entry(u64),
    /// ID of the oldest entry not yet pruned.
    Oldest,
    /// Active [`RetentionPolicy`].
    Retention,
}

/// How long audit entries are kept.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct RetentionPolicy {
    /// Entries older than this many seconds are pruned. Must be `> 0`.
    pub max_age_secs: u64,
    /// At most this many of the newest entries are kept. Must be `> 0`.
    pub max_entries: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub actor: Option<Address>,
    pub action: Symbol,
    pub details: BytesN<128>,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuditLoggerError {
    LogWriteFailed = 1,
    QueryLimitExceeded = 2,
    InvalidFilter = 3,
    /// A retention policy field was zero.
    InvalidRetention = 4,
    /// Caller does not hold the admin role.
    Unauthorized = 5,
}

impl crate::error_standard::StandardContractError for AuditLoggerError {
    fn descriptor(self) -> crate::error_standard::ErrorDescriptor {
        use crate::error_standard::ErrorKind;
        let (kind, retryable) = match self {
            Self::LogWriteFailed => (ErrorKind::Internal, false),
            Self::QueryLimitExceeded => (ErrorKind::ResourceLimit, false),
            Self::InvalidFilter | Self::InvalidRetention => (ErrorKind::Validation, false),
            Self::Unauthorized => (ErrorKind::Authorization, false),
        };
        crate::error_standard::ErrorDescriptor {
            module: "audit_logger",
            code: self as u32,
            kind,
            retryable,
        }
    }
}

pub fn log_audit_event(
    env: &Env,
    actor: Option<&Address>,
    action: Symbol,
    details: BytesN<128>,
) -> Result<AuditEntry, AuditLoggerError> {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&AuditLoggerKey::Counter)
        .unwrap_or(0);

    let entry = AuditEntry {
        id: current_id,
        timestamp: env.ledger().timestamp(),
        actor: actor.cloned(),
        action: action.clone(),
        details: details.clone(),
    };

    env.storage()
        .instance()
        .set(&AuditLoggerKey::Entry(current_id), &entry);
    env.storage()
        .instance()
        .set(&AuditLoggerKey::Counter, &(current_id + 1));

    env.events().publish(
        (symbol_short!("audit"), symbol_short!("entry")),
        (
            entry.id,
            entry.timestamp,
            entry.actor.clone(),
            entry.action.clone(),
            entry.details.clone(),
        ),
    );

    prune_audit_logs(env, AUTO_PRUNE_BATCH);

    Ok(entry)
}

pub fn query_audit_logs(
    env: &Env,
    filter: Symbol,
    limit: u32,
) -> Result<Vec<AuditEntry>, AuditLoggerError> {
    let capped_limit = core::cmp::min(limit, MAX_QUERY_LIMIT);
    let total: u64 = env
        .storage()
        .instance()
        .get(&AuditLoggerKey::Counter)
        .unwrap_or(0);
    let oldest = oldest_audit_id(env);
    let retained = total - oldest;

    let max = if capped_limit == 0 {
        core::cmp::min(retained, u64::from(MAX_QUERY_LIMIT))
    } else {
        core::cmp::min(retained, u64::from(capped_limit))
    };

    let mut results = Vec::new(env);
    let mut i = oldest;
    while i < oldest + max {
        if let Some(entry) = env
            .storage()
            .instance()
            .get::<AuditLoggerKey, AuditEntry>(&AuditLoggerKey::Entry(i))
        {
            if filter == entry.action.clone() {
                results.push_back(entry);
            }
        }
        i += 1;
    }

    Ok(results)
}

/// Total number of entries ever logged (also the next entry's ID). Pruning
/// does not decrease it; see [`get_retained_audit_count`].
pub fn get_audit_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&AuditLoggerKey::Counter)
        .unwrap_or(0)
}

/// ID of the oldest entry still stored.
pub fn oldest_audit_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&AuditLoggerKey::Oldest)
        .unwrap_or(0)
}

/// Number of entries currently stored.
pub fn get_retained_audit_count(env: &Env) -> u64 {
    get_audit_count(env) - oldest_audit_id(env)
}

/// Active retention policy (defaults when never configured).
pub fn get_audit_retention(env: &Env) -> RetentionPolicy {
    env.storage()
        .instance()
        .get(&AuditLoggerKey::Retention)
        .unwrap_or(RetentionPolicy {
            max_age_secs: DEFAULT_AUDIT_RETENTION_SECS,
            max_entries: DEFAULT_MAX_AUDIT_ENTRIES,
        })
}

/// Replace the retention policy. Requires `admin` to hold the admin role.
///
/// Tightening the policy does not delete anything synchronously; the backlog
/// drains through automatic pruning or [`prune_audit_logs`].
pub fn set_audit_retention(
    env: &Env,
    admin: &Address,
    policy: &RetentionPolicy,
) -> Result<(), AuditLoggerError> {
    admin.require_auth();
    if !crate::access_control::has_role(env, &crate::access_control::admin_role(), admin) {
        return Err(AuditLoggerError::Unauthorized);
    }
    if policy.max_age_secs == 0 || policy.max_entries == 0 {
        return Err(AuditLoggerError::InvalidRetention);
    }

    env.storage()
        .instance()
        .set(&AuditLoggerKey::Retention, policy);

    env.events().publish(
        (symbol_short!("audit"), symbol_short!("retain")),
        (policy.max_age_secs, policy.max_entries),
    );

    Ok(())
}

/// Remove up to `max_entries` of the oldest entries that fall outside the
/// retention policy. Returns the number removed.
///
/// Permissionless: it only ever deletes what the policy already says must
/// go, so anyone (typically a keeper) may call it. `max_entries` is clamped
/// to [`MAX_PRUNE_BATCH`].
pub fn prune_audit_logs(env: &Env, max_entries: u32) -> u32 {
    let limit = max_entries.min(MAX_PRUNE_BATCH);
    let policy = get_audit_retention(env);
    let now = env.ledger().timestamp();
    let total = get_audit_count(env);
    let start = oldest_audit_id(env);

    let mut next = start;
    let mut removed = 0u32;
    while removed < limit && next < total {
        let over_capacity = total - next > policy.max_entries;
        let key = AuditLoggerKey::Entry(next);
        if !over_capacity {
            let expired = env
                .storage()
                .instance()
                .get::<_, AuditEntry>(&key)
                .is_none_or(|entry| now.saturating_sub(entry.timestamp) > policy.max_age_secs);
            if !expired {
                // Entries are in timestamp order, so nothing newer is expired.
                break;
            }
        }
        env.storage().instance().remove(&key);
        next += 1;
        removed += 1;
    }

    if removed > 0 {
        env.storage().instance().set(&AuditLoggerKey::Oldest, &next);
        env.events().publish(
            (symbol_short!("audit"), symbol_short!("pruned")),
            (start, next, now),
        );
    }

    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{contract, contractimpl, testutils::Address as _};

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
    fn test_log_and_query_roundtrip() {
        let (env, id) = make_env();
        let player = Address::generate(&env);
        let action = symbol_short!("scan");
        let details = BytesN::from_array(&env, &[1u8; 128]);

        env.as_contract(&id, || {
            let entry = log_audit_event(&env, Some(&player), action.clone(), details.clone())
                .expect("log should succeed");
            assert_eq!(entry.actor, Some(player.clone()));
            assert_eq!(entry.action, action);

            let results = query_audit_logs(&env, action, 10).expect("query should succeed");
            assert_eq!(results.len(), 1);
            assert_eq!(results.get(0).unwrap().id, entry.id);
        });
    }

    #[test]
    fn test_query_limit_capped() {
        let (env, id) = make_env();
        let player = Address::generate(&env);
        let action = symbol_short!("test");
        let details = BytesN::from_array(&env, &[0u8; 128]);

        env.as_contract(&id, || {
            for _ in 0..5 {
                let _ = log_audit_event(&env, Some(&player), action.clone(), details.clone());
            }

            let results = query_audit_logs(&env, action, 3).expect("query should succeed");
            assert_eq!(results.len(), 3);
        });
    }

    #[test]
    fn test_query_limit_does_not_exceed_max() {
        let (env, id) = make_env();
        let player = Address::generate(&env);
        let action = symbol_short!("bulk");
        let details = BytesN::from_array(&env, &[0u8; 128]);

        env.as_contract(&id, || {
            for _ in 0..10 {
                let _ = log_audit_event(&env, Some(&player), action.clone(), details.clone());
            }

            let huge_limit = MAX_QUERY_LIMIT + 5000;
            let results = query_audit_logs(&env, action, huge_limit).expect("query should succeed");
            assert!(results.len() as u32 <= MAX_QUERY_LIMIT);
        });
    }

    #[test]
    fn test_get_audit_count_returns_zero_when_empty() {
        let (env, id) = make_env();
        env.as_contract(&id, || {
            assert_eq!(get_audit_count(&env), 0);
        });
    }

    #[test]
    fn test_log_multiple_events_increments_counter() {
        let (env, id) = make_env();
        let player = Address::generate(&env);
        let details = BytesN::from_array(&env, &[0u8; 128]);

        env.as_contract(&id, || {
            let _ = log_audit_event(&env, Some(&player), symbol_short!("a"), details.clone());
            let _ = log_audit_event(&env, Some(&player), symbol_short!("b"), details.clone());
            assert_eq!(get_audit_count(&env), 2);
        });
    }

    fn details(env: &Env, byte: u8) -> BytesN<128> {
        BytesN::from_array(env, &[byte; 128])
    }

    fn set_policy(env: &Env, max_age_secs: u64, max_entries: u64) {
        env.storage().instance().set(
            &AuditLoggerKey::Retention,
            &RetentionPolicy {
                max_age_secs,
                max_entries,
            },
        );
    }

    #[test]
    fn default_retention_is_applied_when_unconfigured() {
        let (env, id) = make_env();
        env.as_contract(&id, || {
            assert_eq!(
                get_audit_retention(&env),
                RetentionPolicy {
                    max_age_secs: DEFAULT_AUDIT_RETENTION_SECS,
                    max_entries: DEFAULT_MAX_AUDIT_ENTRIES,
                }
            );
        });
    }

    #[test]
    fn logging_prunes_entries_beyond_the_count_cap() {
        let (env, id) = make_env();
        let action = symbol_short!("scan");
        env.as_contract(&id, || {
            set_policy(&env, DEFAULT_AUDIT_RETENTION_SECS, 3);
            for i in 0..5u8 {
                log_audit_event(&env, None, action.clone(), details(&env, i)).unwrap();
            }

            assert_eq!(get_audit_count(&env), 5);
            assert_eq!(get_retained_audit_count(&env), 3);
            assert_eq!(oldest_audit_id(&env), 2);

            // The survivors are the three newest entries, unmodified.
            let kept = query_audit_logs(&env, action, 10).unwrap();
            assert_eq!(kept.len(), 3);
            for (entry, id) in kept.iter().zip(2u8..) {
                assert_eq!(entry.id, u64::from(id));
                assert_eq!(entry.details, details(&env, id));
            }
        });
    }

    #[test]
    fn prune_removes_entries_older_than_the_retention_window() {
        let (env, id) = make_env();
        let action = symbol_short!("scan");
        env.as_contract(&id, || {
            set_policy(&env, 100, 1_000);
            env.ledger().set_timestamp(1_000);
            log_audit_event(&env, None, action.clone(), details(&env, 0)).unwrap();
            log_audit_event(&env, None, action.clone(), details(&env, 1)).unwrap();
            env.ledger().set_timestamp(1_050);
            log_audit_event(&env, None, action.clone(), details(&env, 2)).unwrap();

            // Nothing is old enough yet.
            assert_eq!(prune_audit_logs(&env, MAX_PRUNE_BATCH), 0);

            env.ledger().set_timestamp(1_101);
            assert_eq!(prune_audit_logs(&env, MAX_PRUNE_BATCH), 2);
            assert_eq!(oldest_audit_id(&env), 2);

            let kept = query_audit_logs(&env, action, 10).unwrap();
            assert_eq!(kept.len(), 1);
            assert_eq!(kept.get(0).unwrap().id, 2);
        });
    }

    #[test]
    fn prune_is_bounded_and_resumable() {
        let (env, id) = make_env();
        let action = symbol_short!("bulk");
        env.as_contract(&id, || {
            env.ledger().set_timestamp(1_000);
            for i in 0..10u8 {
                log_audit_event(&env, None, action.clone(), details(&env, i)).unwrap();
            }
            set_policy(&env, 10, 1_000);
            env.ledger().set_timestamp(2_000);

            assert_eq!(prune_audit_logs(&env, 4), 4);
            assert_eq!(prune_audit_logs(&env, 4), 4);
            assert_eq!(prune_audit_logs(&env, 4), 2);
            assert_eq!(prune_audit_logs(&env, 4), 0);
            assert_eq!(get_retained_audit_count(&env), 0);
            assert_eq!(oldest_audit_id(&env), get_audit_count(&env));
        });
    }

    #[test]
    fn ids_keep_increasing_after_pruning() {
        let (env, id) = make_env();
        env.as_contract(&id, || {
            set_policy(&env, DEFAULT_AUDIT_RETENTION_SECS, 1);
            let first = log_audit_event(&env, None, symbol_short!("a"), details(&env, 0)).unwrap();
            let second = log_audit_event(&env, None, symbol_short!("a"), details(&env, 0)).unwrap();
            assert_eq!(first.id, 0);
            assert_eq!(second.id, 1);
            assert_eq!(get_retained_audit_count(&env), 1);
        });
    }

    #[test]
    fn set_audit_retention_requires_the_admin_role() {
        let (env, id) = make_env();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let outsider = Address::generate(&env);
        let policy = RetentionPolicy {
            max_age_secs: 60,
            max_entries: 10,
        };
        env.as_contract(&id, || {
            crate::access_control::init_roles(&env, admin.clone()).unwrap();
        });
        env.as_contract(&id, || {
            assert_eq!(
                set_audit_retention(&env, &outsider, &policy),
                Err(AuditLoggerError::Unauthorized)
            );
        });
        env.as_contract(&id, || {
            set_audit_retention(&env, &admin, &policy).unwrap();
            assert_eq!(get_audit_retention(&env), policy);
        });
    }

    #[test]
    fn set_audit_retention_rejects_zero_fields() {
        let (env, id) = make_env();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        env.as_contract(&id, || {
            crate::access_control::init_roles(&env, admin.clone()).unwrap();
        });
        for policy in [
            RetentionPolicy {
                max_age_secs: 0,
                max_entries: 10,
            },
            RetentionPolicy {
                max_age_secs: 10,
                max_entries: 0,
            },
        ] {
            env.as_contract(&id, || {
                assert_eq!(
                    set_audit_retention(&env, &admin, &policy),
                    Err(AuditLoggerError::InvalidRetention)
                );
            });
        }
    }
}
