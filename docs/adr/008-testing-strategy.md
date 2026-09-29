# ADR 008: Multi-Tier Testing Strategy

## Status
Accepted

## Context
Smart contracts managing financial assets and NFT logic require high assurance against reentrancy, overflow, and state corruption bugs.

## Decision
We enforce a 4-tier testing pipeline:
1. Unit tests (mod level).
2. Integration tests via Soroban host environment (`tests/`).
3. Chaos network/storage fault injection (`tests/chaos/`).
4. Property-based fuzz testing (`fuzz/`).

## Alternatives Considered
- **Unit Testing Only:** Fails to catch cross-module interaction bugs.
- **Manual Testnet Testing:** Slow iteration cycles and incomplete edge-case coverage.

## Consequences
- **Positive:** High security confidence and early detection of economic exploit vectors.
- **Negative:** Longer overall CI test execution times.
