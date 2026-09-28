# Contract Upgrade Procedures & Security Guidelines

This document details the architecture, pre-upgrade verification, step-by-step upgrading process, and rollback procedures for Soroban smart contracts within the `stellar-nebula-nomad` ecosystem.

---

## 1. Overview & Architecture

Contracts in `stellar-nebula-nomad` use Soroban's native contract upgrading mechanisms combined with state versioning defined in `src/migration_framework.rs`.

### Key Security Design Principles
- **WASM Hash Verification**: All contract upgrades update the contract code hash via Soroban's `env.deployer().update_current_contract_wasm(new_wasm_hash)`.
- **Role-Based Access Control (RBAC)**: Only authorized admin addresses (`admin.require_auth()`) can trigger upgrade and migration functions.
- **Atomic State Versioning**: Every contract maintains a `CurrentSchemaVersion` state key to ensure binary logic and storage schemas stay in sync.

---

## 2. Pre-Upgrade Checklist

Before initiating any production upgrade, complete the following mandatory verification steps:

- [ ] **Contract Compilation & Verification**
  - [ ] Build release WASM binary: `cargo build --target wasm32-unknown-unknown --release`
  - [ ] Optimize WASM size using `soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_nebula_nomad.wasm`
  - [ ] Calculate new WASM SHA-256 hash: `sha256sum target/wasm32-unknown-unknown/release/stellar_nebula_nomad.optimized.wasm`
- [ ] **Dry-Run & Validation**
  - [ ] Execute `dry_run_migration()` with sample state payloads to verify gas usage and data schema compatibility.
  - [ ] Verify `is_backward_compatible()` returns `true` for target version transitions.
- [ ] **State Snapshot & Rollback Checkpoint**
  - [ ] Export current state snapshot using `state_snapshot::create_snapshot()`.
  - [ ] Verify rollback checkpoint storage allocation (`MigrationKey::RollbackCheckpoint`).
- [ ] **Governance & Multi-Sig Signoff**
  - [ ] Prepare transaction authorization payloads for admin signatures.
  - [ ] Confirm emergency pause controls are operational via `emergency_controls.rs`.

---

## 3. Step-by-Step Upgrade Procedure

### Step 1: Install New WASM Code
Upload the optimized WASM byte code to the Soroban network:
```bash
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/stellar_nebula_nomad.optimized.wasm \
  --source admin-identity \
  --network mainnet
```
Note down the returned `<NEW_WASM_HASH>`.

### Step 2: Plan the Migration
Invoke `plan_migration` via the migration framework to initialize the upgrade metadata record:
```rust
let record = migration_framework::plan_migration(
    &env,
    &admin,
    current_version,
    target_version,
    symbol_short!("upg_v2"),
)?;
```

### Step 3: Run Dry-Run Verification
Execute `dry_run_migration` with sample records to validate schema constraints without committing storage changes:
```rust
let report = migration_framework::dry_run_migration(
    &env,
    &admin,
    record.id,
    sample_records,
)?;
assert!(report.would_succeed, "Dry-run validation failed!");
```

### Step 4: Execute Code Upgrade & Batch State Migration
Update the contract WASM code and run batch migration steps:
```rust
// 1. Update WASM code on chain
env.deployer().update_current_contract_wasm(new_wasm_hash);

// 2. Process state migration batches (max 100 records per batch)
let batch_state = migration_framework::execute_migration_batch(
    &env,
    &admin,
    record.id,
    batch_index,
    total_batches,
    batch_data,
)?;
```

### Step 5: Finalize Migration & Update Version
Record completion in migration history and advance schema version:
```rust
migration_framework::record_migration_completion(
    &env,
    record.id,
    from_version,
    to_version,
    total_records_processed,
);
```

---

## 4. Rollback Procedures

If an anomaly, state mismatch, or unexpected behavior occurs post-upgrade, execute the automated rollback path immediately:

1. **Trigger Emergency Pause**:
   ```rust
   emergency_controls::pause_contract(&env, &admin);
   ```

2. **Invoke Rollback Migration**:
   Call `rollback_migration` to revert state modifications from the saved checkpoint (`MigrationKey::RollbackCheckpoint`):
   ```rust
   migration_framework::rollback_migration(&env, &admin, migration_id)?;
   ```

3. **Revert Contract WASM**:
   Update contract WASM back to the previous verified WASM hash:
   ```rust
   env.deployer().update_current_contract_wasm(previous_wasm_hash);
   ```

4. **Verify State Consistency & Unpause**:
   Audit storage keys and unpause contract operations.

---

## 5. Security & Access Control

- All upgrade entry points require explicit authorization (`admin.require_auth()`).
- Unauthenticated upgrade attempts emit audit events and fail with `MigrationError::Unauthorized`.
- Incompatible migrations must be explicitly flagged using `mark_incompatible()`.

---

## 6. Upgrade Types & When to Use Each

### 1. Parameter Changes Only
**Use when**: Updating configuration values without changing contract logic.

**Risk Level**: Low

**Steps**:
```rust
// 1. Update config in storage
let mut config = get_config(&env)?;
config.param = new_value;
store_config(&env, &config);

// 2. No migration needed
// 3. Events logged automatically
```

**Testing**: Run full test suite - data structures don't change.

### 2. Code Changes (Logic Only)
**Use when**: Fixing bugs or optimizing code without changing storage schema.

**Risk Level**: Medium

**Steps**:
1. Build new WASM
2. Install on testnet
3. Verify with test transactions
4. Execute contract upgrade via admin

**Testing**: New code must handle existing state correctly.

### 3. Storage Layout Changes
**Use when**: Adding new fields, removing fields, or changing data structures.

**Risk Level**: High

**Steps**:
1. Increment schema version
2. Plan migration
3. Run dry-run validation
4. Execute batch migrations
5. Update WASM code
6. Finalize migration

**Testing**: Test with production-scale datasets.

---

## 7. Migration Strategies & Decision Tree

### Strategy 1: In-Place Updates (Simple)
Best for: Adding optional fields with defaults.

```rust
// Before: struct Ship { id: u64, name: String }
// After: struct Ship { id: u64, name: String, level: u32 }

// Migration: Set all level = 1 (default)
for ship_id in all_ships {
    let ship = get_ship(&env, ship_id);
    ship.level = 1; // Default value
    save_ship(&env, ship_id, ship);
}
```

### Strategy 2: Blue-Green Deployment (Safer)
Best for: Complex changes, zero-downtime required.

```rust
// 1. Deploy new version alongside old
// 2. Gradually migrate users to new version
// 3. Monitor for issues
// 4. Retire old version after stability window

// Step implementation:
// Week 1: New version deployed, 0% traffic
// Week 2: 10% traffic to new version
// Week 3: 50% traffic to new version  
// Week 4: 100% traffic, retire old version
```

### Strategy 3: Gradual Rollout (Recommended)
Best for: Large datasets, high availability required.

```rust
// 1. Split state into chunks
// 2. Migrate one chunk per ledger close
// 3. Monitor gas usage and errors
// 4. Pause/retry if issues detected
// 5. Complete when all chunks migrated

const CHUNK_SIZE = 100; // Records per batch
for i in 0..total_chunks {
    execute_migration_batch(i, CHUNK_SIZE)?;
    if should_pause(&env)? {
        break;
    }
}
```

---

## 8. Post-Upgrade Verification

After successful upgrade, verify:

- [ ] **Functionality Tests**
  ```bash
  cargo test --locked integration_tests
  ```

- [ ] **State Consistency**
  ```rust
  migration_framework::verify_state_checksums(&env)?;
  ```

- [ ] **Performance Baseline**
  - Measure gas usage of common operations
  - Compare to pre-upgrade baseline
  - Alert if usage increased > 10%

- [ ] **Smoke Tests**
  - Execute 100 random transactions
  - Verify all succeed
  - Check event logs for errors

- [ ] **Monitoring Dashboard**
  - Watch error rates
  - Monitor latency
  - Check resource usage

---

## 9. Emergency Procedures

### Contract Stuck in Migration
If migration hangs or gets stuck:

```bash
# 1. Check migration status
soroban contract invoke \
  --id CONTRACT_ID \
  --fn get_migration_status \
  --network mainnet

# 2. If recoverable, resume next batch
soroban contract invoke \
  --id CONTRACT_ID \
  --fn resume_migration \
  --arg <migration_id> \
  --network mainnet

# 3. If not recoverable, rollback immediately
# (See Rollback Procedures section)
```

### Unexpected State Corruption
If data corruption detected during or after upgrade:

1. **Immediate Action**: Pause contract
   ```rust
   emergency_controls::pause_contract(&env, &admin)?;
   ```

2. **Restore from Checkpoint**:
   ```rust
   migration_framework::rollback_migration(&env, &admin, migration_id)?;
   ```

3. **Root Cause Analysis**: Review migration logs and checksums

4. **Re-plan Migration**: With fixes, rerun from checkpoint

---

## 10. Governance Integration

### Upgrade Proposal Workflow

1. **Community Discussion** (GitHub Issues)
   - Propose change with rationale
   - Collect feedback for 1 week

2. **Technical Review**
   - Security audit (if code change)
   - State migration dry-run
   - Gas impact analysis

3. **Multi-Sig Approval**
   - Prepare upgrade transaction
   - Require signatures from N of M signers
   - Execute after N signatures collected

4. **Staged Deployment**
   - Deploy to testnet first
   - Run for 24+ hours
   - Collect metrics
   - Deploy to mainnet

5. **Monitoring & Rollback Window**
   - 7-day rollback window post-upgrade
   - Monitor error rates
   - Be ready to rollback if needed

### Configuration Changes (Lower Risk)
Simpler process for parameter-only changes:

1. **Proposal**: Present change to community
2. **Snapshot vote**: Community votes on parameter
3. **Execute**: If approved, update via multi-sig
4. **Monitor**: Watch effects for 48 hours
