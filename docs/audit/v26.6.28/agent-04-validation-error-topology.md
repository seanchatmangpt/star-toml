# Agent 04 Audit Report: Validation and Error Topology

**Audit Date:** 2026-06-27  
**Auditor:** Agent 04  
**Target Workspace:** [/Users/sac/star-toml](file:///Users/sac/star-toml)  
**Assigned Output Path:** [docs/audit/v26.6.28/agent-04-validation-error-topology.md](file:///Users/sac/star-toml/docs/audit/v26.6.28/agent-04-validation-error-topology.md)

---

## 1. Executive Summary

An audit of the **Validation and Error Topology** boundary of `star-toml` was conducted to verify path-precise validation error tracking, `Severity::Fatal` escalation logic, strict mode operations, FNV-1a variant fingerprinting, and conformance-fitness metrics. The audit reveals critical discrepancies between the declared specifications (specifically [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md) and [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md)) and the implemented codebase:

1. **Unrecognized Field Permissiveness:** The standard [TrustedLoader::load](file:///Users/sac/star-toml/src/loader.rs#L826) and [TrustedLoader::load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1569) methods do not check for unknown keys, silently admitting them by default instead of rejecting them as required by [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md).
2. **Pathless Validation Errors in Strict Mode:** When [TrustedLoader::load_admitted_strict](file:///Users/sac/star-toml/src/loader.rs#L1584) rejects unknown keys, it generates a [ValidationError](file:///Users/sac/star-toml/src/validation.rs#L302) with an empty key location (`loc: Loc(vec![])`), violating the "No Pathless Errors" invariant of [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md).
3. **Non-Halting Fatal Errors:** Recording an error with [Severity::Fatal](file:///Users/sac/star-toml/src/validation.rs#L202) does not halt the validation runner's execution. Subsequent checks are still evaluated, violating the short-circuiting requirement.
4. **Conflating Fitness and Standing:** Admitted configs hardcode `validation_fitness = 1.0`, and the validation runner rejects any configuration producing informational [Severity::Warning](file:///Users/sac/star-toml/src/validation.rs#L197) or [Severity::Advisory](file:///Users/sac/star-toml/src/validation.rs#L195) diagnostics. This conflates the conformance ratio (`fitness`) with absolute admissibility (`q_config`), blocking otherwise usable configs.

---

## 2. Command Executions and Evidence

### Command 1: Ripgrep Validation Topology Search
The following ripgrep search command was run to extract all validation, severity, and strict-mode code structures:
```bash
rg -n "trait Validate|ValidationError|ValidationErrors|Severity|Loc|unknown_field|detect_unknown_fields|load_admitted_strict|fatal|repair|fitness|ValidationReport" src tests
```

**Verbatim Findings:**
* [src/validation.rs:121](file:///Users/sac/star-toml/src/validation.rs#L121): `pub struct Loc(pub(crate) Vec<LocSegment>);`
* [src/validation.rs:193](file:///Users/sac/star-toml/src/validation.rs#L193): `pub enum Severity { Advisory, Warning, Error, Fatal }`
* [src/validation.rs:302](file:///Users/sac/star-toml/src/validation.rs#L302): `pub struct ValidationError { pub loc: Loc, pub kind: ErrorKind, pub severity: Severity, ... }`
* [src/validation.rs:336](file:///Users/sac/star-toml/src/validation.rs#L336): `pub fn repair_hint(&self) -> String { ... }` (Computes non-authoritative formatting suggestions)
* [src/validation.rs:392](file:///Users/sac/star-toml/src/validation.rs#L392): `pub struct ValidationErrors { pub(crate) errors: Vec<ValidationError>, ... }`
* [src/validation.rs:464](file:///Users/sac/star-toml/src/validation.rs#L464): `pub fn fitness(&self) -> f64 { ... }` (Calculates conformance ratio)
* [src/validation.rs:481](file:///Users/sac/star-toml/src/validation.rs#L481): `pub fn variant_id(&self) -> u64 { ... }` (Fingerprints errors using FNV-1a over sorted `loc:code` pairs)
* [src/validation.rs:1003](file:///Users/sac/star-toml/src/validation.rs#L1003): `pub trait Validate { fn validate(&self, v: &mut Validator); ... }`
* [src/loader.rs:1502](file:///Users/sac/star-toml/src/loader.rs#L1502): `pub fn detect_unknown_fields<T: Serialize>(original: &Value, typed: &T) -> Vec<String>`
* [src/loader.rs:1584](file:///Users/sac/star-toml/src/loader.rs#L1584): `pub fn load_admitted_strict<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(...)`
* [tests/brce.rs:555](file:///Users/sac/star-toml/tests/brce.rs#L555): `fn test_load_admitted_strict_rejects_unknown_fields()`
* [src/bin/verifier_report.rs:158](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L158): check for `unknown_field_accepted_in_trusted_mode`
* [src/bin/verifier_report.rs:195](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L195): check for `fatal_error_downgraded`

---

## 3. Bound Index
- **B surface inspected:** The [Validate](file:///Users/sac/star-toml/src/validation.rs#L1003) trait, the [Validator](file:///Users/sac/star-toml/src/validation.rs#L545) accumulation API, built-in checks (`check_semver`, `check_ip_or_domain`, `check_size_format`, `check_path_safe`, `check_consistent`), the [Severity](file:///Users/sac/star-toml/src/validation.rs#L193) levels, and strict deserialization rules.
- **O surface inspected:** Structured configuration payloads (`Cfg`, `App`, `Server`, `Tls`) and raw TOML source strings.
- **μ surface inspected:** The state-machine transition from `Deserialized<T>` to `Validated<T>` via the [TrustedLoader](file:///Users/sac/star-toml/src/loader.rs#L765) pipeline and strict loading entry points.
- **C detectors inspected:** Verifier rules in `verifier_report.rs` (`unknown_field_accepted_in_trusted_mode`, `validation_error_without_path`, `fatal_error_downgraded`).
- **W witnesses inspected:** The `ConfigWitness` hashing inputs, specifically the validation conformance fitness integration.
- **q evidence found:** Verification ladder statuses checked dynamically during `cargo test` and `cargo run --bin verifier_report`.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | [Validate](file:///Users/sac/star-toml/src/validation.rs#L1003) trait interface, [Validator](file:///Users/sac/star-toml/src/validation.rs#L545) checkers, and [Severity](file:///Users/sac/star-toml/src/validation.rs#L193) categories. | None. |
| **O** | Raw TOML inputs and struct mock instances processed in tests. | None. |
| **O\*** | Validated state configuration instances `Config<Validated<T>>`. | None. |
| **μ** | Structured parsing to deserialization, followed by `validate()` and transition to `freeze()`. | None. |
| **A** | [ValidationErrors](file:///Users/sac/star-toml/src/validation.rs#L392) struct returning FNV-1a fingerprint variant IDs. | None. |
| **C** | Verification checks executed in `verifier_report.rs` (checks 8, 9, 10, 11). | None. |
| **W** | Witness hashing incorporating canonical TOML bytes and fitness. | None. |
| **q** | Standing logic evaluating failset cardinality in the verifier. | The verifier does not check if `load()` fails to reject unknown fields or if `with_severity(Fatal)` fails to abort execution early. |

---

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | `test_load_admitted_succeeds` verifies loading a valid config into `AdmittedConfig<T>`. | None. | Confirms valid-path operations. |
| **Falsification** | `test_load_admitted_strict_rejects_unknown_fields` rejects extra settings. | No test verifying that `load` (trusted mode) rejects unknown fields. | Permissive bypass threat remains. |
| **Counterfactual** | `test_unprefixed_admitted_env_override_fails` ensures bad overrides fail. | None. | Confirms boundary safety. |
| **Invariant** | `validation_error_without_path` checks that errors in standard checks have paths. | Verification that strict mode `unknown_field` errors do not violate path invariants. | Pathless errors generated in strict mode. |
| **Metamorphic** | `test_brce_metamorphic_canonical_stability` validates canonical hash consistency. | None. | Confirms structural stability. |
| **Boundary** | `check_range` and `check_size_format` test boundary values. | None. | Confirms range limits. |
| **Conservation** | Winner map captures layer overrides correctly. | None. | Traceability. |
| **Determinism** | `test_witness_is_deterministic` ensures identical runs yield identical hashes. | None. | Relies on sorting. |
| **Idempotence** | `test_brce_idempotence_canonical` confirms stable admission. | None. | Structural stability. |
| **Replay** | `witness_nondeterministic` checks stable witness hashes. | None. | Proof verification. |
| **Provenance** | Winner map captures the layer priority trace. | None. | Provenance audit. |
| **Witness** | `witness_missing_validation_report` checks non-empty witness hashes. | Negative witness tests proving witness changes when validation files are missing. | Weak witness audit. |

---

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `unknown_field_accepted_in_trusted_mode` | Yes (`verifier_report.rs:158`) | Yes (`tests/brce.rs:555`) | **No** (only `load_admitted_strict` rejects it; default `load` permits it) | **ACTIVE FAILURE** |
| `validation_not_run` | Yes (`verifier_report.rs:168`) | Yes (`tests/brce.rs:78`) | Yes | Mitigated |
| `validation_error_without_path` | Yes (`verifier_report.rs:179`) | Yes (`tests/brce.rs:414`) | **No** (strict mode unknown-field errors are root-located with empty `loc`) | **ACTIVE FAILURE** |
| `fatal_error_downgraded` | Yes (`verifier_report.rs:194`) | Yes (`tests/validation.rs`) | **No** (fatal severity does not short-circuit validation chain) | **ACTIVE FAILURE** |

---

## 5. Standing Decision

**Verdict:** **PARTIAL_ALIVE**  
*Rationale:* While the core validation traits and built-in checkers are implemented and functional, three critical safety invariants are broken:
- Unknown fields are silently accepted in the default trusted mode (`load`/`load_admitted`), violating [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md).
- Strict mode unknown field rejection generates a pathless error (`Loc(vec![])`), violating the path-precision requirement of [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md).
- `Severity::Fatal` errors do not short-circuit or halt the validation chain.

---

## 6. Detailed Audit Findings (Falsifier Evaluations)

### Falsifier 1: unknown field accepted in trusted/strict mode
* **Verdict**: **True**  
* **Analysis**: Under [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md), the default behavior of trusted configuration loading must reject unknown fields:
  > "Enabling deny_unknown_fields behavior must be the default behavior for all trusted config loads."
  
  However, in [TrustedLoader::load](file:///Users/sac/star-toml/src/loader.rs#L826) and [TrustedLoader::load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1569), the TOML data is parsed and passed straight to Serde's default deserialization via [deserialize_value](file:///Users/sac/star-toml/src/loader.rs#L471). Serde defaults to ignoring unknown fields unless a struct is decorated with `#[serde(deny_unknown_fields)]`. No automatic unknown-field check is run inside `load` or `load_admitted`.
  
  Only [TrustedLoader::load_admitted_strict](file:///Users/sac/star-toml/src/loader.rs#L1584) runs [detect_unknown_fields](file:///Users/sac/star-toml/src/loader.rs#L1502). Therefore, "trusted mode" permits unknown fields by default, directly violating the type-boundary specification.

### Falsifier 2: validation error without path
* **Verdict**: **True**  
* **Analysis**: The specification in [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) requires that:
  > "Every semantic validation error MUST have a precise dotted key path locating it in the configuration tree."
  
  When `load_admitted_strict` detects unrecognized fields, it records a validation error in [src/loader.rs:1606](file:///Users/sac/star-toml/src/loader.rs#L1606):
  ```rust
  let errors = vec![crate::validation::ValidationError {
      loc: crate::validation::Loc(vec![]),
      kind: crate::validation::ErrorKind::Predicate { code: "unknown_field" },
      severity: crate::validation::Severity::Error,
      input: Some(unknown.join(", ")),
      msg,
  }];
  ```
  This hardcodes the error location as `Loc(vec![])`, which points to the root of the configuration tree rather than the exact field paths (e.g. `extra_field`). This is an active pathless error violating the core tree-precision guarantee.

### Falsifier 3: fatal severity downgraded
* **Verdict**: **True**  
* **Analysis**: The severity specification in [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) mandates:
  > "Any check marked as Fatal (or generating an error with Severity::Fatal) must immediately halt further check execution..."
  
  In the implementation of [Validator::record](file:///Users/sac/star-toml/src/validation.rs#L954) and the various check helpers (e.g., `check_non_empty`, `check_range`), there is no logic to intercept fatal errors. The validation closure continues executing subsequent rules and pushes all errors to the `errors` vector regardless of whether a `Fatal` severity has been reached. While the error retains its `Fatal` severity label in the output list, it is functionally downgraded to a non-halting warning during validation execution.

### Falsifier 4: repair hint treated as authority
* **Verdict**: **False**  
* **Analysis**: The validation rules state that `RepairHint != Authority`.
  
  The implementation of [ValidationError::repair_hint](file:///Users/sac/star-toml/src/validation.rs#L336) generates informational strings suggesting edits (such as recommending allowed values or range adjustments). The core crate never applies these changes automatically or uses them as a source of authority to mutate configurations. Repaired configuration payloads must always go back through the full validation and parsing pipeline before receiving admission standing.

### Falsifier 5: fitness treated as q_config
* **Verdict**: **True**  
* **Analysis**: Under the BRCE framework, standing ($q_{config}$) is a binary quality signal requiring failset-zero, lawful lifecycles, and cryptographic witnesses. In contrast, `fitness` represents a conformance score (`PassedChecks / TotalChecks`).
  
  In the implementation of [build_admitted](file:///Users/sac/star-toml/src/loader.rs#L1635), the loader hardcodes `validation_fitness = 1.0_f64` (line 1646) for all admitted configurations.
  
  Furthermore, the validation engine fails on **any** validation diagnostics (lines 853-861, 940-945), including those of severity `Advisory` or `Warning`. By failing validation when informational or minor warnings are present, the runner prevents the loading of configurations that would have `fitness < 1.0`. Thus, `fitness` is treated as a hard gate identical to the binary standing gate ($q_{config} = 1$), rather than serving as a relative metrics signal.
