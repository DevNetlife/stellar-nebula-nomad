# Rollback Procedures & Incident Response

This document outlines how to safely rollback a contract upgrade if issues are discovered post-deployment.

---

## Quick Reference: When to Rollback

Rollback immediately if any of these occur:

- **Critical Bug**: Contract functions return incorrect results
- **State Corruption**: Data integrity checks fail
- **Gas Explosion**: Operations cost 10x+ normal gas
- **Consensus Failure**: Contract votes diverge across nodes
- **Security Vulnerability**: Previously unknown security issue discovered
- **Unexpected Behavior**: Transaction results contradict specification

**Do NOT wait for stabilization** - immediate rollback minimizes user impact.

---

## Pre-Rollback Checklist

Before initiating rollback:

- [ ] Confirm issue is reproducible (not isolated/transient)
- [ ] Verify admin signatures available and valid
- [ ] Backup current state snapshot for investigation
- [ ] Alert users/community
- [ ] Notify monitoring/support teams
- [ ] Have previous WASM hash and state checkpoint ready

---

## Step-by-Step Rollback Procedure

### Phase 1: Pause Operations (Immediate - 1-2 minutes)

**Objective**: Stop further damage before rollback.

```rust
// 1. Invoke emergency pause
use crate::emergency_controls;

// Call as admin
emergency_controls::pause_contract(&env, &admin)?;

// Verify pause state
let is_paused = emergency_controls::is_paused(&env)?;
assert!(is_paused, "Pause failed!");
```

**Confirmation**: All contract functions should now reject with `Paused` error.

**Check**: Query contract state should still work:
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --fn get_pause_status \
  --network mainnet
```

**Expected Output**: `{"paused": true, "paused_at": <timestamp>}`

---

### Phase 2: Restore State from Checkpoint (5-15 minutes)

**Objective**: Revert to last known-good state.

```rust
// 1. Load rollback checkpoint
use crate::migration_framework::{rollback_migration, MigrationKey};

let checkpoint_exists = env.storage()
    .persistent()
    .has(&MigrationKey::RollbackCheckpoint);

if !checkpoint_exists {
    log!(&env, "ERROR: No rollback checkpoint found!");
    return Err(MigrationError::CheckpointNotFound);
}

// 2. Invoke rollback
rollback_migration(&env, &admin, migration_id)?;

// 3. Verify state restored
let verification = migration_framework::verify_state_checksums(&env)?;
if !verification.all_valid {
    log!(&env, "ERROR: State verification failed after rollback!");
    return Err(MigrationError::StateValidationFailed);
}
```

**Verification Commands**:

```bash
# Check state keys count before
soroban contract invoke \
  --id CONTRACT_ID \
  --fn get_storage_size \
  --network mainnet

# Should match pre-upgrade baseline
```

---

### Phase 3: Revert Contract WASM (2-5 minutes)

**Objective**: Deploy previous stable version.

```rust
// 1. Have previous WASM hash ready (from pre-upgrade checklist)
// Example: SHA256 of previous v1.0 build
let previous_wasm_hash = BytesN::from_array(&env, &PREVIOUS_WASM_HASH_BYTES);

// 2. Update contract WASM to previous version
env.deployer().update_current_contract_wasm(previous_wasm_hash)?;

// 3. Log rollback event
env.events().publish(
    (symbol_short!("emergency"), symbol_short!("rollback")),
    (previous_wasm_hash, env.ledger().timestamp()),
);
```

**Verification**:

```bash
# Query new code hash
soroban contract info CONTRACT_ID --network mainnet

# Should show previous version hash
# Example: 0xabcd...ef01
```

---

### Phase 4: Functional Smoke Tests (5-10 minutes)

**Objective**: Verify core functionality works.

```rust
// Run basic operations to ensure contract functions normally
#[test]
fn smoke_test_after_rollback() {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    // 1. Can initialize
    NebulaGen::init(&env, admin.clone(), 16, 8, 32, 86400)?;
    
    // 2. Can generate layout
    let caller = Address::generate(&env);
    let seed = BytesN::from_array(&env, &[1u8; 32]);
    let layout = NebulaGen::generate_validated_nebula_layout(
        &env, caller.clone(), 42, 100, seed
    )?;
    assert_eq!(layout.ship_id, 42);
    
    // 3. Can query layout
    let fetched = NebulaGen::get_layout(&env, 42);
    assert!(fetched.is_some());
    
    // 4. Can cleanup
    NebulaGen::clean_expired_layout(&env, 42)?;
    
    println!("✓ All smoke tests passed");
}
```

**Execute Smoke Tests**:
```bash
cargo test --locked smoke_test_after_rollback -- --nocapture
```

---

### Phase 5: Resume Operations (2-3 minutes)

**Objective**: Restore normal contract operation.

```rust
// Only after all verification passes!

emergency_controls::unpause_contract(&env, &admin)?;

// Verify
let is_paused = emergency_controls::is_paused(&env)?;
assert!(!is_paused, "Unpause failed!");

log!(&env, "Contract operations resumed");
```

**User Communication**: Notify users that service is restored.

---

## Data Recovery from Backup

If rollback checkpoint is unavailable or corrupted:

### Option 1: Restore from Ledger Snapshot

```bash
# 1. Export state at checkpoint ledger
soroban ledger export \
  --ledger <CHECKPOINT_LEDGER_SEQUENCE> \
  --to rollback_state.json

# 2. Reconstruct state entries
python3 scripts/reconstruct_state.py rollback_state.json
```

### Option 2: Manual State Reconstruction

If automatic recovery fails:

```rust
// Reconstruct critical data from on-chain events
let events = get_historical_events(&env, before_upgrade_ledger);

// Re-apply non-breaking mutations
for event in events {
    match event {
        (symbol!("nebula"), symbol!("generated")) => {
            // Re-create layout from stored coordinates
            let (ship_id, hash, size) = decode_event(&event);
            // Reconstruct and store
        },
        _ => {}
    }
}
```

---

## Post-Rollback Verification

### Checklist

- [ ] **No Errors in Logs**
  ```bash
  curl https://soroban-rpc.testnet.stellar.org/logs | grep -i error
  ```

- [ ] **State Consistency**
  ```bash
  soroban contract invoke --id CONTRACT_ID --fn verify_integrity
  ```

- [ ] **Performance Baseline**
  - Generate 10 layouts, measure average gas
  - Compare to pre-upgrade baseline
  - Should be within 5% of original

- [ ] **User Transactions Success Rate**
  - Monitor success rate for 30 minutes
  - Should be > 99.9%

- [ ] **Community Communication**
  - Post incident summary
  - Explain root cause
  - Share timeline of rollback
  - Outline investigation plan

---

## Incident Investigation

After rollback stabilizes, investigate root cause:

### 1. Collect Evidence

```bash
# Export upgrade transactions
soroban history export \
  --start-ledger <UPGRADE_LEDGER> \
  --end-ledger <ROLLBACK_LEDGER> \
  --to upgrade_txs.json

# Extract contract invocations
jq '.[] | select(.type == "invoke_host_function")' upgrade_txs.json
```

### 2. Reproduce Issue

```rust
// In test environment, reproduce the exact sequence
#[test]
fn reproduce_issue() {
    let env = Env::default();
    
    // Recreate pre-upgrade state
    restore_state_from_backup(&env);
    
    // Perform operations that triggered the bug
    // ...
    
    // Verify bug reproduces
}
```

### 3. Create Fix

- Write test case that fails with broken code
- Implement fix
- Verify test now passes
- Run full test suite

### 4. Plan Re-upgrade

- Schedule new upgrade with 2-week lead time
- Include additional tests
- Extended testnet validation
- More conservative rollout (blue-green preferred)

---

## Communication Template

**For Community/Users**:

```
[INCIDENT REPORT]

Time: [UTC timestamp]
Duration: X minutes
Status: RESOLVED

What happened:
- [Brief description of issue]
- [Impact to users]

What we did:
- Paused contract at [time]
- Rolled back to previous version at [time]
- Resumed operations at [time]

What we learned:
- [Root cause analysis]
- [Preventive measures going forward]

Next steps:
- Investigation: [Timeline]
- Root cause fix: [Timeline]
- Re-deployment: [Timeline]

Incident ID: incident-YYYY-MM-DD-001
```

---

## Preventing Future Rollbacks

### Upgrade Checklist (Mandatory)

- [ ] Unit tests cover new code paths (>90% coverage)
- [ ] Integration tests on testnet (24+ hours)
- [ ] Dry-run migration with production-scale data
- [ ] Gas impact analysis (<5% increase acceptable)
- [ ] Security audit for critical code
- [ ] Load testing (10x normal TPS)
- [ ] Staged deployment (testnet → canary → mainnet)

### Monitoring Alerts

Set up alerts for:

```yaml
alerts:
  - error_rate > 1%: "Contract errors spiking"
  - gas_usage > baseline * 1.1: "Gas explosion detected"
  - transaction_latency > 2s: "Unexpected slowdown"
  - state_checkpoint_mismatch: "State corruption detected"
```

### Circuit Breaker Pattern

Implement automatic pause on critical errors:

```rust
// Auto-pause if error rate spikes
if error_rate_last_5_min > threshold {
    emergency_controls::pause_contract(&env, &admin)?;
    alert_admins("Auto-paused due to error spike");
}
```

---

## Communication Channels

During incident:

1. **Immediate** (< 1 min):
   - Alert: Ops team Slack channel
   - Status: Internal incident channel

2. **Status Update** (5-10 min):
   - Community: Twitter/Discord announcement
   - Docs: Incident page on website

3. **Detailed Report** (1-4 hours):
   - Post-mortem: Detailed analysis
   - Prevention: Measures taken
   - Timeline: Exact sequence of events

---

## Testing Rollback Procedures

Quarterly rollback drills:

```bash
# Schedule: First Friday of each quarter
# Duration: 1 hour
# Participants: Ops team, dev leads

# Procedure:
1. Announce drill (no surprise)
2. Trigger artificial issue
3. Execute full rollback procedure
4. Document time to resolve
5. Collect lessons learned
```

---

## Emergency Contact Information

Keep updated:

- **On-Call Ops**: [Phone/Slack]
- **Engineering Lead**: [Contact]
- **Security Team**: [Email]
- **Community Manager**: [Contact]

---

## Related Documents

- [Contract Upgrades](./CONTRACT_UPGRADES.md) - Full upgrade procedures
- [Migration Guide](./MIGRATION_GUIDE.md) - State migration details
- [Developer Guide](./DEVELOPER_GUIDE.md) - Development setup
