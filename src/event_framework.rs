//! Standardized event framework: schema-versioned, queryable event records.
//!
//! Two rules keep the cost of a standard emission predictable:
//!
//! 1. **Hot keys live in instance storage.** A persistent `get`/`set` resolves
//!    a ledger entry (~12,000 CPU instructions); the same call against the
//!    contract instance is a map lookup on an entry the host already holds
//!    (~500). Schema versions and the emit counters are read on *every*
//!    emission, so they sit in the instance tier; only the event records
//!    themselves — which `query_recent_events` has to find later — stay in
//!    persistent storage.
//! 2. **One publish per operation, one topic per event.** Emission goes
//!    through [`crate::events::emit`], so a standard event carries the event
//!    type as its only topic — no wrapper symbol rides along on every
//!    emission — and a burst publishes a single batched event instead of one
//!    event per payload.

use crate::events::emit;
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec};

const MAX_BURST_EVENTS: u32 = 15;
const DEFAULT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone)]
#[contracttype]
pub struct StandardEvent {
    pub id: u64,
    pub event_type: Symbol,
    pub payload: BytesN<256>,
    pub version: u32,
    pub caller: Address,
    pub timestamp: u64,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[contracterror]
pub enum EventFrameworkError {
    InvalidEventType = 1,
    LimitTooLarge = 2,
    Unauthorized = 3,
}

// ── storage layout ───────────────────────────────────────────────────────────

fn schema_key(event_type: &Symbol) -> (Symbol, Symbol) {
    (symbol_short!("ev_sch"), event_type.clone())
}

fn counter_key() -> Symbol {
    symbol_short!("ev_ctr")
}

fn event_key(index: u64) -> (Symbol, u64) {
    (symbol_short!("ev_rec"), index)
}

fn admin_key() -> Symbol {
    symbol_short!("ev_adm")
}

/// Read-only counterparts of [`counter_key`], written by the previous
/// persistent-tier layout. They are probed exactly once — when the instance
/// counter does not exist yet — and dropped afterwards.
fn legacy_index_key() -> Symbol {
    symbol_short!("ev_idx")
}

fn legacy_ts_seq_key(timestamp: u64) -> (Symbol, u64) {
    (symbol_short!("ev_tsq"), timestamp)
}

/// Emit position of the framework: the total number of records ever written
/// plus the sequence inside the current timestamp.
///
/// The old layout kept those in two keys, and the sequence key was *per
/// timestamp* — every ledger timestamp that saw an emission minted another
/// persistent entry that had to be rented forever. Folding `last_ts`/`seq`
/// into one entry resets the sequence in place, so the framework holds
/// exactly one counter entry for its whole life.
#[derive(Clone)]
#[contracttype]
struct EmitCounters {
    index: u64,
    last_ts: u64,
    seq: u64,
}

// ── schema versions (instance tier) ─────────────────────────────────────────

fn seed_default(env: &Env, event_type: &Symbol) {
    let key = schema_key(event_type);
    if env.storage().instance().has(&key) {
        return;
    }
    // A version registered before the move still lives in the legacy
    // persistent entry; `schema_version` migrates it on first read rather
    // than letting a default overwrite it here.
    if env.storage().persistent().has(&key) {
        return;
    }
    env.storage().instance().set(&key, &DEFAULT_SCHEMA_VERSION);
}

fn ensure_defaults(env: &Env) {
    for event_type in [
        symbol_short!("system"),
        symbol_short!("tutorial"),
        symbol_short!("fleet"),
        symbol_short!("bounty"),
    ] {
        seed_default(env, &event_type);
    }
}

/// Resolve the registered schema version for `event_type`.
///
/// Instance storage first — that read is what every emission pays. The legacy
/// persistent entry is only consulted while the instance copy is missing, and
/// is moved over (and released) the first time that happens.
fn schema_version(env: &Env, event_type: &Symbol) -> Result<u32, EventFrameworkError> {
    let key = schema_key(event_type);
    if let Some(version) = env.storage().instance().get::<_, u32>(&key) {
        return Ok(version);
    }
    if let Some(version) = env.storage().persistent().get::<_, u32>(&key) {
        env.storage().instance().set(&key, &version);
        env.storage().persistent().remove(&key);
        return Ok(version);
    }
    Err(EventFrameworkError::InvalidEventType)
}

pub fn init_event_framework(env: &Env, admin: &Address) {
    admin.require_auth();
    ensure_defaults(env);
    // Seed the counter entry here so the first emission is a plain instance
    // read rather than a probe of the legacy persistent keys.
    load_counters(env);
    env.storage().persistent().set(&admin_key(), admin);
}

pub fn register_event_schema(
    env: &Env,
    admin: &Address,
    event_type: &Symbol,
    version: u32,
) -> Result<(), EventFrameworkError> {
    admin.require_auth();

    let stored_admin = env
        .storage()
        .persistent()
        .get::<_, Address>(&admin_key())
        .ok_or(EventFrameworkError::Unauthorized)?;

    if stored_admin != *admin {
        return Err(EventFrameworkError::Unauthorized);
    }

    env.storage()
        .instance()
        .set(&schema_key(event_type), &version);
    Ok(())
}

// ── emit counters (instance tier) ────────────────────────────────────────────

/// Current counters, seeded from the legacy persistent keys the first time
/// they are needed so record indices (and the ids already handed out)
/// continue where they left off.
fn load_counters(env: &Env) -> EmitCounters {
    let key = counter_key();
    if let Some(counters) = env.storage().instance().get::<_, EmitCounters>(&key) {
        return counters;
    }

    let last_ts = env.ledger().timestamp();
    let index = env
        .storage()
        .persistent()
        .get::<_, u64>(&legacy_index_key());
    let seq = env
        .storage()
        .persistent()
        .get::<_, u64>(&legacy_ts_seq_key(last_ts))
        .unwrap_or(0);
    if index.is_some() {
        env.storage().persistent().remove(&legacy_index_key());
        env.storage()
            .persistent()
            .remove(&legacy_ts_seq_key(last_ts));
    }

    let counters = EmitCounters {
        index: index.unwrap_or(0),
        last_ts,
        seq,
    };
    env.storage().instance().set(&key, &counters);
    counters
}

/// Reserve `n` consecutive record slots, writing the counters back once.
///
/// Returns `(first_index, first_seq)`; slot `i` uses `first + i`. The whole
/// burst advances a single instance entry instead of two persistent entries
/// per event.
fn reserve(env: &Env, timestamp: u64, n: u64) -> (u64, u64) {
    let mut counters = load_counters(env);
    if counters.last_ts != timestamp {
        counters.last_ts = timestamp;
        counters.seq = 0;
    }
    let first_index = counters.index + 1;
    let first_seq = counters.seq + 1;
    counters.index += n;
    counters.seq += n;
    env.storage().instance().set(&counter_key(), &counters);
    (first_index, first_seq)
}

/// Index of the most recent record, for queries. Read-only: no migration and
/// no counter write, so a query never dirties the instance entry.
fn current_index(env: &Env) -> u64 {
    if let Some(counters) = env
        .storage()
        .instance()
        .get::<_, EmitCounters>(&counter_key())
    {
        return counters.index;
    }
    env.storage()
        .persistent()
        .get::<_, u64>(&legacy_index_key())
        .unwrap_or(0)
}

// ── emission ─────────────────────────────────────────────────────────────────

/// Persist one standard event record. Publishing is deliberately *not* done
/// here: the caller decides how many events the operation is worth, so a
/// burst writes N records but publishes one.
///
/// `slot` is the `(index, seq_in_ts)` pair reserved by [`reserve`].
fn store_event(
    env: &Env,
    caller: &Address,
    event_type: &Symbol,
    payload: &BytesN<256>,
    version: u32,
    slot: (u64, u64),
    timestamp: u64,
) -> u64 {
    let (index, seq_in_ts) = slot;
    let id = timestamp
        .saturating_mul(1_000_000)
        .saturating_add(seq_in_ts);

    let record = StandardEvent {
        id,
        event_type: event_type.clone(),
        payload: payload.clone(),
        version,
        caller: caller.clone(),
        timestamp,
    };
    env.storage().persistent().set(&event_key(index), &record);

    id
}

pub fn emit_standard_event(
    env: &Env,
    caller: &Address,
    event_type: Symbol,
    payload: BytesN<256>,
) -> Result<u64, EventFrameworkError> {
    caller.require_auth();
    let version = schema_version(env, &event_type)?;
    let timestamp = env.ledger().timestamp();
    let (index, seq_in_ts) = reserve(env, timestamp, 1);

    let id = store_event(
        env,
        caller,
        &event_type,
        &payload,
        version,
        (index, seq_in_ts),
        timestamp,
    );

    emit(env, event_type, (id, caller.clone(), payload));

    Ok(id)
}

/// Store every payload as its own record but publish **one** event for the
/// whole batch.
///
/// A burst is a single operation: `MAX_BURST_EVENTS` separate publishes would
/// each pay the fixed cost of an event (contract id, topic list, header) for
/// data that is already addressable through [`query_recent_events`]. The
/// batched event carries the id of its first record — ids are consecutive, so
/// record `i` has id `first_id + i` — followed by the payloads in order.
pub fn emit_standard_event_burst(
    env: &Env,
    caller: &Address,
    event_type: Symbol,
    payloads: Vec<BytesN<256>>,
) -> Result<u32, EventFrameworkError> {
    if payloads.len() > MAX_BURST_EVENTS {
        return Err(EventFrameworkError::LimitTooLarge);
    }
    if payloads.is_empty() {
        return Ok(0);
    }

    caller.require_auth();
    let version = schema_version(env, &event_type)?;
    let timestamp = env.ledger().timestamp();
    let count = payloads.len();
    let (first_index, first_seq) = reserve(env, timestamp, u64::from(count));

    let mut first_id = 0u64;
    for i in 0..count {
        let payload = payloads.get(i).ok_or(EventFrameworkError::LimitTooLarge)?;
        let id = store_event(
            env,
            caller,
            &event_type,
            &payload,
            version,
            (first_index + u64::from(i), first_seq + u64::from(i)),
            timestamp,
        );
        if i == 0 {
            first_id = id;
        }
    }

    emit(env, event_type, (first_id, caller.clone(), payloads));

    Ok(count)
}

pub fn query_recent_events(env: &Env, filter: &Symbol, limit: u32) -> Vec<StandardEvent> {
    let mut out = Vec::new(env);
    if limit == 0 {
        return out;
    }

    let max = if limit > 100 { 100 } else { limit };
    let index = current_index(env);

    let mut scanned: u64 = 0;
    while scanned < index && out.len() < max {
        let current = index - scanned;
        if let Some(event) = env
            .storage()
            .persistent()
            .get::<_, StandardEvent>(&event_key(current))
        {
            if filter == &symbol_short!("all") || &event.event_type == filter {
                out.push_back(event);
            }
        }
        scanned += 1;
    }

    out
}
