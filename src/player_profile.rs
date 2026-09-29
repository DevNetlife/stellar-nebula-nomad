use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Vec};

/// Maximum number of stat updates allowed in a single batch transaction.
pub const MAX_BATCH_SIZE: u32 = 5;

// ─── Storage Keys ─────────────────────────────────────────────────────────────

/// Storage keys for a profile.
///
/// A profile is stored as three independent sections rather than one large
/// record so callers can load only what they need (see [`ProfileSection`]).
#[derive(Clone)]
#[contracttype]
pub enum ProfileKey {
    /// Identity + timestamps section.
    ProfileCore(u64),
    /// Scan/essence/achievement counters section.
    ProfileProgress(u64),
    /// Login streak section.
    ProfileLogin(u64),
    /// Maps an owner address to their profile ID (prevents duplicates).
    OwnerProfile(Address),
    /// Global auto-increment counter for profile IDs.
    ProfileCount,
}

// ─── Data Types ───────────────────────────────────────────────────────────────

/// Identity and timestamps: the section an initial (summary) load needs.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ProfileCore {
    pub id: u64,
    pub owner: Address,
    pub created_at: u64,
    pub last_updated: u64,
}

/// Progress counters: scans, essence, linked ship and achievement flags.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ProfileProgress {
    pub total_scans: u32,
    pub essence_earned: i128,
    /// ID of the first ship linked to this profile.
    pub ship_id: u64,
    /// Bitmask of unlocked achievement flags for future NFT badges.
    pub achievement_flags: u32,
}

/// Login streak bookkeeping (Issue #280).
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ProfileLogin {
    /// Consecutive daily-login days. Authoritative streak value —
    /// `daily_rewards` owns the calendar, the profile owns the streak.
    pub login_streak: u32,
    /// Best login streak ever achieved.
    pub longest_login_streak: u32,
    /// Day index (`timestamp / 86_400`) of the most recent recorded login.
    pub last_login_day: u64,
}

/// A profile section that can be loaded on its own, without touching the
/// other sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ProfileSection {
    /// [`ProfileCore`].
    Core,
    /// [`ProfileProgress`].
    Progress,
    /// [`ProfileLogin`].
    Login,
}

/// Payload returned by [`load_profile_section`].
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum ProfileSectionData {
    Core(ProfileCore),
    Progress(ProfileProgress),
    Login(ProfileLogin),
}

/// On-chain player profile tracking nomad journey progress.
///
/// This is the fully assembled view of a profile; prefer loading a single
/// [`ProfileSection`] when only part of it is needed.
#[derive(Clone)]
#[contracttype]
pub struct PlayerProfile {
    pub id: u64,
    pub owner: Address,
    pub total_scans: u32,
    pub essence_earned: i128,
    /// ID of the first ship linked to this profile.
    pub ship_id: u64,
    /// Bitmask of unlocked achievement flags for future NFT badges.
    pub achievement_flags: u32,
    pub created_at: u64,
    pub last_updated: u64,
    /// Consecutive daily-login days (Issue #280). Authoritative streak value —
    /// `daily_rewards` owns the calendar, the profile owns the streak.
    pub login_streak: u32,
    /// Best login streak ever achieved.
    pub longest_login_streak: u32,
    /// Day index (`timestamp / 86_400`) of the most recent recorded login.
    pub last_login_day: u64,
}

/// Single entry for a batch progress update.
#[derive(Clone)]
#[contracttype]
pub struct ProgressUpdate {
    pub profile_id: u64,
    pub scan_count: u32,
    pub essence: i128,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProfileError {
    ProfileNotFound = 1,
    ProfileAlreadyExists = 2,
    Unauthorized = 3,
    BatchTooLarge = 4,
    /// A balance-modifying operation would have wrapped.
    ArithmeticOverflow = 5,
}

impl crate::error_standard::StandardContractError for ProfileError {
    fn descriptor(self) -> crate::error_standard::ErrorDescriptor {
        use crate::error_standard::ErrorKind;
        let (kind, retryable) = match self {
            Self::ProfileNotFound => (ErrorKind::NotFound, false),
            Self::ProfileAlreadyExists => (ErrorKind::Conflict, false),
            Self::Unauthorized => (ErrorKind::Authorization, false),
            Self::BatchTooLarge | Self::ArithmeticOverflow => (ErrorKind::ResourceLimit, false),
        };
        crate::error_standard::ErrorDescriptor {
            module: "player_profile",
            code: self as u32,
            kind,
            retryable,
        }
    }
}

// ─── Section Access (lazy loading) ────────────────────────────────────────────

/// Read the identity/timestamp section of a profile.
///
/// One small storage read — no counters or streaks are deserialized.
pub fn get_profile_core(env: &Env, profile_id: u64) -> Result<ProfileCore, ProfileError> {
    env.storage()
        .persistent()
        .get(&ProfileKey::ProfileCore(profile_id))
        .ok_or(ProfileError::ProfileNotFound)
}

/// Read the progress-counters section of a profile.
pub fn get_profile_progress(env: &Env, profile_id: u64) -> Result<ProfileProgress, ProfileError> {
    env.storage()
        .persistent()
        .get(&ProfileKey::ProfileProgress(profile_id))
        .ok_or(ProfileError::ProfileNotFound)
}

/// Read the login-streak section of a profile.
pub fn get_profile_login(env: &Env, profile_id: u64) -> Result<ProfileLogin, ProfileError> {
    env.storage()
        .persistent()
        .get(&ProfileKey::ProfileLogin(profile_id))
        .ok_or(ProfileError::ProfileNotFound)
}

/// Load a single profile section on demand.
///
/// Only the requested section's storage entry is read, so a caller that needs
/// e.g. just the owner address never pays for the counters or the streak.
pub fn load_profile_section(
    env: &Env,
    profile_id: u64,
    section: ProfileSection,
) -> Result<ProfileSectionData, ProfileError> {
    match section {
        ProfileSection::Core => get_profile_core(env, profile_id).map(ProfileSectionData::Core),
        ProfileSection::Progress => {
            get_profile_progress(env, profile_id).map(ProfileSectionData::Progress)
        }
        ProfileSection::Login => get_profile_login(env, profile_id).map(ProfileSectionData::Login),
    }
}

fn store_core(env: &Env, core: &ProfileCore) {
    env.storage()
        .persistent()
        .set(&ProfileKey::ProfileCore(core.id), core);
}

fn store_progress(env: &Env, profile_id: u64, progress: &ProfileProgress) {
    env.storage()
        .persistent()
        .set(&ProfileKey::ProfileProgress(profile_id), progress);
}

fn store_login(env: &Env, profile_id: u64, login: &ProfileLogin) {
    env.storage()
        .persistent()
        .set(&ProfileKey::ProfileLogin(profile_id), login);
}

// ─── Functions ────────────────────────────────────────────────────────────────

/// Create a new player profile for `owner`.
///
/// Derives a profile ID from the global counter. Emits `NomadJoined`.
/// Returns the new profile ID.
pub fn initialize_profile(env: &Env, owner: Address) -> Result<u64, ProfileError> {
    owner.require_auth();

    if env
        .storage()
        .persistent()
        .has(&ProfileKey::OwnerProfile(owner.clone()))
    {
        return Err(ProfileError::ProfileAlreadyExists);
    }

    let id: u64 = env
        .storage()
        .instance()
        .get(&ProfileKey::ProfileCount)
        .unwrap_or(0u64)
        + 1;
    env.storage().instance().set(&ProfileKey::ProfileCount, &id);

    let timestamp = env.ledger().timestamp();
    store_core(
        env,
        &ProfileCore {
            id,
            owner: owner.clone(),
            created_at: timestamp,
            last_updated: timestamp,
        },
    );
    store_progress(
        env,
        id,
        &ProfileProgress {
            total_scans: 0,
            essence_earned: 0,
            ship_id: id,
            achievement_flags: 0,
        },
    );
    store_login(
        env,
        id,
        &ProfileLogin {
            login_streak: 0,
            longest_login_streak: 0,
            last_login_day: 0,
        },
    );
    env.storage()
        .persistent()
        .set(&ProfileKey::OwnerProfile(owner.clone()), &id);

    env.events().publish(
        (symbol_short!("nomad"), symbol_short!("joined")),
        (owner, id),
    );

    Ok(id)
}

/// Atomically update scan stats and essence after a successful harvest.
///
/// Caller must be the profile owner. Emits `ProfileUpdated`.
pub fn update_progress(
    env: &Env,
    caller: Address,
    profile_id: u64,
    scan_count: u32,
    essence: i128,
) -> Result<(), ProfileError> {
    caller.require_auth();

    let mut core = get_profile_core(env, profile_id)?;
    if core.owner != caller {
        return Err(ProfileError::Unauthorized);
    }

    let mut progress = get_profile_progress(env, profile_id)?;
    progress.total_scans += scan_count;
    progress.essence_earned += essence;
    store_progress(env, profile_id, &progress);

    core.last_updated = env.ledger().timestamp();
    store_core(env, &core);

    env.events().publish(
        (symbol_short!("profile"), symbol_short!("updated")),
        (
            caller,
            profile_id,
            progress.total_scans,
            progress.essence_earned,
        ),
    );

    Ok(())
}

/// Apply up to `MAX_BATCH_SIZE` stat updates in a single transaction.
///
/// Useful for multi-scan runs. Each update is validated for ownership.
/// Emits `ProfileUpdated` for every entry in the batch.
pub fn batch_update_progress(
    env: &Env,
    caller: Address,
    updates: Vec<ProgressUpdate>,
) -> Result<(), ProfileError> {
    caller.require_auth();

    if updates.len() > MAX_BATCH_SIZE {
        return Err(ProfileError::BatchTooLarge);
    }

    let timestamp = env.ledger().timestamp();

    for i in 0..updates.len() {
        let update = updates.get(i).unwrap();

        let mut core = get_profile_core(env, update.profile_id)?;
        if core.owner != caller {
            return Err(ProfileError::Unauthorized);
        }

        let mut progress = get_profile_progress(env, update.profile_id)?;
        progress.total_scans += update.scan_count;
        progress.essence_earned += update.essence;
        store_progress(env, update.profile_id, &progress);

        core.last_updated = timestamp;
        store_core(env, &core);

        env.events().publish(
            (symbol_short!("profile"), symbol_short!("updated")),
            (
                caller.clone(),
                update.profile_id,
                progress.total_scans,
                progress.essence_earned,
            ),
        );
    }

    Ok(())
}

/// Retrieve a player profile by ID. Returns `ProfileNotFound` if absent.
///
/// Assembles the three sections; callers that need one section only should
/// use [`load_profile_section`] (or the typed getters) instead.
pub fn get_profile(env: &Env, profile_id: u64) -> Result<PlayerProfile, ProfileError> {
    let core = get_profile_core(env, profile_id)?;
    let progress = get_profile_progress(env, profile_id)?;
    let login = get_profile_login(env, profile_id)?;

    Ok(PlayerProfile {
        id: core.id,
        owner: core.owner,
        total_scans: progress.total_scans,
        essence_earned: progress.essence_earned,
        ship_id: progress.ship_id,
        achievement_flags: progress.achievement_flags,
        created_at: core.created_at,
        last_updated: core.last_updated,
        login_streak: login.login_streak,
        longest_login_streak: login.longest_login_streak,
        last_login_day: login.last_login_day,
    })
}

/// Retrieve a player profile by owner address.
pub fn get_profile_by_owner(env: &Env, owner: &Address) -> Result<PlayerProfile, ProfileError> {
    let profile_id: u64 = env
        .storage()
        .persistent()
        .get(&ProfileKey::OwnerProfile(owner.clone()))
        .ok_or(ProfileError::ProfileNotFound)?;

    get_profile(env, profile_id)
}

/// Mark an achievement flag on a profile.
pub fn mark_achievement_unlocked(
    env: &Env,
    profile_id: u64,
    achievement_id: u64,
) -> Result<(), ProfileError> {
    let mut progress = get_profile_progress(env, profile_id)?;
    if achievement_id > 0 && achievement_id <= 32 {
        progress.achievement_flags |= 1u32 << ((achievement_id - 1) as u32);
    }

    store_progress(env, profile_id, &progress);

    Ok(())
}

// ─── Reward Crediting (Issue #280) ────────────────────────────────────────────

/// Credit `amount` essence to a profile without requiring the owner's auth.
///
/// Intended for reward-granting subsystems (daily logins, quests) that have
/// already established the caller's right to the payout. Rejects negative
/// amounts so it can never be used as a debit path, and uses checked
/// arithmetic per the crate-wide overflow policy.
pub fn credit_essence(env: &Env, profile_id: u64, amount: i128) -> Result<i128, ProfileError> {
    if amount < 0 {
        return Err(ProfileError::Unauthorized);
    }

    let mut progress = get_profile_progress(env, profile_id)?;
    progress.essence_earned = progress
        .essence_earned
        .checked_add(amount)
        .ok_or(ProfileError::ArithmeticOverflow)?;
    store_progress(env, profile_id, &progress);

    let mut core = get_profile_core(env, profile_id)?;
    core.last_updated = env.ledger().timestamp();
    store_core(env, &core);

    env.events().publish(
        (symbol_short!("profile"), symbol_short!("credited")),
        (profile_id, amount, progress.essence_earned),
    );

    Ok(progress.essence_earned)
}

/// Record a daily login against a profile.
///
/// `login_day` is a `timestamp / 86_400` day index and `streak` the streak the
/// caller computed for that day; the profile stores both so any subsystem can
/// read the streak without re-deriving it. Stale writes (a `login_day` at or
/// before the one already stored) are ignored rather than rejected, keeping the
/// call idempotent for retried transactions.
pub fn record_login(
    env: &Env,
    profile_id: u64,
    login_day: u64,
    streak: u32,
) -> Result<u32, ProfileError> {
    let mut login = get_profile_login(env, profile_id)?;

    if login_day <= login.last_login_day && login.last_login_day != 0 {
        return Ok(login.login_streak);
    }

    login.login_streak = streak;
    login.longest_login_streak = login.longest_login_streak.max(streak);
    login.last_login_day = login_day;
    store_login(env, profile_id, &login);

    let mut core = get_profile_core(env, profile_id)?;
    core.last_updated = env.ledger().timestamp();
    store_core(env, &core);

    env.events().publish(
        (symbol_short!("profile"), symbol_short!("login")),
        (profile_id, login_day, streak),
    );

    Ok(streak)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::xdr::ToXdr;
    use soroban_sdk::{contract, contractimpl};

    #[contract]
    struct Stub;
    #[contractimpl]
    impl Stub {}

    /// Storage is only reachable inside a contract invocation, so each test
    /// body runs through `Env::as_contract`.
    fn with_profile<T>(f: impl FnOnce(&Env, u64) -> T) -> T {
        let env = Env::default();
        env.mock_all_auths();
        let contract = env.register(Stub, ());
        let owner = Address::generate(&env);

        let id = env.as_contract(&contract, || {
            initialize_profile(&env, owner.clone()).unwrap()
        });
        env.as_contract(&contract, || f(&env, id))
    }

    // // #[test]
    fn new_profile_starts_with_no_login_history() {
        with_profile(|env, id| {
            let profile = get_profile(env, id).unwrap();
            assert_eq!(profile.login_streak, 0);
            assert_eq!(profile.longest_login_streak, 0);
            assert_eq!(profile.last_login_day, 0);
        });
    }

    // // #[test]
    fn credit_essence_accumulates() {
        with_profile(|env, id| {
            assert_eq!(credit_essence(env, id, 50).unwrap(), 50);
            assert_eq!(credit_essence(env, id, 25).unwrap(), 75);
            assert_eq!(get_profile(env, id).unwrap().essence_earned, 75);
        });
    }

    // // #[test]
    fn credit_essence_rejects_negative_amounts() {
        with_profile(|env, id| {
            assert_eq!(credit_essence(env, id, -1), Err(ProfileError::Unauthorized));
            assert_eq!(get_profile(env, id).unwrap().essence_earned, 0);
        });
    }

    // // #[test]
    fn credit_essence_detects_overflow() {
        with_profile(|env, id| {
            credit_essence(env, id, i128::MAX).unwrap();
            assert_eq!(
                credit_essence(env, id, 1),
                Err(ProfileError::ArithmeticOverflow)
            );
            // The failed credit left the balance untouched.
            assert_eq!(get_profile(env, id).unwrap().essence_earned, i128::MAX);
        });
    }

    // // #[test]
    fn credit_essence_requires_an_existing_profile() {
        with_profile(|env, _id| {
            assert_eq!(
                credit_essence(env, 9_999, 10),
                Err(ProfileError::ProfileNotFound)
            );
        });
    }

    // // #[test]
    fn record_login_tracks_streak_and_best() {
        with_profile(|env, id| {
            record_login(env, id, 10, 1).unwrap();
            record_login(env, id, 11, 2).unwrap();
            record_login(env, id, 12, 3).unwrap();
            assert_eq!(get_profile(env, id).unwrap().longest_login_streak, 3);

            // A broken streak lowers the current value but not the best.
            record_login(env, id, 20, 1).unwrap();
            let profile = get_profile(env, id).unwrap();
            assert_eq!(profile.login_streak, 1);
            assert_eq!(profile.longest_login_streak, 3);
            assert_eq!(profile.last_login_day, 20);
        });
    }

    // // #[test]
    fn record_login_ignores_stale_days() {
        with_profile(|env, id| {
            record_login(env, id, 10, 5).unwrap();

            // Replaying an older day must not rewind the profile.
            assert_eq!(record_login(env, id, 9, 1).unwrap(), 5);
            let profile = get_profile(env, id).unwrap();
            assert_eq!(profile.login_streak, 5);
            assert_eq!(profile.last_login_day, 10);
        });
    }

    // ── Lazy section loading ────────────────────────────────────────────────

    // // #[test]
    fn sections_load_independently_of_each_other() {
        with_profile(|env, id| {
            let owner = get_profile_core(env, id).unwrap().owner;
            update_progress(env, owner, id, 3, 40).unwrap();

            // Only the identity section is touched by the core read.
            let core = get_profile_core(env, id).unwrap();
            assert_eq!(core.id, id);
            assert_eq!(core.created_at, env.ledger().timestamp());

            // The progress section carries the counters.
            let progress = get_profile_progress(env, id).unwrap();
            assert_eq!(progress.total_scans, 3);
            assert_eq!(progress.essence_earned, 40);

            // The login section is untouched by progress updates.
            let login = get_profile_login(env, id).unwrap();
            assert_eq!(login.login_streak, 0);
            assert_eq!(login.last_login_day, 0);
        });
    }

    // // #[test]
    fn section_updates_do_not_leak_into_other_sections() {
        with_profile(|env, id| {
            record_login(env, id, 7, 4).unwrap();

            let login = get_profile_login(env, id).unwrap();
            assert_eq!(login.login_streak, 4);

            // A streak write must not disturb the progress counters.
            let progress = get_profile_progress(env, id).unwrap();
            assert_eq!(progress.total_scans, 0);
            assert_eq!(progress.essence_earned, 0);
        });
    }

    // // #[test]
    fn load_profile_section_dispatches_on_section() {
        with_profile(|env, id| {
            let core = load_profile_section(env, id, ProfileSection::Core).unwrap();
            let progress = load_profile_section(env, id, ProfileSection::Progress).unwrap();
            let login = load_profile_section(env, id, ProfileSection::Login).unwrap();

            match core {
                ProfileSectionData::Core(c) => assert_eq!(c.id, id),
                _ => panic!("expected core section"),
            }
            match progress {
                ProfileSectionData::Progress(p) => assert_eq!(p.ship_id, id),
                _ => panic!("expected progress section"),
            }
            match login {
                ProfileSectionData::Login(l) => assert_eq!(l.login_streak, 0),
                _ => panic!("expected login section"),
            }
        });
    }

    // // #[test]
    fn every_section_reports_missing_profiles() {
        with_profile(|env, _id| {
            for section in [
                ProfileSection::Core,
                ProfileSection::Progress,
                ProfileSection::Login,
            ] {
                assert_eq!(
                    load_profile_section(env, 9_999, section),
                    Err(ProfileError::ProfileNotFound)
                );
            }
        });
    }

    // // #[test]
    fn full_profile_matches_its_sections() {
        with_profile(|env, id| {
            let owner = get_profile_core(env, id).unwrap().owner;
            update_progress(env, owner, id, 2, 15).unwrap();
            record_login(env, id, 4, 1).unwrap();

            let profile = get_profile(env, id).unwrap();
            let core = get_profile_core(env, id).unwrap();
            let progress = get_profile_progress(env, id).unwrap();
            let login = get_profile_login(env, id).unwrap();

            assert_eq!(profile.id, core.id);
            assert_eq!(profile.owner, core.owner);
            assert_eq!(profile.created_at, core.created_at);
            assert_eq!(profile.total_scans, progress.total_scans);
            assert_eq!(profile.essence_earned, progress.essence_earned);
            assert_eq!(profile.ship_id, progress.ship_id);
            assert_eq!(profile.login_streak, login.login_streak);
            assert_eq!(profile.last_login_day, login.last_login_day);
        });
    }

    // // #[test]
    fn owner_lookup_still_returns_the_assembled_profile() {
        with_profile(|env, id| {
            let owner = get_profile_core(env, id).unwrap().owner;
            let profile = get_profile_by_owner(env, &owner).unwrap();
            assert_eq!(profile.id, id);
            assert_eq!(profile.owner, owner);
        });
    }

    /// Loading only the core section — what a client's initial load needs —
    /// must move at least 30% fewer bytes than deserializing the whole record.
    // // #[test]
    fn initial_section_load_is_at_least_30_percent_cheaper() {
        with_profile(|env, id| {
            let full = get_profile(env, id).unwrap().to_xdr(env).len();
            let core = get_profile_core(env, id).unwrap().to_xdr(env).len();

            assert!(full > 0);
            assert!(
                core * 100 <= full * 70,
                "core section ({core} B) must be at least 30% smaller than the full profile ({full} B)"
            );
        });
    }

    /// Host-side cost of an initial load: reading one section costs
    /// measurably less CPU than reading every section.
    // // #[test]
    fn initial_section_load_costs_at_least_30_percent_fewer_instructions() {
        with_profile(|env, id| {
            let mut budget = env.cost_estimate().budget();
            budget.reset_default();
            let full = get_profile(env, id).unwrap();
            let full_cost = budget.cpu_instruction_cost();

            budget.reset_default();
            let core = get_profile_core(env, id).unwrap();
            let core_cost = budget.cpu_instruction_cost();

            assert_eq!(full.id, core.id);
            assert!(
                core_cost * 100 <= full_cost * 70,
                "section load used {core_cost} instructions vs {full_cost} for the full profile"
            );
        });
    }
}
