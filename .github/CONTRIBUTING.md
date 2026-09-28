# Contributing to stellar-nebula-nomad

Thank you for considering contributing to the Stellar Nebula Nomad project! This document outlines the process for contributing code, reporting bugs, and suggesting improvements.

---

## Code of Conduct

Be respectful and professional in all interactions. We're building a welcoming community for all contributors.

---

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork: `git clone https://github.com/YOUR_USERNAME/stellar-nebula-nomad.git`
3. **Create a feature branch**: `git checkout -b feat/your-feature-name`
4. **Make changes** following our style guide (below)
5. **Test thoroughly** before pushing
6. **Push** to your fork and open a Pull Request

---

## Development Setup

See [DEVELOPER_GUIDE.md](../docs/DEVELOPER_GUIDE.md) for detailed setup instructions.

Quick start:
```bash
rustup update
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli
git clone https://github.com/YOUR_USERNAME/stellar-nebula-nomad.git
cd stellar-nebula-nomad
cargo check --locked
cargo test --locked
```

---

## Code Style Guide

### Rust Code Conventions

**Formatting**:
```bash
cargo fmt
```

**Linting**:
```bash
cargo clippy --all-targets -- -D warnings
```

**Documentation**:
```bash
cargo doc --no-deps
```

### Naming Conventions

- **Functions**: `snake_case` (e.g., `generate_layout`)
- **Types/Structs**: `PascalCase` (e.g., `NebulaLayout`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_REGION_ID`)
- **Modules**: `snake_case` files (e.g., `nebula_gen.rs`)

### Comments & Documentation

Only comment non-obvious behavior:

```rust
/// Generate a deterministic nebula layout for a ship.
///
/// # Parameters
/// - `ship_id` - Unique identifier (must be >= 1)
/// - `region_id` - Galactic region (must be in [1, MAX_REGION_ID])
///
/// # Returns
/// Complete layout with positioned anomalies.
///
/// # Errors
/// Returns error if inputs are invalid or contract not initialized.
pub fn generate_validated_nebula_layout(
    env: Env,
    caller: Address,
    ship_id: u64,
    region_id: u64,
    seed: BytesN<32>,
) -> Result<NebulaLayout, NebulaError>
```

### Error Handling

Prefer `Result<T, E>` over panics:

```rust
// Good
if value < 0 {
    return Err(ContractError::InvalidInput);
}

// Avoid
if value < 0 {
    panic!("Invalid value");
}
```

### Tests

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_case() {
        // Arrange
        let env = Env::default();
        
        // Act
        let result = some_function(&env, valid_input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_error_case() {
        let env = Env::default();
        let result = some_function(&env, invalid_input);
        assert!(result.is_err());
    }
}
```

---

## Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): subject

body (optional)

footer (optional)
```

**Types**:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `test` - Tests
- `refactor` - Code refactoring
- `perf` - Performance improvement
- `chore` - Build/dependency changes
- `ci` - CI/CD changes

**Examples**:

```
feat(nebula_gen): add rate limiting to layout generation

- Prevents DoS attacks by limiting generation per account
- Configurable rate limits per operation type
- Applies cost-based accounting

Closes #123
```

```
fix(input_validation): validate seed bytes not all-zero

Previously allowed degenerate seeds that would generate predictable layouts.
Now rejects seeds where all 32 bytes are identical.

Closes #456
```

---

## Pull Request Process

### Before Submitting

1. **Update main branch**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:
   ```bash
   cargo test --locked
   ```

3. **Run linter**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

4. **Run formatter**:
   ```bash
   cargo fmt
   ```

5. **Update documentation** if needed

### Pull Request Template

```markdown
## Description
Brief description of the changes and why they are needed.

## Related Issues
Closes #123

## Changes
- Bullet point 1
- Bullet point 2

## Testing
How did you test these changes?

- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Tested locally on testnet
- [ ] All existing tests pass

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No new warnings from clippy
- [ ] Commit messages follow format
```

### Review Process

1. Maintainers will review your PR
2. Address feedback and make requested changes
3. Re-request review when ready
4. Maintainer merges after approval

**Response Time**: Try to respond to feedback within 48 hours.

---

## Reporting Bugs

Use GitHub Issues to report bugs. Provide:

1. **Title**: Short description
2. **Environment**: 
   - Rust version: `rustc --version`
   - Soroban CLI version: `soroban --version`
   - OS (Linux/macOS/Windows)

3. **Reproduction**:
   ```bash
   # Minimal steps to reproduce
   1. 
   2.
   3.
   ```

4. **Expected vs Actual**:
   - Expected: What should happen
   - Actual: What actually happens

5. **Logs/Error Messages**: Include full error output

Example:
```
## Title
Contract reverts when ship_id is 0

## Environment
- Rust: 1.75
- Soroban CLI: 20.5.0
- OS: macOS

## Reproduction
1. Initialize contract
2. Call generate_validated_nebula_layout with ship_id=0
3. Contract panics instead of returning InvalidShipId

## Expected
Should return NebulaError::InvalidShipId

## Actual
thread panicked with "value < 1"
```

---

## Suggesting Enhancements

Use GitHub Discussions or Issues (labeled `enhancement`) for feature suggestions.

Include:
- Clear use case and motivation
- Proposed solution (if any)
- Alternative approaches considered
- Potential impact on existing code

---

## Documentation

- **Code comments**: Only for non-obvious behavior
- **Rustdoc**: Document all public APIs with examples
- **README**: Update if you change how to build/deploy
- **DEVELOPER_GUIDE.md**: Update if setup/workflow changes

Build and review docs locally:
```bash
cargo doc --no-deps --open
```

---

## Security Considerations

If you discover a security vulnerability, DO NOT open a public issue.

Instead, email security@space-nebula.dev with:
- Vulnerability description
- Affected code/component
- Proof of concept (if possible)
- Suggested fix (if any)

We will acknowledge within 48 hours and work on a fix.

---

## Licensing

By contributing, you agree that your contributions will be licensed under the same license as the project (check LICENSE file).

---

## Questions?

- **Setup help**: See [DEVELOPER_GUIDE.md](../docs/DEVELOPER_GUIDE.md)
- **Troubleshooting**: See [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)
- **API reference**: See [API_REFERENCE.md](../docs/API_REFERENCE.md)
- **Stellar Developers**: https://developers.stellar.org
- **Stellar Discord**: https://discord.gg/stellar

---

## Contributor Recognition

We recognize all contributions! Contributors are listed in:
- GitHub Contributors page
- Project changelog

Thank you for helping make stellar-nebula-nomad better!
