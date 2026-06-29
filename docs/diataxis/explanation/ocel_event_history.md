# OCEL Event History Logs

This article explains the role of the Object-Centric Event Log (OCEL) in star-toml and why it is decoupled from standing.

## The Role of OCEL

OCEL logging tracks operational lifecycle events. Every configuration read, merge, validation, or validation failure emits an event. These logs provide observability for debugging and compliance.

## Decoupling from Standing

A critical design rule in star-toml is:
- **OCEL logs do not determine standing.**

The mathematical standing $q_{config}$ is strictly derived from the cryptographic witness and the validation checklist. The event logger cannot influence validation checks or create valid typestate wrappers. This prevents circular dependencies and ensure that logging subsystems cannot compromise security bounds.
