# Comprehensive Developer Onboarding Guide

Welcome to `stellar-nebula-nomad`! This guide provides end-to-end instructions for setting up your local environment, building smart contracts, running test suites, deploying to Stellar Soroban networks, and troubleshooting common issues.

---

## 1. Prerequisites & Installation

### Required Toolchain Components
Ensure you have the following installed on your host system:

1. **Rust Toolchain (1.75+ recommended)**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **WebAssembly Compilation Target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Soroban CLI**:
   ```bash
   cargo install --locked soroban-cli
   ```

4. **Docker & Docker Compose** (for running local Soroban standalone node):
   - Install [Docker Desktop](https://www.docker.com/products/docker-desktop/).

---

## 2. Project Structure

The repository is organized as follows:

```
stellar-nebula-nomad/
├── src/                    # Core contract modules
│   ├── nebula_gen.rs      # Nebula layout generation
│   ├── resource_minter.rs # Resource minting and management
│   ├── ship_registry.rs   # Ship registration
│   ├── nomad_bonding.rs   # Bonding curves and staking
│   ├── access_control.rs  # Role-based access control
│   ├── input_validation.rs # Input validation framework
│   └── lib.rs             # Module exports
├── docs/                   # Documentation
│   ├── API_REFERENCE.md   # Public API reference
│   ├── DEVELOPER_GUIDE.md # This file
│   └── ...
├── .github/
│   ├── workflows/         # CI/CD workflows
│   └── CONTRIBUTING.md    # Contribution guidelines
├── Cargo.toml             # Package manifest
└── tests/                 # Integration tests
```

Key directories:
- `src/` - All Soroban contract code organized by feature
- `tests/` - Integration tests that verify contract interactions
- `docs/` - User and developer documentation

---

## 3. Development Workflow

### Git Branch Strategy

1. **Create feature branch from main**:
   ```bash
   git checkout main
   git pull upstream main
   git checkout -b feat/short-description
   ```

2. **Commit with descriptive messages**:
   ```bash
   git commit -m "feat(module): description of changes

   - Detailed explanation of what changed
   - Why it was changed
   - Any breaking changes or side effects"
   ```

3. **Push and open Pull Request**:
   ```bash
   git push origin feat/short-description
   ```

### Running Tests During Development

```bash
# Run all tests
cargo test --locked

# Run tests with output
cargo test --locked -- --nocapture

# Run specific test
cargo test --locked test_contract_initialization

# Watch tests (requires cargo-watch)
cargo watch -x 'test --locked'
```

### Using CI

The repository has GitHub Actions CI that runs:
- Unit tests on every push
- Clippy linting checks
- Rustfmt formatting verification
- Documentation builds

Ensure CI passes before requesting review. Fix any lint errors:

```bash
cargo fmt
cargo clippy --fix --allow-dirty
```

---

## 4. Building, Testing, and Linting

### Compilation Commands
- **Check Compilation**:
  ```bash
  cargo check --locked
  ```

- **Build Release WebAssembly Binary**:
  ```bash
  cargo build --target wasm32-unknown-unknown --release
  ```

- **Optimize WASM Size**:
  ```bash
  soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_nebula_nomad.wasm
  ```

### Test Suite Execution
- **Run Unit Tests**:
  ```bash
  cargo test --locked
  ```

- **Run Specific Test Module**:
  ```bash
  cargo test --test contract_tests
  ```

### Code Formatting & Linting
- **Clippy Analysis**:
  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

- **Rustfmt Verification**:
  ```bash
  cargo fmt --check
  ```

- **Documentation Build Verification**:
  ```bash
  cargo doc --no-deps
  ```

---

## 5. Deployment to Testnet

### Deploying to Soroban Testnet

1. **Configure Soroban CLI Network**:
   ```bash
   soroban network add --rpc-url https://soroban-testnet.stellar.org:443 --network-passphrase "Test SDF Network ; September 2015" testnet
   ```

2. **Generate / Fund Identity**:
   ```bash
   soroban keys generate developer-identity
   soroban keys fund developer-identity --network testnet
   ```

3. **Deploy WASM Binary**:
   ```bash
   soroban contract deploy \
     --wasm target/wasm32-unknown-unknown/release/stellar_nebula_nomad.wasm \
     --source developer-identity \
     --network testnet
   ```

---

## 6. Testing Guide

### Writing Unit Tests

Create tests directly in your contract modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_generate_layout() {
        let env = Env::default();
        let admin = Address::generate(&env);
        
        // Initialize contract
        NebulaGen::init(&env, admin.clone(), 16, 8, 32, 86400).unwrap();
        
        // Generate layout
        let caller = Address::generate(&env);
        env.mock_all_signatures();
        
        let seed = BytesN::from_array(&env, &[1u8; 32]);
        let layout = NebulaGen::generate_validated_nebula_layout(
            &env, caller, 42, 100, seed
        ).unwrap();
        
        assert_eq!(layout.ship_id, 42);
        assert_eq!(layout.region_id, 100);
        assert!(layout.anomalies.len() > 0);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test --locked

# Run with output (helpful for debugging)
cargo test --locked -- --nocapture --test-threads=1

# Run tests in a specific file
cargo test --locked --test contract_tests

# Run a specific test by name
cargo test --locked test_generate_layout
```

### Debugging Test Failures

1. **Add println debugging**:
   ```bash
   cargo test --locked -- --nocapture
   ```

2. **Use the Soroban debugging tools**:
   ```bash
   soroban contract invoke --help
   ```

3. **Check error messages carefully** - they indicate validation failures, not internal bugs.

---

## 7. Contribution Guidelines

### Code Style

This project follows the Rust community standards:

- Run `cargo fmt` before committing
- Follow Clippy recommendations: `cargo clippy -- -D warnings`
- Use descriptive names for functions and variables
- Write comments only for non-obvious behavior
- Prefer explicit error handling over panics

### Commit Message Format

Use conventional commits:

```
type(scope): brief description under 50 chars

- Detailed explanation of the change
- Why this change was made
- Any breaking changes

Closes #123
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`

Example:
```
feat(nebula_gen): add rate limiting to layout generation

- Prevents DoS attacks by limiting generation frequency
- Configurable per-account rate limits
- Applies cost-based accounting for expensive operations

Closes #170
```

### Pull Request Process

1. Ensure your branch is up to date with main
2. Run full test suite: `cargo test --locked`
3. Run linter: `cargo clippy --all-targets -- -D warnings`
4. Run formatter: `cargo fmt`
5. Create PR with detailed description
6. Respond to reviewer feedback
7. Maintainer merges when approved

---

## 8. Common Pitfalls & Solutions

### Soroban-Specific Issues

| Problem | Cause | Solution |
|---------|-------|----------|
| "wasm32-unknown-unknown" target not found | Target not installed | `rustup target add wasm32-unknown-unknown` |
| Unexpected rate limiting errors | Rate limiter misconfigured | Check Rate Limiter configuration in tests |
| Layout expires immediately | TTL set to zero | Verify ttl_seconds > 0 |
| "All-zero seed" validation error | Seed bytes are all 0x00 | Use random seed with varied bytes |
| Storage rent exceeded during test | Test creates too much data | Clean up storage in test teardown |

### Memory & Performance

- **Avoid creating large vectors in loops** - use chunked operations
- **Minimize storage I/O** - read once, batch updates
- **Use symbol_short!() for keys** - reduces storage footprint
- **Cache computed values** when used multiple times in same function

### Testing Gotchas

- **Mock signatures before calling authenticated functions**: `env.mock_all_signatures()`
- **Ledger state must be set** before certain operations
- **Storage is isolated per test** - no cross-test pollution (good!)
- **Register contract before using client** - `env.register_contract(None, Contract)`

---

## 9. Getting Help

### Documentation Resources

- [Soroban Docs](https://developers.stellar.org/docs)
- [Stellar Developer Discord](https://discord.gg/stellar)
- [Soroban Examples](https://github.com/stellar/rs-soroban-sdk/tree/master/soroban-sdk/examples)

### Reporting Bugs

Before reporting:
1. Check if issue already exists in GitHub Issues
2. Reproduce with minimal test case
3. Document Rust version: `rustc --version`
4. Document Soroban CLI version: `soroban --version`

Report in GitHub Issues with:
- Minimal code to reproduce
- Expected vs actual behavior
- Environment info (OS, Rust version, Soroban CLI version)

### Code Examples for Common Developer Tasks

### Example 1: Defining Storage Keys & Contract Methods
```rust
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    Admin,
    Counter,
}

#[contract]
pub struct NomadContract;

#[contractimpl]
impl NomadContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Counter, &0u32);
    }

    pub fn increment(env: Env) -> u32 {
        let mut count: u32 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::Counter, &count);
        count
    }
}
```

### Example 2: Writing Soroban Unit Tests
```rust
#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_signatures();

    let contract_id = env.register_contract(None, NomadContract);
    let client = NomadContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    let val = client.increment();
    assert_eq!(val, 1);
}
```

---

## 10. Quick Reference Table

| Issue / Error | Cause | Resolution |
| :--- | :--- | :--- |
| `error[E0463]: can't find crate for std` when building WASM | Missing `wasm32-unknown-unknown` target | Run `rustup target add wasm32-unknown-unknown`. |
| `soroban: command not found` | Cargo binary directory not in system `PATH` | Ensure `~/.cargo/bin` is added to your shell PATH variable. |
| Dependency mismatch on `soroban-sdk` | Unlocked dependencies pulling breaking versions | Always build and test using `cargo check --locked` and `cargo test --locked`. |
| `Error: Storage limit exceeded` during test execution | Soroban ledger storage footprint exceeded | Optimize storage keys using `symbol_short!` and cleanup unused persistent data. |
| Clippy warnings on unused returns | Result returned from contract function ignored | Wrap calls with `let _ = ...` or explicitly return `Result<(), Error>`. |
