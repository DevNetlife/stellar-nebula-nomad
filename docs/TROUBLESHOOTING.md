# Troubleshooting Guide

Common issues and solutions when developing on stellar-nebula-nomad.

---

## Installation & Setup Issues

### "rustup: command not found"

**Cause**: Rust installer script not found or PATH not updated.

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment or restart your terminal
source $HOME/.cargo/env
rustup --version  # Should display version
```

### "cargo: command not found"

**Cause**: Cargo binary not in PATH after Rust installation.

**Solution**:
```bash
# Add Cargo to PATH
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
```

### "soroban: command not found"

**Cause**: Soroban CLI not installed or binary not in PATH.

**Solution**:
```bash
# Install Soroban CLI
cargo install --locked soroban-cli

# Verify installation
soroban --version
```

---

## Build & Compilation Issues

### "error[E0463]: can't find crate for std"

**Cause**: Building WASM target without the wasm32 target installed.

**Solution**:
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### "error: failed to verify the checksum for `soroban-sdk`"

**Cause**: Dependency versions not locked or index corrupted.

**Solution**:
```bash
# Use locked versions
cargo build --locked

# If still failing, update dependencies
cargo update soroban-sdk
```

### "error: could not compile `stellar-nebula-nomad`"

**Solution**:
1. Check Rust version: `rustup update stable`
2. Clean build artifacts: `cargo clean`
3. Rebuild: `cargo build --locked`
4. Check for syntax errors in your code

---

## Testing Issues

### "panicked at 'assertion failed'"

**Cause**: Test assertion condition failed.

**Solution**:
1. Run test with output: `cargo test --locked -- --nocapture`
2. Check the printed values
3. Verify test setup (mock signatures, ledger state)
4. Add debug prints: `println!("value: {:?}", var);`

### "Storage limit exceeded"

**Cause**: Test allocated too much persistent storage.

**Solution**:
```rust
#[test]
fn test_with_cleanup() {
    let env = Env::default();
    // ... test code ...
    
    // Clean up storage after use
    env.storage().persistent().remove(&key);
}
```

### "error: called `Option::unwrap()` on a `None` value"

**Cause**: Expected data was not found (empty Option).

**Solution**:
1. Verify data was stored: `env.storage().persistent().set(&key, &value)`
2. Use `.ok_or()` for proper error handling
3. Check storage keys match exactly

### "error: RateLimitExceeded"

**Cause**: Test exceeded configured rate limits.

**Solution**:
```rust
// In tests, mock rate limiter or adjust limits
let env = Env::default();
env.mock_all_signatures();  // Helps with auth mocking
```

---

## Contract Interaction Issues

### "error: invalid ship_id"

**Cause**: ship_id is zero (minimum is 1) or exceeds maximum.

**Solution**:
```rust
// Valid range: 1 to u64::MAX
let valid_ship_id = 42u64;  // Good

// Invalid
let invalid_ship_id = 0u64;  // Error: InvalidShipId
```

### "error: invalid region_id"

**Cause**: region_id outside valid range [1, 1_000_000].

**Solution**:
```rust
// Valid range: 1 to 1_000_000
let valid_region = 100u64;  // Good

// Invalid
let invalid_region = 1_000_001u64;  // Error: InvalidRegionId
let zero_region = 0u64;             // Error: InvalidRegionId
```

### "error: InvalidSeed"

**Cause**: Seed bytes are all-zero (degenerate seed).

**Solution**:
```rust
// Bad: all zeros
let bad_seed = BytesN::from_array(&env, &[0u8; 32]);

// Good: varied bytes
let good_seed = BytesN::from_array(&env, &[1, 2, 3, ..., 32]);
```

### "error: LayoutNotFound"

**Cause**: No active layout exists for the ship or it has expired.

**Solution**:
1. Generate layout first: `generate_validated_nebula_layout(...)`
2. Check TTL hasn't expired
3. Verify ship_id matches what was used in generation

### "error: AnomalyOutOfBounds"

**Cause**: Requested anomaly index >= layout.size.

**Solution**:
```rust
// Get layout first
let layout = NebulaGen::get_layout(&env, ship_id)?;

// Access within bounds
for i in 0..layout.anomalies.len() {
    let anomaly = NebulaGen::query_anomaly(&env, ship_id, i)?;
}
```

---

## Performance Issues

### "Test runs slowly"

**Cause**: Too many storage operations or heavy computation.

**Solution**:
1. Batch operations: group reads/writes
2. Reduce test data size
3. Use symbol_short!() for keys to reduce storage
4. Profile with `env.cost_estimate().budget()`

### "Contract deployment fails with gas error"

**Cause**: Contract binary too large or storage footprint too high.

**Solution**:
1. Optimize WASM: `soroban contract optimize --wasm contract.wasm`
2. Remove unused dependencies from Cargo.toml
3. Use inline functions for hot paths
4. Reduce string/binary constants

---

## Network & Deployment Issues

### "Connection refused" connecting to Soroban RPC

**Cause**: Testnet node unreachable or network misconfigured.

**Solution**:
```bash
# Check network configuration
soroban network list

# Add testnet if missing
soroban network add --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015" testnet

# Test connection
curl -X POST https://soroban-testnet.stellar.org:443 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth","params":[]}'
```

### "Account doesn't have enough balance"

**Cause**: Account not funded or balance too low.

**Solution**:
```bash
# Generate new account
soroban keys generate my-dev-account

# Fund from faucet
soroban keys fund my-dev-account --network testnet

# Check balance
soroban account balance my-dev-account --network testnet
```

### "error: Contract not found"

**Cause**: Contract deployed to wrong network or contract ID incorrect.

**Solution**:
1. Verify contract ID is correct: `soroban contract info <CONTRACT_ID> --network testnet`
2. Verify contract deployed to correct network
3. Re-deploy if needed

---

## IDE & Editor Setup

### VSCode autocomplete not working

**Solution**:
1. Install Rust-Analyzer extension
2. Create `.vscode/settings.json`:
   ```json
   {
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```
3. Reload window (Cmd+Shift+P > "Reload Window")

### IntelliJ IDEA Rust plugin issues

**Solution**:
1. Install Rust plugin from marketplace
2. Set Project SDK: File > Project Structure > SDK > Download JDK if missing
3. Mark `src/` as Sources Root

---

## Environment Variables

### Setting DEBUG mode

```bash
# Enable debug logging
export RUST_LOG=debug
cargo test --locked -- --nocapture

# Enable Soroban verbose logging  
export SOROBAN_LOG=info
soroban contract deploy ...
```

### Testnet RPC endpoint

```bash
# Use custom RPC endpoint
export SOROBAN_RPC_URL=https://soroban-testnet.stellar.org:443
soroban network add --rpc-url $SOROBAN_RPC_URL testnet
```

---

## Getting Help

1. **Check existing issues**: https://github.com/Space-Nebula/stellar-nebula-nomad/issues
2. **Ask in Stellar Discord**: https://discord.gg/stellar
3. **Review Soroban docs**: https://developers.stellar.org/docs
4. **Report new bugs** with minimal reproduction case

---

## Quick Diagnostic Commands

```bash
# Check all versions
rustc --version
cargo --version
soroban --version

# Verify setup
cargo check --locked
cargo test --locked -- --nocapture
cargo clippy -- -D warnings
cargo fmt --check

# Generate documentation
cargo doc --no-deps --open
```
