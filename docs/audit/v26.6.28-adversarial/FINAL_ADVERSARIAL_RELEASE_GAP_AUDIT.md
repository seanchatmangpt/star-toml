# star-toml v26.6.28 Adversarial Release Gap Audit

## Final Release Classification

DO_NOT_RELEASE

## Agent Verdict Matrix

| Agent | Area | Area Classification | Highest Severity | Blocking Findings |
|---|---|---|---|---|
| Agent 01 | Release Packaging, Tag, Crates.io Readiness | PARTIAL_ALIVE | RELEASE_BLOCKER | Build fails under `wasm4pm-compat` feature; crates.io packaging is broken due to missing `.gitignore` including all compiler caches. |
| Agent 02 | Typestate Bypass and Admission Forgery | PARTIAL_ALIVE | RELEASE_BLOCKER | All typestate wrappers (Raw, Merged, etc.) and `AdmittedConfig` / `ConfigWitness` have public fields allowing external forgery and bypasses. |
| Agent 03 | Witness and q_config Correctness | ALIVE | None | None. Witness values and q_config pipeline validation invariants are structurally correct. |
| Agent 04 | Path Security, Traversal, PathWitness | PARTIAL_ALIVE | RELEASE_BLOCKER | Windows separators bypass directory checks on Unix (`foo\\..\\bar`); lexical-only clean_path leaves symlinks vulnerable; BlockForbidden directory prefix match can be bypassed. |
| Agent 05 | Validation, Unknown Fields, Error Topology | PARTIAL_ALIVE | RELEASE_BLOCKER | Arrays of tables bypass unknown-field detection entirely; unknown keys trigger pathless root `Loc(vec![])` errors; `Severity::Fatal` does not stop check execution. |
| Agent 06 | OCEL and wasm4pm-compat Boundary | BUILD_BROKEN | RELEASE_BLOCKER | Binary target `verifier_report.rs` fails to compile under `wasm4pm-compat` feature due to type mismatch (E0308). |
| Agent 07 | Verifier Integrity and Counterexamples | ALIVE | None | None. All 23 counterexamples are run and validated by integrated tests. |
| Agent 08 | Docs, Examples, Release Claims | PARTIAL_ALIVE | PATCH_BEFORE_RELEASE | Mismatches in ontologies (claims states are deferred when they exist); `validate.rs` example bypasses the admission pipeline using raw parses. |
| Agent 09 | SemVer/API/Compatibility/Feature Matrix | BUILD_BROKEN | RELEASE_BLOCKER | Non-additive signature change under feature flags (`export_events_to_ocel` changes return type between `()` and `OcelLog`). |

## Release Blockers

| Finding | Evidence | Required Action |
|---|---|---|
| Mismatched return types in `export_events_to_ocel` | `src/ocel.rs` returns `()` without feature and `OcelLog` with feature. `verifier_report.rs:368` fails to compile under feature flag. | Align signatures of `export_events_to_ocel` to always return `OcelLog` (with dummy stub if feature is disabled). |
| Unknown Field Bypass in Array of Tables | `collect_unknown_keys` in `src/loader.rs` only traverses tables, ignoring arrays of tables. | Traverse arrays of tables to check nested elements for unknown fields. |
| Windows separator directory traversal bypass | `resolve_and_validate` cleans backslashes but some paths can bypass. | Fully canonicalize/normalise path separators. |
| Typestate Admission Bypasses | Public fields on `Validated`, `Frozen`, `AdmittedConfig`, and `ConfigWitness`. | Hide internal representation fields, making them constructible only through pipeline. |

## Patch-Before-Release Findings

| Finding | Evidence | Required Action |
|---|---|---|
| Broken crates.io packaging | Missing `.gitignore` causes intermediate target/ directories to be packaged. | Add a proper `.gitignore` file. |
| Root-level validation errors | Unknown field validation error uses empty path segments `Loc(vec![])`. | Enforce that all errors contain path-precise segments. |

## Documentation / Release Note Findings

| Finding | Evidence | Required Action |
|---|---|---|
| Receipt documentation contradictions | `STAR_TOML_V26_6_28_ADMISSION_RECEIPT.md` claims OCEL has authority to grant standing. | Clarify that OCEL is process history log only, not standing authority. |

## Non-Blocking Risks

| Finding | Evidence | Suggested Follow-Up |
|---|---|---|
| Symlink Sandbox Escape | Pure lexical clean_path ignores symlink resolution. | Document as known limitation and recommend filesystem-level sandboxing. |

## Boundary Confirmation

| Boundary                   | Required State                       | Status | Evidence |
| -------------------------- | ------------------------------------ | ------ | -------- |
| star-toml → wasm4pm-compat | Required, version `"26.6"`           | PASS | Cargo.toml dependency declaration. |
| star-toml → wasm4pm        | Forbidden                            | PASS | Confirmed absent from tree. |
| wasm4pm → star-toml        | Required downstream direction        | PASS | Verified via architecture specifications. |
| OCEL export                | Uses wasm4pm-compat types            | FAIL | Under feature flag, it does; signature mismatches without feature flag. |
| OCEL authority             | Lifecycle history only, not q_config | PASS | Exporter does not compute q_config or construct AdmittedConfig. |
| AdmittedConfig<T>          | Witness-backed admission only        | FAIL | Manual construction possible due to public fields. |
| q_config                   | Witness completeness + failset zero  | PASS | Enforced through pipeline. |

## Counterexample Standing

| Counterexample | Status | Evidence |
|---|---|---|
| `parse_valid_treated_as_trusted` | PASS | Tested. |
| `implicit_source_used` | PASS | Tested. |
| `missing_required_file_not_error` | PASS | Tested. |
| `ambiguous_layer_order` | PASS | Tested. |
| `unreported_layer_override` | PASS | Tested. |
| `env_override_without_prefix` | PASS | Tested. |
| `env_override_not_reported` | PASS | Tested. |
| `unknown_field_accepted_in_trusted_mode` | FAIL | Bypassed by default loader in non-strict mode. |
| `validation_not_run` | PASS | Tested. |
| `validation_error_without_path` | FAIL | Strict mode unknown key errors are emitted at root. |
| `fatal_error_downgraded` | PASS | Tested. |
| `path_traversal_accepted` | FAIL | Traversal bypass on Windows separators. |
| `null_byte_path_accepted` | PASS | Tested. |
| `source_relative_path_unresolved` | FAIL | Relative configs break CWD-invariance. |
| `nondeterministic_save` | PASS | Tested. |
| `comment_preservation_claim_unproven` | PASS | Acknowledged. |
| `rewrite_without_validation` | PASS | Tested. |
| `witness_missing_source_digest` | PASS | Tested. |
| `witness_missing_env_report` | PASS | Tested. |
| `witness_missing_validation_report` | PASS | Tested. |
| `witness_nondeterministic` | PASS | Tested. |
| `downstream_policy_inside_star_toml` | PASS | Tested. |
| `ocel_treated_as_standing_authority` | FAIL | Omitted from verifier bin checks under feature compiler failure. |

## Final Recommendation

DO_NOT_RELEASE

## Required Next Commands

```bash
# Correct type mismatch signature in src/ocel.rs to always return OcelLog (with stub if no-feature)
# Fix collect_unknown_keys to traverse arrays of tables
# Hide fields of typestate structs by removing public access
# Exclude target/ directories and add .gitignore
```
