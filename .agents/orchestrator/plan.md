# Project Plan: star-toml Hardening and Release (v26.6.27)

This plan outlines the dual-track strategy to harden, test, and release `star-toml` version `26.6.27` as a trusted, explicit, typestate-safe configuration substrate.

## Architecture & Code Layout
- `src/lib.rs` - Main entry point and public API.
- `src/loader.rs` - Layered TOML loading and Typestate Lifecycle Abstraction (`Config<S>`).
- `src/merge.rs` - Deep merge of TOML tables/scalars and environment coercion.
- `src/validation.rs` - Validation engine, `Validate` trait, built-in checkers, and multi-error reports.
- `src/schema.rs` - Declarative Schema-based validation and `schema!` macro.
- `src/expand.rs` - Environment variable expansion.
- `star-toml-derive` (or proc-macro module) - Procedural macro `#[derive(Validate)]`.

## Milestones

### Track 1: E2E Testing Track (Requirement-Driven)
The E2E Testing Track runs independently to construct a requirement-driven test suite.
- **Milestone E1: Test Infrastructure Design**
  - Establish E2E testing infrastructure.
  - Create `TEST_INFRA.md`.
  - Status: `IN_PROGRESS` (conv: f0616e4f-e52f-4731-956b-25682da8e271)
- **Milestone E2: E2E Test Suite Creation**
  - Implement Tier 1 (Feature Coverage), Tier 2 (Boundary/Corner), Tier 3 (Cross-Feature), and Tier 4 (Real-World) tests.
  - Test areas: typestate transition safety, trusted config output, macro validation, canonical saving, lifecycle hooks.
  - Publish `TEST_READY.md` when completed.
  - Status: `PLANNED`

### Track 2: Implementation & Hardening Track
The Implementation Track implements the code changes and verifies them against the E2E test suite.
- **Milestone I1: Version Bump (R1)**
  - Bump `Cargo.toml` package version to `26.6.27`.
  - Status: `IN_PROGRESS` (conv: 5b7ec365-a7ba-4428-a41f-ba63f899910f)
- **Milestone I2: Typestate & Conv (R2, R3)**
  - Implement `Config<S>` typestate lifecycles and `trusted()` builder.
  - Status: `PLANNED` (conv: 5b7ec365-a7ba-4428-a41f-ba63f899910f)
- **Milestone I3: Validation Macros (R4, R5)**
  - Implement `#[derive(Validate)]` macro and `schema!` macro.
  - Status: `PLANNED` (conv: 5b7ec365-a7ba-4428-a41f-ba63f899910f)
- **Milestone I4: Save & Lifecycle Hooks (R6, R7, R8)**
  - Implement canonical saving (`save_canonical`, `save_file`, `save_pretty`) and lifecycle hooks (`ConfigLifecycle<T>`).
  - Status: `PLANNED` (conv: 5b7ec365-a7ba-4428-a41f-ba63f899910f)
- **Milestone I5: Safety Verification & Release Validation (R9, R10)**
  - Run E2E tests, clippy Warnings check, formatting, and `cargo publish --dry-run`.
  - Perform Tier 5 (Adversarial Coverage Hardening) white-box testing.
  - Status: `PLANNED`

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
