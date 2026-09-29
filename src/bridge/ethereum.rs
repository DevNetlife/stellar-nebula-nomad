//! Ethereum bridge module for cross-chain asset transfers.
//!
//! Implements lock/mint mechanism for bridging assets between Stellar and Ethereum.

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Bytes, BytesN, Env, Vec};

/// Bridge fee in basis points (0.3% = 30 bps)
pub const DEFAULT_BRIDGE_FEE_BPS: u32 = 30;

/// Minimum bridge amount to prevent dust attacks
pub const MIN_BRIDGE_AMOUNT: i128 = 1_000_000;

/// Lock period in seconds for cross-chain transfers (12 hours)
pub const LOCK_PERIOD_SECS: u64 = 43_200;

// ─── Storage Keys ──────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum BridgeKey {
    /// Bridge configuration (fee bps, paused state)
    Config,
    /// Locked assets: (asset_id, amount) mapping
    LockedAssets(BytesN<32>),
    /// Wrapped assets on Stellar: original_chain -> original_asset -> wrapped_asset
    WrappedAsset(Bytes, BytesN<32>),
    /// Pending transfers awaiting validator confirmation
    PendingTransfer(BytesN<32>),
    /// Transfer counter for ID generation
    TransferCount,
    /// Bridge admin address
    Admin,
    /// Total fees collected
    TotalFeesCollected,
}

// ─── Errors ────────────────────────────────────────────────────────────────
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BridgeError {
    /// Amount below minimum bridge threshold
    AmountTooLow = 1,
    /// Bridge is currently paused
    BridgePaused = 2,
    /// Caller is not authorized
    NotAuthorized = 3,
    /// Transfer already exists
    TransferExists = 4,
    /// Transfer not found
    TransferNotFound = 5,
    /// Invalid chain identifier
    InvalidChain = 6,
    /// Asset not supported for bridging
    AssetNotSupported = 7,
    /// Insufficient locked assets for unlock
    InsufficientLocked = 8,
    /// Transfer not yet confirmed by validators
    NotConfirmed = 9,
    /// Fee calculation overflow
    FeeOverflow = 10,
    /// Invalid fee percentage
    InvalidFee = 11,
}

// ─── Data Types ────────────────────────────────────────────────────────────
/// Supported chains for cross-chain bridging
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Chain {
    Stellar = 0,
    Ethereum = 1,
    Polygon = 2,
    Arbitrum = 3,
}

/// Bridge transfer states
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum TransferState {
    /// Initiated on source chain, awaiting validator confirmations
    Pending = 0,
    /// Confirmed by required validator threshold
    Confirmed = 1,
    /// Completed on destination chain
    Completed = 2,
    /// Cancelled due to timeout or dispute
    Cancelled = 3,
}

/// Cross-chain transfer record
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BridgeTransfer {
    /// Unique transfer ID
    pub id: BytesN<32>,
    /// Source chain
    pub source_chain: Chain,
    /// Destination chain
    pub dest_chain: Chain,
    /// Asset being transferred
    pub asset_id: BytesN<32>,
    /// Amount being transferred
    pub amount: i128,
    /// Sender on source chain
    pub sender: Address,
    /// Recipient on destination chain (as bytes for cross-chain compatibility)
    pub recipient: Bytes,
    /// Fee charged for the transfer
    pub fee: i128,
    /// Current state of the transfer
    pub state: TransferState,
    /// Timestamp when transfer was initiated
    pub initiated_at: u64,
    /// Number of validator confirmations received
    pub confirmations: u32,
}

/// Bridge configuration
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BridgeConfig {
    /// Fee in basis points
    pub fee_bps: u32,
    /// Whether bridge is paused
    pub paused: bool,
    /// Required validator confirmations
    pub required_confirmations: u32,
    /// Minimum transfer amount
    pub min_amount: i128,
}

// ─── Public API ────────────────────────────────────────────────────────────
/// Initialize the bridge with configuration
pub fn initialize_bridge(env: &Env, admin: &Address, fee_bps: u32) -> Result<(), BridgeError> {
    admin.require_auth();
    
    if fee_bps > 10_000 {
        return Err(BridgeError::InvalidFee);
    }
    
    let config = BridgeConfig {
        fee_bps,
        paused: false,
        required_confirmations: 3,
        min_amount: MIN_BRIDGE_AMOUNT,
    };
    
    env.storage().instance().set(&BridgeKey::Config, &config);
    env.storage().instance().set(&BridgeKey::Admin, admin);
    env.storage().instance().set(&BridgeKey::TransferCount, &0u64);
    env.storage().instance().set(&BridgeKey::TotalFeesCollected, &0i128);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("init")),
        (admin.clone(), fee_bps),
    );
    
    Ok(())
}

/// Lock assets on Stellar for bridging to another chain
pub fn lock_assets(
    env: &Env,
    sender: &Address,
    asset_id: BytesN<32>,
    amount: i128,
    dest_chain: Chain,
    recipient: Bytes,
) -> Result<BridgeTransfer, BridgeError> {
    sender.require_auth();
    
    let config: BridgeConfig = env
        .storage()
        .instance()
        .get(&BridgeKey::Config)
        .ok_or(BridgeError::NotAuthorized)?;
    
    if config.paused {
        return Err(BridgeError::BridgePaused);
    }
    
    if amount < config.min_amount {
        return Err(BridgeError::AmountTooLow);
    }
    
    if dest_chain == Chain::Stellar {
        return Err(BridgeError::InvalidChain);
    }
    
    // Calculate fee
    let fee = (amount * config.fee_bps as i128) / 10_000;
    let transfer_amount = amount - fee;
    
    // Generate transfer ID
    let count: u64 = env
        .storage()
        .instance()
        .get(&BridgeKey::TransferCount)
        .unwrap_or(0);
    env.storage().instance().set(&BridgeKey::TransferCount, &(count + 1));
    
    let transfer_id = generate_transfer_id(env, count, &asset_id, amount);
    
    // Update locked assets
    let locked: i128 = env
        .storage()
        .instance()
        .get(&BridgeKey::LockedAssets(asset_id.clone()))
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&BridgeKey::LockedAssets(asset_id.clone()), &(locked + transfer_amount));
    
    // Update total fees
    let total_fees: i128 = env
        .storage()
        .instance()
        .get(&BridgeKey::TotalFeesCollected)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&BridgeKey::TotalFeesCollected, &(total_fees + fee));
    
    let transfer = BridgeTransfer {
        id: transfer_id.clone(),
        source_chain: Chain::Stellar,
        dest_chain,
        asset_id,
        amount: transfer_amount,
        sender: sender.clone(),
        recipient,
        fee,
        state: TransferState::Pending,
        initiated_at: env.ledger().timestamp(),
        confirmations: 0,
    };
    
    env.storage()
        .instance()
        .set(&BridgeKey::PendingTransfer(transfer_id.clone()), &transfer);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("locked")),
        (sender.clone(), transfer_id, transfer_amount, fee),
    );
    
    Ok(transfer)
}

/// Mint wrapped assets on Stellar from another chain
pub fn mint_wrapped(
    env: &Env,
    source_chain: Chain,
    original_asset: BytesN<32>,
    amount: i128,
    recipient: &Address,
    transfer_id: BytesN<32>,
) -> Result<(), BridgeError> {
    recipient.require_auth();
    
    let config: BridgeConfig = env
        .storage()
        .instance()
        .get(&BridgeKey::Config)
        .ok_or(BridgeError::NotAuthorized)?;
    
    if config.paused {
        return Err(BridgeError::BridgePaused);
    }
    
    if source_chain == Chain::Stellar {
        return Err(BridgeError::InvalidChain);
    }
    
    // Check if transfer already processed
    if env
        .storage()
        .instance()
        .has(&BridgeKey::PendingTransfer(transfer_id.clone()))
    {
        return Err(BridgeError::TransferExists);
    }
    
    // Create a placeholder transfer record to prevent replay
    let transfer = BridgeTransfer {
        id: transfer_id.clone(),
        source_chain,
        dest_chain: Chain::Stellar,
        asset_id: original_asset.clone(),
        amount,
        sender: recipient.clone(),
        recipient: Bytes::new(env),
        fee: 0,
        state: TransferState::Completed,
        initiated_at: env.ledger().timestamp(),
        confirmations: config.required_confirmations,
    };
    
    env.storage()
        .instance()
        .set(&BridgeKey::PendingTransfer(transfer_id), &transfer);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("minted")),
        (recipient.clone(), original_asset, amount),
    );
    
    Ok(())
}

/// Unlock assets on Stellar when bridging back from another chain
pub fn unlock_assets(
    env: &Env,
    recipient: &Address,
    asset_id: BytesN<32>,
    amount: i128,
    transfer_id: BytesN<32>,
) -> Result<(), BridgeError> {
    recipient.require_auth();
    
    let config: BridgeConfig = env
        .storage()
        .instance()
        .get(&BridgeKey::Config)
        .ok_or(BridgeError::NotAuthorized)?;
    
    if config.paused {
        return Err(BridgeError::BridgePaused);
    }
    
    // Check if transfer already processed
    if env
        .storage()
        .instance()
        .has(&BridgeKey::PendingTransfer(transfer_id.clone()))
    {
        return Err(BridgeError::TransferExists);
    }
    
    // Check locked assets
    let locked: i128 = env
        .storage()
        .instance()
        .get(&BridgeKey::LockedAssets(asset_id.clone()))
        .unwrap_or(0);
    
    if locked < amount {
        return Err(BridgeError::InsufficientLocked);
    }
    
    // Update locked assets
    env.storage()
        .instance()
        .set(&BridgeKey::LockedAssets(asset_id.clone()), &(locked - amount));
    
    // Record transfer to prevent replay
    let transfer = BridgeTransfer {
        id: transfer_id.clone(),
        source_chain: Chain::Ethereum, // Assume Ethereum for unlock
        dest_chain: Chain::Stellar,
        asset_id: asset_id.clone(),
        amount,
        sender: recipient.clone(),
        recipient: Bytes::new(env),
        fee: 0,
        state: TransferState::Completed,
        initiated_at: env.ledger().timestamp(),
        confirmations: config.required_confirmations,
    };
    
    env.storage()
        .instance()
        .set(&BridgeKey::PendingTransfer(transfer_id), &transfer);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("unlocked")),
        (recipient.clone(), asset_id, amount),
    );
    
    Ok(())
}

/// Set bridge fee (admin only)
pub fn set_bridge_fee(env: &Env, admin: &Address, fee_bps: u32) -> Result<(), BridgeError> {
    admin.require_auth();
    
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&BridgeKey::Admin)
        .ok_or(BridgeError::NotAuthorized)?;
    
    if *admin != stored_admin {
        return Err(BridgeError::NotAuthorized);
    }
    
    if fee_bps > 10_000 {
        return Err(BridgeError::InvalidFee);
    }
    
    let mut config: BridgeConfig = env
        .storage()
        .instance()
        .get(&BridgeKey::Config)
        .ok_or(BridgeError::NotAuthorized)?;
    
    config.fee_bps = fee_bps;
    env.storage().instance().set(&BridgeKey::Config, &config);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("fee_set")),
        (admin.clone(), fee_bps),
    );
    
    Ok(())
}

/// Pause/unpause bridge (admin only)
pub fn set_bridge_paused(env: &Env, admin: &Address, paused: bool) -> Result<(), BridgeError> {
    admin.require_auth();
    
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&BridgeKey::Admin)
        .ok_or(BridgeError::NotAuthorized)?;
    
    if *admin != stored_admin {
        return Err(BridgeError::NotAuthorized);
    }
    
    let mut config: BridgeConfig = env
        .storage()
        .instance()
        .get(&BridgeKey::Config)
        .ok_or(BridgeError::NotAuthorized)?;
    
    config.paused = paused;
    env.storage().instance().set(&BridgeKey::Config, &config);
    
    env.events().publish(
        (symbol_short!("bridge"), symbol_short!("paused")),
        (admin.clone(), paused),
    );
    
    Ok(())
}

/// Get bridge configuration
pub fn get_bridge_config(env: &Env) -> Option<BridgeConfig> {
    env.storage().instance().get(&BridgeKey::Config)
}

/// Get transfer by ID
pub fn get_transfer(env: &Env, transfer_id: BytesN<32>) -> Option<BridgeTransfer> {
    env.storage().instance().get(&BridgeKey::PendingTransfer(transfer_id))
}

/// Get total fees collected
pub fn get_total_fees(env: &Env) -> i128 {
    env.storage().instance().get(&BridgeKey::TotalFeesCollected).unwrap_or(0)
}

// ─── Internal Helpers ──────────────────────────────────────────────────────────
fn generate_transfer_id(
    env: &Env,
    count: u64,
    asset_id: &BytesN<32>,
    amount: i128,
) -> BytesN<32> {
    // Simple ID generation using count and timestamp
    let timestamp = env.ledger().timestamp();
    let mut result = [0u8; 32];
    
    // Mix count and timestamp into the result
    result[0..8].copy_from_slice(&count.to_be_bytes());
    result[8..16].copy_from_slice(&timestamp.to_be_bytes());
    result[16..24].copy_from_slice(&(amount as u64).to_be_bytes());
    
    // Copy asset_id bytes
    let asset_bytes = asset_id.to_array();
    result[24..32].copy_from_slice(&asset_bytes[0..8]);
    
    BytesN::from_array(env, &result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    // // #[test]
    #[ignore]
    fn test_initialize_bridge() {
        // Tests require contract context
    }
}
