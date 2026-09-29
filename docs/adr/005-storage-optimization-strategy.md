# ADR 005: Soroban Storage Tiering & Optimization Strategy

## Status
Accepted

## Context
Soroban provides three distinct storage tiers: Instance, Persistent, and Temporary. Choosing the wrong tier leads to state eviction or excessive rent.

## Decision
- **Instance Storage:** Global contract configurations, governance keys, sink metrics.
- **Persistent Storage:** Player balances, ship NFT states, user profiles.
- **Temporary Storage:** Short-lived session nonces and transient cache keys.

## Alternatives Considered
- **All Persistent:** Unnecessary rent costs for temporary metadata.
- **All Instance:** Exceeds Soroban instance storage payload limits.

## Consequences
- **Positive:** Optimized ledger footprint and predictable storage rent costs.
- **Negative:** Requires managing TTL extensions for persistent storage keys.
