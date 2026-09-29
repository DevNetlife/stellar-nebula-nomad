# ADR 006: Role-Based Access Control (RBAC) Model

## Status
Accepted

## Context
Administrative actions (parameter adjustments, emergency pauses) require strict authentication and role separation.

## Decision
We implement a modular Role-Based Access Control system (`access_control.rs`) leveraging Soroban `require_auth()` with Admin, Operator, and Governance roles.

## Alternatives Considered
- **Single Owner Address:** Single point of failure if owner key is compromised.
- **Multisig On-Chain per Call:** High gas overhead for routine parameter updates.

## Consequences
- **Positive:** Fine-grained role delegation and security scoping.
- **Negative:** Additional authorization checks in administrative entry points.
