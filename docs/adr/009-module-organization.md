# ADR 009: Domain-Driven Module Organization

## Status
Accepted

## Context
As the contract codebase grew to over 100 modules, maintaining clear boundaries and re-exports became essential to prevent circular dependencies.

## Decision
Organize the codebase domain-first (`constants/`, `economics/`, `bridge/`, `notifications/`) with explicit public entry point re-exports at `src/lib.rs`.

## Alternatives Considered
- **Single Monolithic File:** Impossible to maintain and audit.
- **Layer-Based Structuring (models/controllers/views):** Obscures domain relationships in Rust smart contracts.

## Consequences
- **Positive:** Clear responsibility isolation and maintainable compilation units.
- **Negative:** Requires disciplined re-export management in `src/lib.rs`.
