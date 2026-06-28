# star-toml v26.6.28 Finish Audit

## Final Classification

PARTIAL_ALIVE

## Agent Verdict Matrix

| Agent | Area | Classification | Blocking Findings |
|---|---|---|---|
| Agent 01 | Repository, Tag, Test Baseline | ALIVE | None. Working tree is clean of dirty tracked files, tests pass, tag is recorded. |
| Agent 02 | Typestate and Admission | ALIVE | None. Typestate structures correctly enforce valid pipeline transitions. |
| Agent 03 | Source, Layer, Env Reports | ALIVE | None. Source digests, winner mapping, and env prefix policies are correctly captured. |
| Agent 04 | Validation and Error Topology | PARTIAL_ALIVE | Non-strict load modes accept unknown fields; strict mode unknown-field errors emit root `Loc(vec![])` (no path-precision); `Severity::Fatal` does not stop subsequent checks; validation fails entirely on minor advisory warnings. |
| Agent 05 | Path Policy and PathWitness | PARTIAL_ALIVE | Windows separator bypasses (`foo\\..\\bar`) on Unix; lack of symlink canonicalization; CWD-variance on relative config sources; absolute path bypass under `RelativeOnly` policy; `PathWitness` lacks `rejection_code` and sandbox `root`. |
| Agent 06 | Witness and q_config | ALIVE | None. Cryptographic witness is computed deterministically using BLAKE3. |
| Agent 07 | OCEL and wasm4pm-compat Boundary | ALIVE | None. Optional dependency on `wasm4pm-compat` is properly gated; no direct dependency on `wasm4pm`. |
| Agent 08 | Verifier Report and Counterexamples | PARTIAL_ALIVE | Check #16 is a hardcoded stub returning `true`; 23rd ontology detector check (`ocel_treated_as_standing_authority`) is missing; verifier binary is not run under `cargo test`. |
| Agent 09 | Docs, Jira, Examples Alignment | PARTIAL_ALIVE | Ontological schema claims `AdmittedConfig` is deferred; `load()` returns `TrustedConfig` instead of `AdmittedConfig`; examples use raw `from_str` parser bypasses. |

## Release Claim

Is `v26.6.28` finished?

**No.** While the core parser and merge pipeline are complete, `v26.6.28` is not fully finished and cannot be promoted to production due to critical path validation escapes, verifier completeness gaps, and validation metadata discrepancies.

## Evidence Summary

* **158 unit/BRCE/adversarial tests** pass on `cargo test`.
* **4 additional OCEL tests** pass under `--features wasm4pm-compat`.
* The `verifier_report` binary runs successfully showing 22/22 passes, but auditing revealed stubbed checks and a missing 23rd ontology check.

## Boundary Summary

- **wasm4pm-compat**: Optional dependency correctly gated behind the `wasm4pm-compat` feature gate.
- **wasm4pm direct dependency**: Absent. No circular dependency risk.
- **OCEL**: One-way export implemented via `export_events_to_ocel`. It does not calculate standing.
- **q_config**: Conceptually enforced by typestates. It does not leak runtime mutable authority.
- **ConfigWitness**: Cryptographic witness correctly hashes sorted sources, layers, env overrides, validation fitness, and canonical output.
- **AdmittedConfig<T>**: Enforces computation of `ConfigWitness` before instantiation.
- **save_canonical**: Typestate-fenced compile-time restriction works perfectly.
- **verifier_report**: Runs to completion but has check stubs and lacks coverage for the 23rd detector check.

## Counterexample Standing

| Counterexample | Status | Evidence |
|---|---|---|
| `parse_valid_treated_as_trusted` | PASS | Verified in verifier check 1. |
| `implicit_source_used` | PASS | Verified in verifier check 2. |
| `missing_required_file_not_error` | PASS | Verified in verifier check 3. |
| `ambiguous_layer_order` | PASS | Verified in verifier check 4. |
| `unreported_layer_override` | PASS | Verified in verifier check 5. |
| `env_override_without_prefix` | PASS | Verified in verifier check 6. |
| `env_override_not_reported` | PASS | Verified in verifier check 7. |
| `unknown_field_accepted_in_trusted_mode` | FAIL | Accepted by default loader in non-strict mode. |
| `validation_not_run` | PASS | Verified in verifier check 9. |
| `validation_error_without_path` | FAIL | Strict mode unknown key errors are emitted at root `Loc(vec![])`. |
| `fatal_error_downgraded` | PASS | Verified in verifier check 11. |
| `path_traversal_accepted` | FAIL | Bypassed by Windows directory separators (`\\..\\`) on Unix; symlink sandbox escape. |
| `null_byte_path_accepted` | PASS | Verified in verifier check 13. |
| `source_relative_path_unresolved` | FAIL | Relative config source path breaks CWD-invariance. |
| `nondeterministic_save` | PASS | Verified in verifier check 15. |
| `comment_preservation_claim_unproven` | PASS | Stubbed (acknowledged limitation). |
| `rewrite_without_validation` | PASS | Verified in verifier check 17. |
| `witness_missing_source_digest` | PASS | Verified in verifier check 18. |
| `witness_missing_env_report` | PASS | Verified in verifier check 19. |
| `witness_missing_validation_report` | PASS | Verified in verifier check 20. |
| `witness_nondeterministic` | PASS | Verified in verifier check 21. |
| `downstream_policy_inside_star_toml` | PASS | Verified in verifier check 22. |
| `ocel_treated_as_standing_authority` | FAIL | Completely missing from verifier binary checks. |

## Remaining Risks

1. **Path Validation Escape**: Users can bypass sandbox constraints on Unix using Windows-style separators (`\\`), or using symlinks due to lack of `canonicalize()` checks.
2. **CWD-Variance**: Relative paths inside configurations resolve dynamically based on the current process CWD rather than the configuration file's parent directory.
3. **Verifier Drift**: Lack of integration between `verifier_report` and `cargo test` creates a risk of drift during future code updates.
4. **Validation metadata**: Conflation of warnings/advisories with the terminal validation gating logic.

## Final Recommendation

**RETAG_AFTER_PATCH**

We must patch the path validation traversal escapes, add the 23rd ontology detector check to `verifier_report.rs`, and correct the documentation discrepancies before re-tagging `v26.6.28`.
