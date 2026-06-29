# Understanding Authority Boundaries

This article explains how authority boundaries restrict and delegate configuration control in star-toml.

## Context: Sandbox Constraints

In modern infrastructure, configurations contain values that affect security and resource allocations (e.g. log paths, maximum cache sizes). If downstream deployments can arbitrarily modify these, it creates operational risks.

## Solution: Explicit Boundaries

An Authority Boundary defines exactly:
1. **Allowed Control**: Fields that can be modified by local configuration templates.
2. **Forbidden Claims**: Values that are locked and cannot be claimed.
3. **Delegation Targets**: External identities (e.g. `platform-ops`) trusted to modify variables.

The loader verifies the boundary structure during validation, rejecting configurations that violate boundaries.
