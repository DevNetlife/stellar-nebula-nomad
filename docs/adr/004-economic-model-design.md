# ADR 004: Dual-Token Economic Model & Sink Design

## Status
Accepted

## Context
Continuous resource minting without balanced destruction mechanisms leads to token hyper-inflation.

## Decision
We implement a dual-token model with multi-tier resource sinks (crafting loss, ship wear & repair, recycling fees, tournament deposits, guild maintenance).

## Alternatives Considered
- **Pure Burn Taxes:** Punishes active traders; discourages game engagement.
- **Fixed Supply Caps:** Limits economy growth as user base scales.

## Consequences
- **Positive:** Dynamic equilibrium where creation matches destruction within $\pm 10\%$.
- **Negative:** Requires continuous parameter monitoring via `sink_monitor.rs`.
