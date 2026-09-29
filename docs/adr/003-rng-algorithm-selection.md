# ADR 003: Deterministic Pseudo-Random Number Generator (PRNG) Selection

## Status
Accepted

## Context
Procedural generation and loot table evaluations require fast, pseudo-random output reproducible across nodes without state divergence.

## Decision
We select PCG32 (Permuted Congruential Generator) seeded by Stellar ledger hash and user action salt.

## Alternatives Considered
- **Linear Congruential Generator (LCG):** Poor statistical distribution, easy to exploit.
- **ChaCha20 / Cryptographic RNG:** High gas overhead for non-cryptographic gameplay outcomes.

## Consequences
- **Positive:** Uniform statistical quality, minimal gas footprint.
- **Negative:** Not suitable for secret key generation (gameplay randomness only).
