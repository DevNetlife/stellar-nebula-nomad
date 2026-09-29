# ADR 001: Rust & Soroban SDK Technology Choice

## Status
Accepted

## Context
Stellar Nebula Nomad requires a smart contract framework capable of performing complex mathematical calculations (procedural nebula grid generation, dynamic bonding curves, TWAP pricing) within low gas budgets and predictable WASM runtime constraints.

## Decision
We chose **Rust** compiled to WebAssembly (WASM) via the **Soroban SDK** on the Stellar network.

## Alternatives Considered
- **EVM / Solidity (Ethereum / L2s):** Higher gas fees for heavy procedural matrix calculations; non-deterministic gas variations.
- **Move / CosmWasm:** Smaller ecosystem on Stellar; requires custom cross-chain bridging infrastructure.

## Consequences
- **Positive:** Type safety, zero-cost abstractions, memory safety without garbage collection overhead, seamless integration with Stellar native assets.
- **Negative:** Strict `#![no_std]` constraints requiring careful allocation management.
