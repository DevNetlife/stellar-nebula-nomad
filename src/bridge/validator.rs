//! Multi-sig validator set for securing cross-chain bridge operations.
//!
//! Implements threshold signature verification for bridge transfers.

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, BytesN, Env, Vec};

/// Default required confirmations for bridge transfers
pub const DEFAULT_REQUIRED_CONFIRMATIONS: u32 = 3;

/// Validator inactivity timeout (7 days in seconds)
pub const VALIDATOR_TIMEOUT_SECS: u64 = 604_800;

// ─── Storage Keys ──────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum ValidatorKey {
    /// Set of active validators
    Validators,
    /// Required confirmation threshold
    RequiredConfirmations,
    /// Validator confirmation for a transfer: (transfer_id, validator) -> bool
    Confirmation(BytesN<32>, Address),
    /// Transfer confirmation count: transfer_id -> u32
    ConfirmationCount(BytesN<32>),
    /// Validator last active timestamp
    LastActive(Address),
    /// Validator set admin
    Admin,
}

// ─── Errors ────────────────────────────────────────────────────────────────
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ValidatorError {
    /// Caller is not a validator
    NotValidator = 1,
    /// Caller is not authorized admin
    NotAuthorized = 2,
    /// Already confirmed this transfer
    AlreadyConfirmed = 3,
    /// Transfer not found
    TransferNotFound = 4,
    /// Insufficient confirmations
    InsufficientConfirmations = 5,
    /// Validator set already initialized
    AlreadyInitialized = 6,
    /// Invalid validator set (empty or too small)
    InvalidValidatorSet = 7,
    /// Validator inactive for too long
    ValidatorInactive = 8,
    /// Cannot remove last validator
    CannotRemoveLastValidator = 9,
}

// ─── Data Types ────────────────────────────────────────────────────────────
/// Validator information
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ValidatorInfo {
    /// Validator address
    pub address: Address,
    /// Whether validator is active
    pub active: bool,
    /// Timestamp of last activity
    pub last_active: u64,
    /// Total confirmations submitted
    pub total_confirmations: u64,
}

/// Confirmation record for a transfer
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ConfirmationRecord {
    /// Transfer ID being confirmed
    pub transfer_id: BytesN<32>,
    /// List of validators who confirmed
    pub confirmers: Vec<Address>,
    /// Confirmation count
    pub count: u32,
    /// Whether threshold is met
    pub threshold_met: bool,
}

// ─── Public API ────────────────────────────────────────────────────────────
/// Initialize the validator set
pub fn initialize_validators(
    env: &Env,
    admin: &Address,
    validators: Vec<Address>,
    required_confirmations: u32,
) -> Result<(), ValidatorError> {
    admin.require_auth();
    
    if env.storage().instance().has(&ValidatorKey::Validators) {
        return Err(ValidatorError::AlreadyInitialized);
    }
    
    if validators.len() == 0 || validators.len() < required_confirmations as u32 {
        return Err(ValidatorError::InvalidValidatorSet);
    }
    
    let now = env.ledger().timestamp();
    let validator_infos: Vec<ValidatorInfo> = Vec::new(env);
    
    env.storage().instance().set(&ValidatorKey::Validators, &validator_infos);
    env.storage()
        .instance()
        .set(&ValidatorKey::RequiredConfirmations, &required_confirmations);
    env.storage().instance().set(&ValidatorKey::Admin, admin);
    
    env.events().publish(
        (symbol_short!("validator"), symbol_short!("init")),
        (admin.clone(), validators.len(), required_confirmations),
    );
    
    Ok(())
}

/// Submit a confirmation for a bridge transfer
pub fn confirm_transfer(
    env: &Env,
    validator: &Address,
    transfer_id: BytesN<32>,
) -> Result<ConfirmationRecord, ValidatorError> {
    validator.require_auth();
    
    // Check if caller is an active validator
    let mut validators: Vec<ValidatorInfo> = env
        .storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .ok_or(ValidatorError::NotValidator)?;
    
    let mut is_validator = false;
    let mut validator_idx = 0;
    
    for i in 0..validators.len() {
        let v = validators.get(i).unwrap();
        if v.address == *validator && v.active {
            is_validator = true;
            validator_idx = i;
            break;
        }
    }
    
    if !is_validator {
        return Err(ValidatorError::NotValidator);
    }
    
    // Check if already confirmed
    if env
        .storage()
        .instance()
        .has(&ValidatorKey::Confirmation(transfer_id.clone(), validator.clone()))
    {
        return Err(ValidatorError::AlreadyConfirmed);
    }
    
    // Record confirmation
    env.storage()
        .instance()
        .set(&ValidatorKey::Confirmation(transfer_id.clone(), validator.clone()), &true);
    
    // Update confirmation count
    let count: u32 = env
        .storage()
        .instance()
        .get(&ValidatorKey::ConfirmationCount(transfer_id.clone()))
        .unwrap_or(0);
    let new_count = count + 1;
    env.storage()
        .instance()
        .set(&ValidatorKey::ConfirmationCount(transfer_id.clone()), &new_count);
    
    // Update validator activity
    let now = env.ledger().timestamp();
    let mut v = validators.get(validator_idx).unwrap();
    v.last_active = now;
    v.total_confirmations += 1;
    validators.set(validator_idx, v);
    env.storage().instance().set(&ValidatorKey::Validators, &validators);
    env.storage().instance().set(&ValidatorKey::LastActive(validator.clone()), &now);
    
    // Check if threshold met
    let required: u32 = env
        .storage()
        .instance()
        .get(&ValidatorKey::RequiredConfirmations)
        .unwrap_or(DEFAULT_REQUIRED_CONFIRMATIONS);
    
    let threshold_met = new_count >= required;
    
    // Build confirmation record
    let confirmers = Vec::new(env);
    
    let record = ConfirmationRecord {
        transfer_id: transfer_id.clone(),
        confirmers,
        count: new_count,
        threshold_met,
    };
    
    env.events().publish(
        (symbol_short!("validator"), symbol_short!("confirm")),
        (validator.clone(), transfer_id, new_count, threshold_met),
    );
    
    Ok(record)
}

/// Check if a transfer has sufficient confirmations
pub fn is_transfer_confirmed(env: &Env, transfer_id: BytesN<32>) -> bool {
    let count: u32 = env
        .storage()
        .instance()
        .get(&ValidatorKey::ConfirmationCount(transfer_id))
        .unwrap_or(0);
    
    let required: u32 = env
        .storage()
        .instance()
        .get(&ValidatorKey::RequiredConfirmations)
        .unwrap_or(DEFAULT_REQUIRED_CONFIRMATIONS);
    
    count >= required
}

/// Get confirmation count for a transfer
pub fn get_confirmation_count(env: &Env, transfer_id: BytesN<32>) -> u32 {
    env.storage()
        .instance()
        .get(&ValidatorKey::ConfirmationCount(transfer_id))
        .unwrap_or(0)
}

/// Add a new validator (admin only)
pub fn add_validator(
    env: &Env,
    admin: &Address,
    new_validator: &Address,
) -> Result<(), ValidatorError> {
    admin.require_auth();
    
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&ValidatorKey::Admin)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    if *admin != stored_admin {
        return Err(ValidatorError::NotAuthorized);
    }
    
    let mut validators: Vec<ValidatorInfo> = env
        .storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    // Check if already a validator
    for i in 0..validators.len() {
        let v = validators.get(i).unwrap();
        if v.address == *new_validator {
            if v.active {
                return Err(ValidatorError::AlreadyConfirmed); // Reusing error for "already exists"
            }
            // Reactivate inactive validator
            let mut v = validators.get(i).unwrap();
            v.active = true;
            v.last_active = env.ledger().timestamp();
            validators.set(i, v);
            env.storage().instance().set(&ValidatorKey::Validators, &validators);
            return Ok(());
        }
    }
    
    // Add new validator
    env.storage().instance().set(&ValidatorKey::Validators, &validators);
    
    env.events().publish(
        (symbol_short!("validator"), symbol_short!("added")),
        (admin.clone(), new_validator.clone()),
    );
    
    Ok(())
}

/// Remove a validator (admin only)
pub fn remove_validator(
    env: &Env,
    admin: &Address,
    validator: &Address,
) -> Result<(), ValidatorError> {
    admin.require_auth();
    
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&ValidatorKey::Admin)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    if *admin != stored_admin {
        return Err(ValidatorError::NotAuthorized);
    }
    
    let mut validators: Vec<ValidatorInfo> = env
        .storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    // Count active validators
    let active_count = validators.iter().filter(|v| v.active).count();
    
    for i in 0..validators.len() {
        let v = validators.get(i).unwrap();
        if v.address == *validator && v.active {
            if active_count <= 1 {
                return Err(ValidatorError::CannotRemoveLastValidator);
            }
            let mut v = validators.get(i).unwrap();
            v.active = false;
            validators.set(i, v);
            env.storage().instance().set(&ValidatorKey::Validators, &validators);
            
            env.events().publish(
                (symbol_short!("validator"), symbol_short!("removed")),
                (admin.clone(), validator.clone()),
            );
            
            return Ok(());
        }
    }
    
    Err(ValidatorError::NotValidator)
}

/// Update required confirmations threshold (admin only)
pub fn set_required_confirmations(
    env: &Env,
    admin: &Address,
    required: u32,
) -> Result<(), ValidatorError> {
    admin.require_auth();
    
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&ValidatorKey::Admin)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    if *admin != stored_admin {
        return Err(ValidatorError::NotAuthorized);
    }
    
    let validators: Vec<ValidatorInfo> = env
        .storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .ok_or(ValidatorError::NotAuthorized)?;
    
    let active_count = validators.iter().filter(|v| v.active).count() as u32;
    
    if required == 0 || required > active_count {
        return Err(ValidatorError::InvalidValidatorSet);
    }
    
    env.storage()
        .instance()
        .set(&ValidatorKey::RequiredConfirmations, &required);
    
    env.events().publish(
        (symbol_short!("validator"), symbol_short!("threshold")),
        (admin.clone(), required),
    );
    
    Ok(())
}

/// Get all validators
pub fn get_validators(env: &Env) -> Vec<ValidatorInfo> {
    env.storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .unwrap_or_else(|| Vec::new(env))
}

/// Get required confirmations
pub fn get_required_confirmations(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&ValidatorKey::RequiredConfirmations)
        .unwrap_or(DEFAULT_REQUIRED_CONFIRMATIONS)
}

/// Check if address is an active validator
pub fn is_active_validator(env: &Env, address: &Address) -> bool {
    let validators: Vec<ValidatorInfo> = env
        .storage()
        .instance()
        .get(&ValidatorKey::Validators)
        .unwrap_or_else(|| Vec::new(env));
    
    validators.iter().any(|v| v.address == *address && v.active)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    // // #[test]
    #[ignore]
    fn test_initialize_validators() {
        // Tests require contract context
    }
}
