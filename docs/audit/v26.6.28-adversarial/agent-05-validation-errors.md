# Agent 05 Audit Report: Validation, Unknown Fields, and Error Topology

**Audit Date:** 2026-06-27  
**Auditor:** Agent 05  
**Target Workspace:** [/Users/sac/star-toml](file:///Users/sac/star-toml)  
**Assigned Output Path:** [docs/audit/v26.6.28-adversarial/agent-05-validation-errors.md](file:///Users/sac/star-toml/docs/audit/v26.6.28-adversarial/agent-05-validation-errors.md)

---

## 1. Executive Summary

An audit of the **Validation, Unknown Fields, and Error Topology** boundary of `star-toml` was conducted for the `v26.6.28-adversarial` release. The audit analyzed mechanisms for unknown key detection, precision of error locations, fatal error immutability, and validation metrics. While some security improvements have been implemented (such as resolving Windows-style separator traversals and adding sandbox root tracking in path witnesses), the core validation and admission pipelines still contain three critical safety violations:

1. **Array of Tables Unknown Field Bypass:** The unknown fields detector ([detect_unknown_fields](file:///Users/sac/star-toml/src/loader.rs#L1502)) relies on [collect_unknown_keys](file:///Users/sac/star-toml/src/loader.rs#L1512), which only matches key-value mappings of `Value::Table`. It does not support `Value::Array`. As a result, any unrecognized/unknown field defined inside an array of tables is silently accepted by [load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1572), violating the ST-106 specification.
2. **Allowed Pathless Validation Errors:** The system design does not prevent or assert against root-level validation errors (which carry an empty `Loc(vec![])`). Furthermore, the codebase explicitly maintains a unit test ([root_level_error_renders_as_root](file:///Users/sac/star-toml/src/validation.rs#L1194)) asserting that pathless errors are supported and rendered as `(root)`, violating the "No Pathless Errors" invariant of the ST-107 specification.
3. **Non-Halting Fatal Errors:** Generating an error with `Severity::Fatal` does not short-circuit or halt the validation runner. Subsequent validation rules and nested structures are still fully evaluated, and all diagnostics are collected. The verifier check passes only because it checks if the `Fatal` label is preserved in the final output, not whether execution was aborted early.
4. **Fitness Conflation:** The loader hardcodes `validation_fitness = 1.0` in `build_admitted` ([src/loader.rs:1670](file:///Users/sac/star-toml/src/loader.rs#L1670)) for all admitted configurations. Because the validation runner rejects configurations producing any error (including minor `Advisory` or `Warning` diagnoses), it conflates partial conformance metrics (`fitness < 1.0`) with binary admissibility ($q_{config} = 1.0$), preventing warnings from coexisting with successful configuration loading.

---

## 2. Command Executions and Evidence

### Command 1: Ripgrep Validation Search
The following ripgrep search command was run to locate key validation, severity, and strict-mode code structures:
```bash
rg -n "detect_unknown_fields|unknown_field|load_admitted_strict|load_admitted_exploratory|ValidationError|ValidationErrors|Loc|LocSegment|Severity|Fatal|ErrorKind|repair|fitness|is_root|Predicate" src tests
```

**Verbatim Findings:**
* [src/validation.rs:121](file:///Users/sac/star-toml/src/validation.rs#L121): `pub struct Loc(pub(crate) Vec<LocSegment>);`
* [src/validation.rs:193](file:///Users/sac/star-toml/src/validation.rs#L193): `pub enum Severity { Advisory, Warning, Error, Fatal }`
* [src/validation.rs:302](file:///Users/sac/star-toml/src/validation.rs#L302): `pub struct ValidationError { pub loc: Loc, pub kind: ErrorKind, pub severity: Severity, ... }`
* [src/validation.rs:336](file:///Users/sac/star-toml/src/validation.rs#L336): `pub fn repair_hint(&self) -> String { ... }` (suggests minimal fixes for built-in checks)
* [src/validation.rs:392](file:///Users/sac/star-toml/src/validation.rs#L392): `pub struct ValidationErrors { pub(crate) errors: Vec<ValidationError>, ... }`
* [src/validation.rs:464](file:///Users/sac/star-toml/src/validation.rs#L464): `pub fn fitness(&self) -> f64 { ... }` (conformance fitness metric calculation)
* [src/loader.rs:1502](file:///Users/sac/star-toml/src/loader.rs#L1502): `pub fn detect_unknown_fields<T: Serialize>(original: &Value, typed: &T) -> Vec<String>`
* [src/loader.rs:1572](file:///Users/sac/star-toml/src/loader.rs#L1572): `pub fn load_admitted<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(...)`
* [src/loader.rs:1652](file:///Users/sac/star-toml/src/loader.rs#L1652): `pub fn load_admitted_strict<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(...)`
* [tests/brce.rs:555](file:///Users/sac/star-toml/tests/brce.rs#L555): `fn test_load_admitted_strict_rejects_unknown_fields()`
* [src/bin/verifier_report.rs:159](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L159): verifier check for `unknown_field_accepted_in_trusted_mode`
* [src/bin/verifier_report.rs:183](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L183): verifier check for `validation_error_without_path`
* [src/bin/verifier_report.rs:215](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L215): verifier check for `fatal_error_downgraded`

---

## 3. Bound Index
- **B surface inspected:** The [Validate](file:///Users/sac/star-toml/src/validation.rs#L1009) trait, the [Validator](file:///Users/sac/star-toml/src/validation.rs#L545) context, built-in validation methods (`check_semver`, `check_ip_or_domain`, `check_size_format`, `check_path_safe`, `check_consistent`, `check_profile`, `check_policy`), `Severity`, `ErrorKind`, `Loc`, and `LocSegment`.
- **O surface inspected:** Structured configuration payloads (`Cfg`, `App`, `Server`, `Tls`), mock payloads in tests, and raw TOML source inputs.
- **μ surface inspected:** `validate()`, `validate_lifecycle()`, `deserialize()`, `load_admitted()`, `load_admitted_exploratory()`, and `load_admitted_strict()`.
- **C detectors inspected:** Verifier rules in `verifier_report.rs` (`unknown_field_accepted_in_trusted_mode`, `validation_not_run`, `validation_error_without_path`, `fatal_error_downgraded`).
- **W witnesses inspected:** `ConfigWitness` generation schema, validating the integration of validation conformance fitness.
- **q evidence found:** Dynamic verifier statuses and unit/integration tests in `tests/brce.rs`, `tests/adversarial.rs`, and `tests/e2e_tests.rs`.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | `Validate` trait interface, `Validator` struct, built-in validation checks, and `Severity` stratification levels. | None. |
| **O** | Raw TOML inputs and configuration structs processed in tests. | None. |
| **O\*** | Validated configuration states: `Config<Validated<T>>` and `AdmittedConfig<T>`. | None. |
| **μ** | Deserialization to `Deserialized<T>` followed by `validate()` and transition to `freeze()`. | None. |
| **A** | `ValidationErrors` struct returning FNV-1a fingerprint variant IDs. | None. |
| **C** | Verification checks executed in `verifier_report.rs` (checks 8, 9, 10, 11). | None. |
| **W** | Witness hashing incorporating validation fitness, canonical bytes, and provenance reports. | None. |
| **q** | Standing logic evaluating validation errors and fitness bounds in the verifier. | The verifier does not check if `load_admitted` fails to reject unknown fields inside arrays of tables, or if `Severity::Fatal` fails to abort execution early. |

---

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | Unit tests in `src/validation.rs` (`locations_are_path_precise`, `repair_hint_for_empty`) verify expected passing and failing constraints. | None. | High (Positive validation). |
| **Falsification** | `test_load_admitted_strict_rejects_unknown_fields` checks rejection of unknown fields. | No tests verify rejection of unknown fields inside arrays of tables. | Critical (Vulnerability untested). |
| **Counterfactual** | `verifier_report.rs` runs checks on unknown fields, pathless validation errors, and fatal error downgrading. | None. | Confirms boundary safety. |
| **Invariant** | `validation_error_without_path` checks that errors have path-precise locations. | Assertions in the runtime prohibiting root-level pathless validation errors. | Root-level pathless errors are supported and unit-tested. |
| **Metamorphic** | `same_error_pattern_same_variant_id` confirms FNV-1a fingerprint variant hashing is stable. | None. | Confirms structural stability. |
| **Boundary** | Range checks (`check_range`) test boundary values. | None. | Confirms range limits. |
| **Conservation** | Total atomic checks run (`checks_run`) is tracked to compute fitness. | None. | Traceability. |
| **Determinism** | FNV-1a fingerprint variant hashing sorts location and error codes before hashing, ensuring stable variant IDs. | None. | Relies on sorting. |
| **Idempotence** | Re-running validation yields identical errors and variant IDs. | None. | Structural stability. |
| **Replay** | `ValidationErrors` deserialization and variant hashing. | None. | Proof verification. |
| **Provenance** | `source_path` tracked inside the `Validator` to resolve path safety constraints relative to the configuration file source parent directory. | None. | Provenance audit. |
| **Witness** | Validation fitness is integrated into the `ConfigWitness` hash. | Verification that warning/advisory diagnostics are admitted and witness reflects correct fitness < 1.0. | Hardcoded fitness prevents witness representation. |

---

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `unknown_field_accepted_in_trusted_mode` | Yes (`verifier_report.rs:159`) | Yes (`tests/brce.rs:555`) | **No** (fails to detect unknown fields nested within arrays of tables due to `collect_unknown_keys` neglecting arrays). | **ACTIVE FAILURE** |
| `validation_not_run` | Yes (`verifier_report.rs:168`) | Yes (`tests/brce.rs:78`) | Yes | Mitigated |
| `validation_error_without_path` | Yes (`verifier_report.rs:183`) | Yes (`tests/brce.rs:570`) | **No** (allows root-level pathless errors at runtime, and contains a unit test asserting root-level pathless errors are supported). | **ACTIVE FAILURE** |
| `fatal_error_downgraded` | Yes (`verifier_report.rs:215`) | Yes (`tests/validation.rs`) | **No** (the verifier check only checks if the Fatal severity label is preserved. However, fatal errors fail to halt validation execution, meaning subsequent checks are evaluated anyway). | **ACTIVE FAILURE** |

---

## 5. Standing Decision

**Verdict:** **PARTIAL_ALIVE**  
*Rationale:* Although the core validation traits and built-in checkers are implemented and functional, three critical safety invariants are broken:
- Unknown fields nested within arrays of tables are silently accepted, bypassing `load_admitted`.
- Strict mode allows and unit-tests root-level pathless validation errors (`Loc(vec![])`), violating the path-precision requirement.
- `Severity::Fatal` errors do not halt validation execution.

---

## 6. Detailed Audit Findings (Falsifier Evaluations)

### Falsifier 1: Unknown fields inside arrays of tables accepted
* **Verdict**: **True**  
* **Analysis**: Under ST-106, trusted configuration loading must reject unknown fields. The implementation of `load_admitted` calls `detect_unknown_fields` to find unknown keys.
  However, `detect_unknown_fields` delegates to `collect_unknown_keys`:
  ```rust
  fn collect_unknown_keys(
      original: &Value,
      typed: &Value,
      prefix: &str,
      unknown: &mut Vec<String>,
  ) {
      if let (Value::Table(orig_t), Value::Table(typed_t)) = (original, typed) {
          for (k, v) in orig_t {
              let path = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
              if let Some(typed_v) = typed_t.get(k) {
                  collect_unknown_keys(v, typed_v, &path, unknown);
              } else {
                  unknown.push(path);
              }
          }
      }
  }
  ```
  This function only recursively descends when both the original and deserialized values are of type `Value::Table`. If the configuration contains an array of tables (e.g., `[[servers]]`), both values are `Value::Array`. The `if let (Value::Table, Value::Table)` condition evaluates to `false`, and the function returns without comparing the keys of the tables inside the array. Any unrecognized field inside an array of tables is silently accepted, creating a major security and validation bypass.

### Falsifier 2: Validation error without path allowed
* **Verdict**: **True**  
* **Analysis**: The ST-107 specification requires that:
  > "Every semantic validation error MUST have a precise dotted key path locating it in the configuration tree. Errors reported without a location or with a blank path are inadmissible. Attempting to record or emit a validation error without a precise location fails validation or raises a compiling/runtime assertion."
  
  In the implementation, `Validator::error` and `Validator::record` push the error to the `errors` list using `self.loc` as is. There is no check or assertion that `!self.loc.is_root()`.
  
  Furthermore, the codebase contains a unit test [root_level_error_renders_as_root](file:///Users/sac/star-toml/src/validation.rs#L1194) explicitly asserting that pathless errors are supported and render as `(root)`:
  ```rust
  #[test]
  fn root_level_error_renders_as_root() {
      struct Thing;
      impl Validate for Thing {
          fn validate(&self, v: &mut Validator) {
              v.error(ErrorKind::Predicate { code: "always" }, "always fails");
          }
      }
      let errs = Thing.check().unwrap_err();
      assert_eq!(errs.errors()[0].loc.to_string(), "(root)");
      assert!(errs.errors()[0].loc.is_root());
  }
  ```
  This confirms that pathless validation errors are allowed by design and implementation, violating the precise path safety invariant.

### Falsifier 3: Fatal severity does not halt execution
* **Verdict**: **True**  
* **Analysis**: Under ST-107:
  > "Any check marked as Fatal (or generating an error with Severity::Fatal) must immediately halt further check execution..."
  
  In the validation runner, there is no logic to intercept fatal errors during execution. When a check is run with `Severity::Fatal`, it is recorded and pushed to the `errors` vector just like any other error. The validation closures of subsequent sibling fields and nested objects continue executing. The verifier check `fatal_error_downgraded` only checks if the `Fatal` severity is preserved in the final output list, masking the fact that the check runner failed to abort execution early.

### Falsifier 4: Fitness treated as q_config
* **Verdict**: **True**  
* **Analysis**: In `build_admitted` ([src/loader.rs:1670](file:///Users/sac/star-toml/src/loader.rs#L1670)), the loader hardcodes `validation_fitness = 1.0_f64` for all admitted configurations.
  
  Furthermore, the validation engine fails on **any** validation diagnostics, including those of severity `Advisory` or `Warning`. By failing validation when informational or minor warnings are present, the runner prevents the loading of configurations that would have `fitness < 1.0`. Thus, `fitness` is treated as a hard gate identical to the binary standing gate ($q_{config} = 1$), rather than serving as a relative metrics signal.
