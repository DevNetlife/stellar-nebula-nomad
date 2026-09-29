# ADR 007: Event Emission & Subgraph Indexing Strategy

## Status
Accepted

## Context
Off-chain applications and user interfaces need real-time updates regarding state changes (trades, ship upgrades, pricing updates).

## Decision
Emit structured Soroban events via `env.events().publish()` using standardized topic vectors for indexing by subgraphs and RPC indexers.

## Alternatives Considered
- **Polling Storage State:** High RPC bandwidth and latent UI updates.
- **Custom Logging Sidecars:** Adds centralized server requirement.

## Consequences
- **Positive:** Low-cost real-time telemetry for frontend interfaces and indexers.
- **Negative:** Events must be carefully schema-versioned to prevent indexer breakage.
