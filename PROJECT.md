# Project: star-toml Hardening and Release (v26.6.27)

This is the global index for the hardening, testing, and release of `star-toml` version `26.6.27` as a trusted, explicit, typestate-safe configuration substrate.

## Architecture
- `src/lib.rs` - Main entry point and public API.
- `src/loader.rs` - Layered TOML loading and Typestate Lifecycle Abstraction (`Config<S>`).
- `src/merge.rs` - Deep merge of TOML tables/scalars and environment coercion.
- `src/validation.rs` - Validation engine, `Validate` trait, built-in checkers, and multi-error reports.
- `src/schema.rs` - Declarative Schema-based validation and `schema!` macro.
- `src/expand.rs` - Environment variable expansion.
- `star-toml-derive` (or proc-macro module) - Procedural macro `#[derive(Validate)]`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| E1 | E2E Test Suite | Design test cases and infrastructure (Tiers 1-4); publish `TEST_READY.md` | None | DONE |
| E2 | Tier 1-4 Tests | Implement E2E test cases for Tiers 1-4, including typestate lifecycles, derive macro validation, trusted loader, custom checkers, safety tests, and publish `TEST_READY.md` | E1 | DONE |
| I1 | Version Bump | Bump version to 26.6.27 in Cargo.toml | None | DONE |
| I2 | Typestate & Conv | Implement `Config<S>` typestate lifecycles and `trusted()` builder (R2, R3) | None | DONE |
| I3 | Validation Macros | Implement `#[derive(Validate)]` and `schema!` macros (R4, R5) | None | DONE |
| I4 | Save & Lifecycle | Implement canonical saving & lifecycle hooks (R6, R7, R8) | None | DONE |
| I5 | Safety & Release | Harden traversal, null bytes, host safety, and run final release checks (R9, R10) | E1, I1, I2, I3, I4 | DONE |

## Interface Contracts & Typestate Lifecycle
1. **Config Lifecycle**:
   - `Config<Raw>`: initial loaded TOML value.
   - `Config<Merged>`: environment overrides and table merging applied.
   - `Config<Deserialized<T>>`: mapped to Rust struct.
   - `Config<Validated<T>>`: validation invariants checked.
   - `Config<Frozen<T>>`: read-only immutable configuration.
   - `save_canonical` requires `Config<Frozen<T>>` or `Config<Validated<T>>`.
2. **Trust Path**:
   - `star_toml::trusted()` returns `TrustedConfig<T>` containing:
     - `value: T`
     - `source: ConfigSourceReport`
     - `validation: ValidationReport`
     - `digest: ConfigDigest`
3. **Derive validation**:
   - `#[derive(Validate)]` generates `Validate` implementation.
   - `schema!` macro for declarative schema checks.
