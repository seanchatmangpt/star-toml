# BRCE Standing Audit Report: SemVer, API, Compatibility, and Feature Matrix
**Agent ID**: Agent 09
**Target File**: `docs/audit/v26.6.28-adversarial/agent-09-cross-platform-semver-api.md`
**Local Time**: 2026-06-27T22:20:00-07:00

---

## 1. Executive Summary

An audit of the SemVer boundaries, public API surface, feature gates, and stub compatibility was performed on `star-toml` v26.6.28. The audit revealed a critical compile-time failure and SemVer violation when the optional `wasm4pm-compat` feature is enabled:

1. **Non-Additive Feature Gate (SemVer Violation)**: The function [export_events_to_ocel](file:///Users/sac/star-toml/src/ocel.rs#L85) changes its return type from `()` (when `wasm4pm-compat` is disabled) to [OcelLog](file:///Users/sac/star-toml/src/ocel.rs#L21) (when `wasm4pm-compat` is enabled). Under Cargo's feature unification model, features must be strictly additive. Changing the return signature based on a feature flag is a breaking change for downstream crates that assume the default `()` signature.
2. **Build Failure**: When compiling or testing with `--features wasm4pm-compat`, the build fails in the workspace binary [verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs). The verifier includes a compile-time check (`ocel_treated_as_standing_authority`) asserting that `export_events_to_ocel` returns `()`. When the feature is enabled, this type mismatch causes compile error `E0308` and breaks the entire test suite.
3. **Exploratory vs. Strict Load**: The addition of [load_admitted_exploratory](file:///Users/sac/star-toml/src/loader.rs#L1641) provides a way to parse configurations containing unknown fields. [load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1572) is now strict by default (rejecting unknown fields). While the API signature is preserved, changing the default validation behavior in a patch release is a runtime behavior change. [load_admitted_strict](file:///Users/sac/star-toml/src/loader.rs#L1652) has been appropriately deprecated in `v26.6.28`.

Due to the compilation failure under the `wasm4pm-compat` feature, the standing decision is **BUILD_BROKEN**.

---

## 2. Command Executions and Evidence

### Command 1: Ripgrep Search
The following search was run to trace public APIs, feature gates, and target methods:
```bash
rg -n "pub fn|pub struct|pub enum|deprecated|load_admitted|load_admitted_strict|load_admitted_exploratory|export_events_to_ocel|feature|wasm4pm-compat" src Cargo.toml README.md docs tests
```

**Key Findings:**
- **Feature Declaration**: [Cargo.toml:49](file:///Users/sac/star-toml/Cargo.toml#L49) defines `wasm4pm-compat = ["dep:wasm4pm-compat"]`.
- **Stub Definition**: [src/ocel.rs:84-87](file:///Users/sac/star-toml/src/ocel.rs#L84-L87) defines:
  ```rust
  #[cfg(not(feature = "wasm4pm-compat"))]
  pub fn export_events_to_ocel(_events: &[crate::events::AdmissionEvent]) {
      // No-op stub.
  }
  ```
- **Real Implementation**: [src/ocel.rs:21-22](file:///Users/sac/star-toml/src/ocel.rs#L21-L22) defines:
  ```rust
  #[cfg(feature = "wasm4pm-compat")]
  pub fn export_events_to_ocel(events: &[AdmissionEvent]) -> OcelLog { ... }
  ```
- **Verifier Assertion**: [src/bin/verifier_report.rs:368](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L368) asserts:
  ```rust
  let _unit: () = star_toml::ocel::export_events_to_ocel(&events);
  ```

### Command 2: Cargo Test (Default Features)
```bash
cargo test
```
**Result**: **PASSED**. All 82 unit tests and integration tests compile and pass.

### Command 3: Cargo Test (wasm4pm-compat Feature)
```bash
cargo test --features wasm4pm-compat
```
**Result**: **FAILED**.
**Compiler Error Output:**
```text
error[E0308]: mismatched types
   --> src/bin/verifier_report.rs:368:25
    |
368 |         let _unit: () = star_toml::ocel::export_events_to_ocel(&events);
    |                    --   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `()`, found `OcelLog`
```

---

## 3. Bound Index
- **B surface inspected**: `wasm4pm-compat` feature boundary, optional dependency configuration, stub mappings, and public `load_admitted` API variants.
- **O surface inspected**: [Cargo.toml](file:///Users/sac/star-toml/Cargo.toml), [src/ocel.rs](file:///Users/sac/star-toml/src/ocel.rs), [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs), and [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs).
- **μ surface inspected**: The signature mismatch of `export_events_to_ocel` across feature gates, and the behavior transitions between strict and exploratory loaders.
- **C detectors inspected**: Verifier check 23 (`ocel_treated_as_standing_authority`) in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L356).
- **W witnesses inspected**: Compilation status and compiler diagnostic E0308.
- **q evidence found**: Build success under default features, build failure under unified `wasm4pm-compat` feature.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | `wasm4pm-compat` feature in `Cargo.toml` and conditional compile gates in `src/ocel.rs`. | None. |
| O | Core events and config observation models. | None. |
| O* | Return type expectations for logs. Signature changes from `()` to `OcelLog` depending on feature state. | Consistent return type definition regardless of active features. |
| μ | Signature conversion for `export_events_to_ocel` is non-additive. | Identical function signatures across feature gates. |
| A | `OcelLog` and unit types returned by the mapping. | A unified stub representation of `OcelLog` when the feature is disabled. |
| C | Verifier check 23 in `src/bin/verifier_report.rs` enforces that `export_events_to_ocel` does not return `AdmittedConfig`. | A compiler check implementation that compiles under all feature flag configurations. |
| W | Type checker feedback validating compile status. | Consistent compilation across feature combinations. |
| q | Admissibility verification passes on default features. | Compilation and test success with the `wasm4pm-compat` feature enabled. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | `cargo test` compiles and passes all unit and integration tests under default features. | None. | Confirms core API logic is correct. |
| Falsification | `cargo test --features wasm4pm-compat` fails compiling due to type mismatch in `verifier_report.rs`. | None. | Proves feature unification breaks the build. |
| Counterfactual | None. | None. | None. |
| Invariant | Verifier check 23 ensures `export_events_to_ocel` cannot return `AdmittedConfig`. | A compile-safe mechanism to verify return types. | Core safety constraint exists but breaks the build. |
| Metamorphic | None. | None. | None. |
| Boundary | Optional feature gates `#[cfg(feature = "wasm4pm-compat")]` in `src/ocel.rs`. | A consistent stub definition to prevent type divergence. | API signature divergence breaks compatibility. |
| Conservation | None. | None. | None. |
| Determinism | None. | None. | None. |
| Idempotence | None. | None. | None. |
| Replay | None. | None. | None. |
| Provenance | None. | None. | None. |
| Witness | Cargo compiler type checking. | A consistent type checker validation for all features. | Type mismatch is caught but breaks build. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `ocel_treated_as_standing_authority` | Yes (`src/bin/verifier_report.rs:363`) | Yes (`src/bin/verifier_report.rs`) | Fails compilation on feature activation. | **BUILD_FAIL** (under feature unification) |
| `wasm4pm-compat feature unification safety` | Yes (`Cargo.toml` / `src/ocel.rs`) | Yes (`cargo test --features wasm4pm-compat`) | Yes, breaks builds. | **FAIL** (Non-additive feature signature) |

### Standing Decision
BUILD_BROKEN

---

## 5. Detailed Audit Findings

### 1. Feature Unification Signature Divergence (SemVer Violation)
- **Problem**: The public function `export_events_to_ocel` has two different signatures based on conditional compilation:
  - **Feature disabled**: `pub fn export_events_to_ocel(_events: &[crate::events::AdmissionEvent])` -> returns `()`
  - **Feature enabled**: `pub fn export_events_to_ocel(events: &[AdmissionEvent]) -> OcelLog` -> returns `OcelLog`
- **SemVer Impact**: In Rust, Cargo features must be additive. This signature change is a direct violation of SemVer. If a downstream crate uses `star-toml` without the feature and relies on it returning `()`, its compilation will break if any other crate in the dependency tree enables the `wasm4pm-compat` feature.
- **Remediation**: `OcelLog` (or a stub struct representation) should be defined unconditionally in `star-toml` (even if it's empty when the feature is off). The function `export_events_to_ocel` should *always* return `OcelLog`, regardless of the feature gate.

### 2. Verifier Check Compile-Time Failure
- **Problem**: When `wasm4pm-compat` is enabled, the workspace binary `verifier_report.rs` fails to compile because line 368:
  ```rust
  let _unit: () = star_toml::ocel::export_events_to_ocel(&events);
  ```
  mismatches with `OcelLog`.
- **Impact**: The workspace cannot run its full test suite or verification report when the feature is enabled. This prevents continuous integration validation and deployment.
- **Remediation**: The verifier check must be updated to dynamically allow the returned type or be gated so that it compiles under both configurations, but ideally the API itself should be made consistent as described in Finding 1.

### 3. Runtime Behavioral Change in `load_admitted`
- **Problem**: In `v26.6.28`, `load_admitted` was modified to be strict by default (rejecting unknown fields).
- **SemVer Impact**: Although the type signature remains identical, changing the default parsing behavior (from accepting/ignoring unknown fields to rejecting them with validation errors) in a patch release (`26.6.27` -> `26.6.28`) changes runtime contract expectations and could break working downstream systems.
- **Mitigation**: The introduction of `load_admitted_exploratory` provides a way for users to preserve the old behavior. The alias `load_admitted_strict` has been appropriately deprecated.
