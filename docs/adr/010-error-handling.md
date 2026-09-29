# ADR 010: Standardized Error Handling Approach

## Status
Accepted

## Context
Soroban contract calls return error codes across WASM host boundaries. Inconsistent error enums cause opaque contract invocations for client dApps.

## Decision
All contract errors derive `#[contracterror]` with explicit `#[repr(u32)]` discriminants mapped to standardized domain categories (`error_standard.rs`).

## Alternatives Considered
- **String Panic Messages (`panic!("...")`):** Consumes excessive WASM string byte overhead and cannot be cleanly parsed by client SDKs.
- **Generic Error Codes (1, 2, 3... reused across modules):** Ambiguous diagnostic reporting.

## Consequences
- **Positive:** O(1) error mapping and clean dApp SDK error handling.
- **Negative:** Requires maintaining global uniqueness of error enum discriminants per module.
