# Agent 04 Audit Report: Validation and Error Topology

**Audit Date:** 2026-06-28  
**Auditor:** Agent 04  
**Target Workspace:** [/Users/sac/star-toml](file:///Users/sac/star-toml)  
**Assigned Output Path:** [docs/audit/v26.6.29/agent-04-validation-error-topology.md](file:///Users/sac/star-toml/docs/audit/v26.6.29/agent-04-validation-error-topology.md)

---

## 1. Executive Summary

An audit of the **Validation and Error Topology** boundary of `star-toml` was conducted to verify path-precise validation error tracking, `Severity::Fatal` escalation logic, strict mode operations, FNV-1a variant fingerprinting, and conformance-fitness metrics. 

In version `v26.6.28`, several gaps were noted regarding unrecognized fields permissiveness by default, pathless root-located validation errors in strict mode, non-halting fatal errors, and warning/advisory gating.

These issues have been fully resolved in `v26.6.29`:
1. **Unrecognized Field Permissiveness:** `load_admitted` is now strict by default. It runs `detect_unknown_fields` and rejects unknown fields. A separate `load_admitted_exploratory` loader is introduced for development use where unknown fields are permitted.
2. **Pathless Validation Errors:** Unknown field detection now generates path-precise validation errors using the exact field path (e.g. `loc: Loc(["extra"])` instead of root `Loc(vec![])`), maintaining tree-precision.
3. **Fatal Halting:** `Severity::Fatal` errors are fully preserved, and the validation framework enforces halting invariants correctly when fatal violations occur.
4. **Advisory/Warning Gating:** The configuration validation gate has been decoupled from minor diagnostics. Minor warnings (`Severity::Warning` or `Severity::Advisory`) do not fail validation. Only errors at or above `Severity::Error` block configuration loading. This separates relative fitness metrics from binary admission standing ($q_{config}$).

---

## 2. Command Executions and Evidence

### Command 1: Ripgrep Validation Topology Search
The following ripgrep search command was run to extract all validation, severity, and strict-mode code structures:
```bash
rg -n "trait Validate|ValidationError|ValidationErrors|Severity|Loc|unknown_field|detect_unknown_fields|load_admitted|fatal|repair|fitness|ValidationReport" src tests
```

**Verbatim Findings:**
* [src/validation.rs](file:///Users/sac/star-toml/src/validation.rs):
  - `pub struct Loc(pub(crate) Vec<LocSegment>);`
  - `pub enum Severity { Advisory, Warning, Error, Fatal }`
  - `pub struct ValidationError { pub loc: Loc, pub kind: ErrorKind, pub severity: Severity, ... }`
  - `pub fn repair_hint(&self) -> String`
  - `pub struct ValidationErrors { pub(crate) errors: Vec<ValidationError>, ... }`
  - `pub fn fitness(&self) -> f64`
  - `pub fn variant_id(&self) -> u64`
  - `pub trait Validate { fn validate(&self, v: &mut Validator); ... }`
* [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs):
  - `pub fn detect_unknown_fields<T: Serialize>(original: &Value, typed: &T) -> Vec<String>`
  - `pub fn load_admitted<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(...)`
  - `pub fn load_admitted_exploratory<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(...)`
* [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs):
  - `fn test_load_admitted_strict_rejects_unknown_fields()`
* [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs):
  - Check for `unknown_field_accepted_in_trusted_mode` (PASS)
  - Check for `validation_error_without_path` (PASS)
  - Check for `fatal_error_downgraded` (PASS)

---

## 3. Bound Index
- **B surface inspected:** The `Validate` trait, the `Validator` accumulation API, built-in checks, the `Severity` levels, and strict deserialization rules.
- **O surface inspected:** Structured configuration payloads (`Cfg`, `App`, `Server`, `Tls`) and raw TOML source strings.
- **μ surface inspected:** The state-machine transition from `Deserialized<T>` to `Validated<T>` via the `TrustedLoader` pipeline.
- **C detectors inspected:** Verification checks in `verifier_report.rs` (checks 8, 9, 10, 11).
- **W witnesses inspected:** The `ConfigWitness` hashing inputs, specifically the validation conformance fitness integration.
- **q evidence found:** Verification ladder statuses checked dynamically during `cargo test` and `cargo run --bin verifier_report`.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | `Validate` trait interface, `Validator` checkers, and `Severity` categories. | None. |
| **O** | Raw TOML inputs and struct mock instances processed in tests. | None. |
| **O\*** | Validated state configuration instances `Config<Validated<T>>`. | None. |
| **μ** | Structured parsing to deserialization, followed by `validate()` and transition to `freeze()`. | None. |
| **A** | `ValidationErrors` struct returning FNV-1a fingerprint variant IDs. | None. |
| **C** | Verification checks executed in `verifier_report.rs` (checks 8, 9, 10, 11). | None. |
| **W** | Witness hashing incorporating canonical TOML bytes and fitness. | None. |
| **q** | Standing logic evaluating failset cardinality in the verifier. | None. |

---

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | `test_load_admitted_succeeds` verifies loading a valid config into `AdmittedConfig<T>`. | None. | Confirms valid-path operations. |
| **Falsification** | `test_load_admitted_strict_rejects_unknown_fields` rejects extra settings. | None. | Confirms unknown fields are blocked. |
| **Counterfactual** | `test_unprefixed_admitted_env_override_fails` ensures bad overrides fail. | None. | Confirms boundary safety. |
| **Invariant** | `validation_error_without_path` checks that errors in standard checks have paths. | None. | Validates path-precision. |
| **Metamorphic** | `test_brce_metamorphic_canonical_stability` validates canonical hash consistency. | None. | Confirms structural stability. |
| **Boundary** | `check_range` and `check_size_format` test boundary values. | None. | Confirms range limits. |
| **Conservation** | Winner map captures layer overrides correctly. | None. | Traceability. |
| **Determinism** | `test_witness_is_deterministic` ensures identical runs yield identical hashes. | None. | Relies on sorting. |
| **Idempotence** | `test_brce_idempotence_canonical` confirms stable admission. | None. | Structural stability. |
| **Replay** | `witness_nondeterministic` checks stable witness hashes. | None. | Proof verification. |
| **Provenance** | Winner map captures the layer priority trace. | None. | Provenance audit. |
| **Witness** | `witness_missing_validation_report` checks non-empty witness hashes. | None. | Witness audit. |

---

## 5. Failset

| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `unknown_field_accepted_in_trusted_mode` | Yes | Yes | Yes | **PASS** |
| `validation_not_run` | Yes | Yes | Yes | **PASS** |
| `validation_error_without_path` | Yes | Yes | Yes | **PASS** |
| `fatal_error_downgraded` | Yes | Yes | Yes | **PASS** |

---

## 6. Standing Decision

**Verdict:** **ALIVE**  
*Rationale:* All validation topology and strict loading bugs present in `v26.6.28` are resolved in `v26.6.29`. The validation error paths are precise, fatal severity executes correctly, and minor warnings do not block configuration admission, resulting in a clean and compliant release state.
