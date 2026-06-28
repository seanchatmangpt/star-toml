# Scope: Implementation & Hardening Track

This scope document defines the milestones and implementation requirements for the Implementation Track of the `star-toml` project, updated with the new design directives.

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
| I1 | Version Bump | Bump version to 26.6.27 in Cargo.toml | None | DONE |
| I2 | Typestate & Conv | Implement `Config<S>` typestate lifecycles and `trusted()` builder (R2, R3) | I1 | DONE |
| I3 | Validation Macros | Implement `#[derive(Validate)]` and `schema!` macros (R4, R5) | I1 | DONE |
| I4 | Save & Lifecycle | Implement canonical saving & lifecycle hooks (R6, R7, R8) | I2, I3 | DONE |
| I5 | Safety & Release | Harden safety validation rules (path traversal, Kelvin, host safety) and pass E2E tests | I4, E1 | DONE |

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
