# Agent 08 Audit Report: Verifier Report and Counterexamples

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | `TrustedLoader` and `PathPolicy` API definitions in `src/loader.rs` define the configuration boundaries checked. | None. |
| O | Temp configs generated dynamically in `src/bin/verifier_report.rs` (using `write_toml`) serve as raw observations. | None. |
| O* | Loaded `AdmittedConfig<T>` returned by `load_admitted` / `load_frozen` inside `verifier_report.rs` checks. | None. |
| μ | Typestate-guided loading transitions through lifecycle steps (Raw -> Merged -> Deserialized -> Validated -> Frozen). | None. |
| A | Generated output written to `VERIFIER_REPORT.md` containing the verification table. | None. |
| C | 22 counterexample checks executed in `verifier_report.rs` returning boolean statuses. | Check for `ocel_treated_as_standing_authority` is omitted. |
| W | `ConfigWitness` checked for deterministic hash values in `verifier_report.rs`. | Missing verification tests that actually falsify the witness (e.g. check that witness fails to generate/changes when fields or sources are tampered). |
| q | `verifier_report.rs` computes and prints total/passed/failed counts, exiting with non-zero code if any fail. | Standing bit computation is static for stubs (like comment-preservation claim check). |

---

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | Checked in `verifier_report.rs` when loading valid TOML configs (e.g. `parse_valid_treated_as_trusted`, `witness_nondeterministic`). | None. | Conformance verification. |
| Falsification | Checked via rejection of bad inputs (e.g. `missing_required_file_not_error`, `unknown_field_accepted_in_trusted_mode`, `validation_not_run`, `path_traversal_accepted`, `null_byte_path_accepted`, `rewrite_without_validation`). | None. | Mitigates failure scenarios. |
| Counterfactual | Modifying prefixes/paths to show that incorrect configs fail (e.g. `env_override_without_prefix`, `env_override_not_reported`). | No counterfactual validation of witness hashing (i.e. modifying validation report or source to verify that witness hash strictly shifts). | Critical for causal verification. |
| Invariant | Checked via `validation_error_without_path` ensuring location is never root, and `fatal_error_downgraded` checking fatal error preservation. | None. | Structural integrity guarantee. |
| Metamorphic | Metamorphic ordering check implicitly validated via `ambiguous_layer_order`. | No explicit metamorphic stability check inside the verifier itself for whitespace/ordering. | Semantic stability. |
| Boundary | Checked via port range validation (`validation_not_run` checks that out-of-range port like `80` is rejected). | None. | Prevents boundary drift. |
| Conservation | Checked via `unreported_layer_override` ensuring winning layers are not lost. | None. | Complete traceability. |
| Determinism | `nondeterministic_save` checks that saving a config twice yields the same output witness. | None. | Replayability foundation. |
| Idempotence | Tested in `tests/brce.rs` but not in `verifier_report.rs`. | The verifier binary itself does not execute an idempotence check for `save_canonical`. | Structural stability. |
| Replay | `witness_nondeterministic` checks that witness hashes match when loaded from identical environments. | No active test in verifier verifying witness reconstruction/replay from raw parts. | Prevents verification bypass. |
| Provenance | `implicit_source_used` and `env_override_not_reported` verify source and environment tracing. | None. | Complete auditability. |
| Witness | `witness_missing_source_digest`, `witness_missing_env_report`, and `witness_missing_validation_report` verify presence of reports. | Active negative checks (falsification tests) that prove witness generation fails if components are omitted. | Witness integrity. |

---

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `parse_valid_treated_as_trusted` | Yes (`verifier_report.rs:72`) | Yes (`tests/brce.rs:78`) | Yes | Mitigated |
| `implicit_source_used` | Yes (`verifier_report.rs:79`) | Yes (`tests/brce.rs:113`) | Yes | Mitigated |
| `missing_required_file_not_error` | Yes (`verifier_report.rs:91`) | Yes (`tests/brce.rs:137`) | Yes | Mitigated |
| `ambiguous_layer_order` | Yes (`verifier_report.rs:99`) | Yes (`tests/brce.rs:567`) | Yes | Mitigated |
| `unreported_layer_override` | Yes (`verifier_report.rs:114`) | Yes (`tests/brce.rs:374`) | Yes | Mitigated |
| `env_override_without_prefix` | Yes (`verifier_report.rs:128`) | Yes (`tests/brce.rs:242`) | Yes | Mitigated |
| `env_override_not_reported` | Yes (`verifier_report.rs:142`) | Yes (`tests/brce.rs:187`) | Yes | Mitigated |
| `unknown_field_accepted_in_trusted_mode` | Yes (`verifier_report.rs:157`) | Yes (`tests/brce.rs:271`) | Yes | Mitigated |
| `validation_not_run` | Yes (`verifier_report.rs:168`) | Yes (`tests/brce.rs:78`) | Yes | Mitigated |
| `validation_error_without_path` | Yes (`verifier_report.rs:179`) | Yes (`tests/brce.rs:414`) | Yes | Mitigated |
| `fatal_error_downgraded` | Yes (`verifier_report.rs:194`) | Yes (`tests/validation.rs`) | Yes | Mitigated |
| `path_traversal_accepted` | Yes (`verifier_report.rs:205`) | Yes (`tests/brce.rs:309`) | Yes | Mitigated |
| `null_byte_path_accepted` | Yes (`verifier_report.rs:212`) | Yes (`tests/brce.rs:327`) | Yes | Mitigated |
| `source_relative_path_unresolved` | Yes (`verifier_report.rs:219`) | Yes (`tests/brce.rs:356`) | Yes | Mitigated |
| `nondeterministic_save` | Yes (`verifier_report.rs:230`) | Yes (`tests/brce.rs:462`) | Yes | Mitigated |
| `comment_preservation_claim_unproven` | No active check. Stubbed to return `true` (`verifier_report.rs:245`) | No | No, returns true statically | Static Stub (Unverified) |
| `rewrite_without_validation` | Yes (`verifier_report.rs:253`) | Yes (doc-tests) | Yes | Mitigated |
| `witness_missing_source_digest` | Yes (`verifier_report.rs:267`) | Yes (`tests/brce.rs:114`) | Yes | Mitigated |
| `witness_missing_env_report` | Yes (`verifier_report.rs:278`) | Yes (`tests/brce.rs:187`) | Yes | Mitigated |
| `witness_missing_validation_report` | Yes, but weak assertion (`verifier_report.rs:292` checks only `!hash.is_empty()`) | Yes | No (only checked for non-emptiness) | Weak Check |
| `witness_nondeterministic` | Yes (`verifier_report.rs:304`) | Yes (`tests/brce.rs:608`) | Yes | Mitigated |
| `downstream_policy_inside_star_toml` | Yes (`verifier_report.rs:324`) | No | Yes | Mitigated |
| `ocel_treated_as_standing_authority` | No (`verifier_report.rs` omits this check) | No | No | Missing |

---

### Standing Decision
**PARTIAL_ALIVE**

*Rationale:*
- The 23rd counterexample detector (`ocel_treated_as_standing_authority`) defined in the detectors ontology (`docs/ontology/star-toml.detectors.ttl`) is completely missing from the verifier checks.
- Several counterexamples are stubs (`comment_preservation_claim_unproven` statically returns `true` without actual checks) or weakly verified (`witness_missing_validation_report` just checks that the hash is not empty, rather than confirming that omitting validation results actually changes/fails the witness).
- The verifier report binary itself is not integrated into `cargo test`, meaning it is not run during normal testing pipelines, creating a drift threat.

---

## Detailed Audit Findings (Falsifier Evaluations)

### Falsifier 1: Verifier only writes prose but does not execute checks
* **Verdict**: **Partially False / Partially True**
* **Analysis**: The verifier executes actual Rust code checks for 21 out of the 22 reported counterexamples. It constructs temporary configuration files, invokes the `TrustedLoader`, applies environment overrides, and asserts the correctness of error conditions and outputs. However, for `comment_preservation_claim_unproven`, it returns `true` statically without running any verification logic. This static bypass qualifies as a prose-only assertion.

### Falsifier 2: Verifier exits success when failset > 0
* **Verdict**: **False**
* **Analysis**: In `src/bin/verifier_report.rs`, the exit code logic in `main` is implemented as:
  ```rust
  if failed > 0 {
      eprintln!("{failed} counterexample(s) still active");
      std::process::exit(1);
  }
  ```
  If any check in the run returns `passed = false`, the count of `failed` is incremented, and the binary exits with code `1`.

### Falsifier 3: 22 counterexamples not actually represented
* **Verdict**: **True**
* **Analysis**: While the verifier report generates a list of 22 checks, the official detectors ontology (`docs/ontology/star-toml.detectors.ttl`) specifies **23** detectors under `<concept/CounterexampleSet>`. The 23rd detector, `ocel_treated_as_standing_authority`, is completely omitted from the verifier binary. Furthermore, the check for `comment_preservation_claim_unproven` is a static stub returning `true`, and the check for `witness_missing_validation_report` only validates that the witness hash is not empty rather than asserting that the witness changes or fails in the absence of a validation report.

### Falsifier 4: Counterexamples not connected to tests
* **Verdict**: **True**
* **Analysis**: There is no link between the main cargo test suite (`cargo test`) and the `verifier_report` binary. The tests under `tests/` do not invoke or depend on the binary, and the binary is not run as part of the normal test suite. If an implementation change breaks a counterexample detector in the verifier, `cargo test` will still pass.

### Falsifier 5: OCEL treated as standing authority
* **Verdict**: **False**
* **Analysis**: The OCEL export adapter is compiled as a no-op stub unless the optional `wasm4pm-compat` feature is enabled. When enabled, it converts admission events into an OCEL event log for history export, but does not read from or mutate the core configuration status or the cryptographic witness computation. However, because the check for `ocel_treated_as_standing_authority` is missing from the verifier, there is no active verification safeguard ensuring this invariant holds.

### Falsifier 6: Downstream policy inside star-toml
* **Verdict**: **False**
* **Analysis**: `star-toml` defines generic bounds (such as traversal checks and path policy rules) but does not hardcode downstream application-specific or organizational policies inside the library.
