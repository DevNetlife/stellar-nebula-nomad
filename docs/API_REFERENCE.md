# Public API Reference

This document outlines all public APIs for the Stellar Nebula Nomad contract suite.

## Module: nebula_gen

Handles nebula layout generation, anomaly tracking, and TTL management for space exploration.

### Key Types

- `NebulaError` - Error codes for nebula operations
- `NebulaLayout` - Generated nebula layout with anomalies and metadata
- `Anomaly` - Individual anomaly in a nebula (position, type, rarity)
- `ResourceClass` - Resource scarcity level: Sparse, Moderate, Abundant
- `AnomalyType` - Type of anomaly: DustCloud, IonStorm, CrystalFormation, PlasmaVent, DarkMatterPocket

### Public Functions

#### init(env: Env, admin: Address, anomaly_count: u32, min_size: u32, max_size: u32) -> Result<(), NebulaError>

Initialize the nebula generation contract with configuration parameters.

- `admin` - Admin address for configuration updates
- `anomaly_count` - Default anomalies per layout (will be clamped to [min_size, max_size])
- `min_size, max_size` - Bounds for nebula size configuration

Returns error if already initialized or parameters are invalid.

#### generate_validated_nebula_layout(env: Env, ship_id: u64, region_id: u64, seed: BytesN<32>) -> Result<NebulaLayout, NebulaError>

Generate a new nebula layout for a ship in a region.

- `ship_id` - Unique identifier for ship (must be > 0)
- `region_id` - Galactic region (must be 1..=MAX_REGION_ID)
- `seed` - 32-byte generation seed (must not be degenerate)

Returns complete NebulaLayout with positioned anomalies and hash, or validation/storage error.

#### query_anomaly(env: Env, ship_id: u64, anomaly_index: u32) -> Result<Anomaly, NebulaError>

Retrieve a single anomaly from an active layout by index.

- `ship_id` - Target ship
- `anomaly_index` - Zero-indexed position in anomaly array

Returns Anomaly if layout exists and index is in bounds.

#### has_anomaly(env: Env, ship_id: u64, anomaly_index: u32) -> Result<bool, NebulaError>

Check if an anomaly exists without fetching the full structure.

#### get_layout(env: Env, ship_id: u64) -> Option<NebulaLayout>

Fetch complete layout for a ship, if it exists and is not expired.

#### update_layout_ttl(env: Env, new_ttl: u64) -> Result<(), NebulaError>

Update time-to-live configuration for layouts (admin only).

#### clean_expired_layout(env: Env, ship_id: u64) -> Result<bool, NebulaError>

Remove single expired layout, freeing storage. Returns true if layout was deleted.

#### clean_expired_layouts(env: Env, ship_ids: Vec<u64>) -> Result<u32, NebulaError>

Batch clean multiple expired layouts. Returns count of deleted layouts.

---

## Module: resource_minter

Mints and manages in-game resources (credits, materials, tokens).

### Public Functions

- `mint_resource(env, amount, resource_type)` - Create new resource tokens
- `burn_resource(env, amount, resource_type)` - Destroy resource tokens
- `get_balance(env, account)` - Query account resource balance
- `transfer(env, from, to, amount)` - Transfer resources between accounts
- `approve_spending(env, spender, amount)` - Grant approval for resource spending

---

## Module: ship_registry

Manages ship registration, ownership, and metadata.

### Data Structures

- Ship registration records
- Ownership tracking
- Ship metadata and configurations

---

## Module: nomad_bonding

Handles bonding curves, staking, and yield calculations.

### Key Functions

- `create_bond(env, parameters)` - Initialize new bonding curve
- `buy_tokens(env, bond_id, amount)` - Purchase tokens on curve
- `sell_tokens(env, bond_id, amount)` - Sell tokens back to curve
- `claim_yield(env, account)` - Claim accumulated yield rewards

---

## Module: access_control

Role-based access control for contract operations.

### Public Functions

- `set_admin(env, new_admin)` - Transfer admin privileges
- `grant_role(env, account, role)` - Assign role to account
- `revoke_role(env, account, role)` - Remove role from account
- `has_role(env, account, role)` - Check if account has role
- `require_role(env, account, role)` - Enforce role requirement (reverts if missing)

---

## Error Handling

All contract functions return `Result<T, ContractError>` for proper error propagation. Errors include:

- **Validation Errors**: Input constraints violated (bounds, format, content)
- **Not Found**: Requested resource does not exist
- **Unauthorized**: Caller lacks required permissions
- **Conflict**: State transition impossible (already initialized, etc.)
- **Rate Limit**: Operation frequency limit exceeded

---

## Gas Optimization

Functions are optimized for gas efficiency with:

- Minimal storage reads
- Streamlined type conversions
- Efficient PRNG implementation
- Batch operations for bulk updates

---

## Usage Examples

### Generate a Nebula

```rust
use soroban_sdk::{Env, Address, BytesN};

// Prepare generation parameters
let ship_id = 42u64;
let region_id = 100u64;
let seed = BytesN::from_array(&env, &SEED_BYTES);

// Generate layout
let layout = contract.generate_validated_nebula_layout(ship_id, region_id, seed)?;

// Access anomalies
for (i, anomaly) in layout.anomalies.iter().enumerate() {
    println!("Anomaly at ({}, {}): {:?}", anomaly.x, anomaly.y, anomaly.anomaly_type);
}
```

### Mint Resources

```rust
// Mint 1000 credits to an account
let amount = 1000i128;
contract.mint_resource(&env, amount, "CREDIT")?;

// Transfer to another account
contract.transfer(&from, &to, 500)?;
```

---

## Constants

- `MAX_REGION_ID: u64 = 1_000_000` - Maximum region ID
- `MIN_SHIP_ID: u64 = 1` - Minimum ship ID
- `DEFAULT_ANOMALY_COUNT: u32 = 16` - Anomalies per layout
- `DEFAULT_LAYOUT_TTL: u64 = 86_400` - 24-hour default TTL

---

## Security Considerations

- All inputs validated at contract boundaries
- Access control enforced via RBAC
- Rate limiting prevents DoS attacks
- Degenerate seeds rejected to ensure randomness quality
- Zero addresses rejected for asset safety
