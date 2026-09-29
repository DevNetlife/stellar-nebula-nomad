# ADR 002: Procedural Nebula Generation Strategy

## Status
Accepted

## Context
Storing pre-rendered universe coordinates on-chain is cost-prohibitive due to storage ledger fees.

## Decision
We implement procedural nebula layout generation using deterministic 2D Simplex/Perlin noise algorithms seeded by coordinate hashes (`(x, y, seed)`).

## Alternatives Considered
- **Pre-computed On-Chain Storage:** Storing grid maps on-chain costs thousands of XLM per sector.
- **Off-Chain Server Generation:** Reduces decentralization and introduces centralized dependency.

## Consequences
- **Positive:** Infinite deterministic world generation with zero persistent grid storage costs.
- **Negative:** Minor WASM CPU computation during sector discovery.
