# Audit Report: Repository, Tag, and Test Baseline (v26.6.28)

**Audit Date:** June 28, 2026  
**Auditor:** Agent 01  
**Target Repository:** `star-toml`  
**Target Tag:** `v26.6.28`  
**Local System Time:** 2026-06-27T21:31:50-07:00  

---

## Required Command Executions and Evidence

Below are the exact execution logs of the nine required commands performed in the workspace.

### 1. `git status --short`
```
(empty - no tracked files are modified)
```
*Note: Git status is completely clean of any changes to tracked files in the source tree (including `src/`, `tests/`, and `Cargo.toml`). Only untracked build cache files exist under the ignored `target/` directory.*

### 2. `git log --oneline -8`
```
eea7e04 docs: add v26.6.28 admission receipt
ba0e8a2 chore: remove stale "deferred" doc comments now that AdmittedConfig/ConfigWitness exist
aab0113 feat: v26.6.28 — ST-102/106/108/109/111 admission substrate complete
800cb39 feat: WP-1/WP-2/WP-3 — typestate pipeline, provenance reports, traced merge
09714e9 chore: exclude agent scratch files and docs from crates.io package
1438c3f fix: add required metadata to star-toml-derive for crates.io publishing
e39c2ca feat: complete v26.6.27 implementation with typestate lifecycle, derive macro, and hardening
ad81ec7 feat: initial commit of standalone star-toml repository under praxis house style
```

### 3. `git tag --list "v26.6.28"`
```
v26.6.28
```

### 4. `git rev-parse HEAD`
```
eea7e04c88e3c57225134894c49bc5d22f19a7a1
```

### 5. `git rev-parse v26.6.28`
```
48adcc6c3cfb71be2382182fc46c433bbb2b8bb6
```
*Verification Detail:* The annotated Git tag `v26.6.28` points to the tag object `48adcc6c3cfb71be2382182fc46c433bbb2b8bb6`. Resolving the tag target via `git cat-file -p v26.6.28` reveals:
```
object ba0e8a238e097c76238a0650b7a34e559ac97481
type commit
tag v26.6.28
tagger Sean Chatman <136349053+seanchatmangpt@users.noreply.github.com> 1782620184 -0700

star-toml v26.6.28 — admission substrate admitted
```
The tag points directly to commit `ba0e8a238e097c76238a0650b7a34e559ac97481` (represented as `ba0e8a2`). HEAD is at commit `eea7e04c88e3c57225134894c49bc5d22f19a7a1` (`eea7e04`), which is exactly 1 commit ahead of the tag commit. The diff between them contains solely the addition of `STAR_TOML_V26_6_28_ADMISSION_RECEIPT.md` (as documented in `git diff ba0e8a238e097c76238a0650b7a34e559ac97481 HEAD`). The library source code is completely identical between the tagged commit and HEAD.

### 6. `cargo test`
```
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.29s
```
*Note: A total of 158 tests passed successfully, including 82 unit tests, 10 adversarial checks, 31 BRCE ladder tests, 4 macro validations, and 31 documentation tests.*

### 7. `cargo test --features wasm4pm-compat`
```
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.16s
```
*Note: A total of 162 tests passed successfully. The additional 4 tests are in `tests/ocel_export.rs`, validating features activated by the `wasm4pm-compat` dependency.*

### 8. `cargo metadata --format-version=1 --no-deps`
```json
{"packages":[{"name":"star-toml","version":"26.6.28","id":"path+file:///Users/sac/star-toml#26.6.28","license":"MIT","license_file":null,"description":"Framework for loading, layering, and validating any *.toml configuration file","source":null,"dependencies":[{"name":"blake3","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^1","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null,"registry":null},{"name":"serde","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^1.0","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":["derive"],"target":null,"registry":null},{"name":"star-toml-derive","source":null,"req":"^26.6.28","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null,"registry":null,"path":"/Users/sac/star-toml/star-toml-derive"},{"name":"thiserror","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^2.0","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null,"registry":null},{"name":"toml","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^1.1.0","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":["preserve_order"],"target":null,"registry":null},{"name":"wasm4pm-compat","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^26.6","kind":null,"rename":null,"optional":true,"uses_default_features":true,"features":[],"target":null,"registry":null},{"name":"serde","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^1.0","kind":"dev","rename":null,"optional":false,"uses_default_features":true,"features":["derive"],"target":null,"registry":null},{"name":"tempfile","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^3","kind":"dev","rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null,"registry":null}],"targets":[{"kind":["lib"],"crate_types":["lib"],"name":"star_toml","src_path":"/Users/sac/star-toml/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},{"kind":["bin"],"crate_types":["bin"],"name":"verifier_report","src_path":"/Users/sac/star-toml/src/bin/verifier_report.rs","edition":"2021","doc":true,"doctest":false,"test":true},{"kind":["example"],"crate_types":["bin"],"name":"validate","src_path":"/Users/sac/star-toml/examples/validate.rs","edition":"2021","doc":false,"doctest":false,"test":false},{"kind":["test"],"crate_types":["bin"],"name":"adversarial","src_path":"/Users/sac/star-toml/tests/adversarial.rs","edition":"2021","doc":false,"doctest":false,"test":true},{"kind":["test"],"crate_types":["bin"],"name":"brce","src_path":"/Users/sac/star-toml/tests/brce.rs","edition":"2021","doc":false,"doctest":false,"test":true},{"kind":["test"],"crate_types":["bin"],"name":"e2e_tests","src_path":"/Users/sac/star-toml/tests/e2e_tests.rs","edition":"2021","doc":false,"doctest":false,"test":true},{"kind":["test"],"crate_types":["bin"],"name":"ocel_export","src_path":"/Users/sac/star-toml/tests/ocel_export.rs","edition":"2021","doc":false,"doctest":false,"test":true},{"kind":["test"],"crate_types":["bin"],"name":"validation_macros","src_path":"/Users/sac/star-toml/tests/validation_macros.rs","edition":"2021","doc":false,"doctest":false,"test":true}],"features":{"e2e_tests":[],"wasm4pm-compat":["dep:wasm4pm-compat"]},"manifest_path":"/Users/sac/star-toml/Cargo.toml","metadata":null,"publish":null,"authors":["Sean Chatman <sean@chatmangpt.com>"],"categories":["development-tools","config"],"keywords":["config","toml","layered","validation"],"readme":"README.md","repository":"https://github.com/seanchatmangpt/star-toml","homepage":"https://github.com/seanchatmangpt/star-toml","documentation":null,"edition":"2021","links":null,"default_run":null,"rust_version":"1.82"},{"name":"star-toml-derive","version":"26.6.28","id":"path+file:///Users/sac/star-toml/star-toml-derive#26.6.28","license":"MIT","license_file":null,"description":"Procedural macro derive crate for star-toml validation","source":null,"dependencies":[{"name":"quote","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^1.0","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null,"registry":null},{"name":"syn","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^2.0","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":["full"],"target":null,"registry":null}],"targets":[{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"star_toml_derive","src_path":"/Users/sac/star-toml/star-toml-derive/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true}],"features":{},"manifest_path":"/Users/sac/star-toml/star-toml-derive/Cargo.toml","metadata":null,"publish":null,"authors":["Sean Chatman <sean@chatmangpt.com>"],"categories":[],"keywords":[],"readme":null,"repository":"https://github.com/seanchatmangpt/star-toml","homepage":null,"documentation":null,"edition":"2021","links":null,"default_run":null,"rust_version":"1.82"}],"workspace_members":["path+file:///Users/sac/star-toml#26.6.28","path+file:///Users/sac/star-toml/star-toml-derive#26.6.28"],"workspace_default_members":["path+file:///Users/sac/star-toml#26.6.28"],"resolve":null,"target_directory":"/Users/sac/star-toml/target","build_directory":"/Users/sac/star-toml/target","version":1,"workspace_root":"/Users/sac/star-toml","metadata":null}
```

### 9. `cargo tree -i wasm4pm-compat`
```
wasm4pm-compat v26.6.28
└── star-toml v26.6.28 (/Users/sac/star-toml)
```

---

## BRCE Standing Analysis

### Admissibility Tuple

| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | Declared configuration admission boundary properties: explicit configuration sources, merge layers order validation (`Defaults < Files < Env`), environment variable policy schema definition, path safety check policies, and witness requirements. | None |
| **O** | Raw TOML file inputs, default fallback values, and environment variable override mappings captured during pipeline loading. | None |
| **O\*** | Implemented as `AdmittedConfig<T>` container, ensuring that configurations are admitted only after a cryptographic validation witness has been verified. | None |
| **μ** | Transform functions including recursive table-merging rules, environment coercion to primitives, and alphabetical-sorted canonical serialization (`save_canonical`). | None |
| **A** | Created output artifacts: `ConfigWitness`, `VERIFIER_REPORT.md` and `STAR_TOML_V26_6_28_ADMISSION_RECEIPT.md`. | None |
| **C** | The 22 distinct counterexample detectors in `src/bin/verifier_report.rs` validating mathematical limits and security boundaries. | None |
| **W** | Cryptographic admission witness (`ConfigWitness`) generated via BLAKE3 hashes of input files, configuration layers order, environment overrides, validation results, and canonical outputs. | None |
| **q** | Standing score indicator. Value matches `q_config = 1` since all baseline tests, adversarial checks, and the verifier report pass without errors (`failset_cardinality = 0`). | None |

---

### Evidence Categories

| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | `test_load_admitted_succeeds`, `test_load_frozen_succeeds_with_valid_config` (positive validation of valid configurations) | None | Core baseline verification of valid configuration paths. |
| **Falsification** | `test_forbidden_path_fails`, `test_null_byte_fails` (adversarial rejection of traversal inputs) | None | Essential security proof preventing inadmissible configuration states. |
| **Counterfactual** | `test_witness_changes_on_source_change` (altering environment keys or source files updates the witness digest) | None | Verifies causal sensitivity of the cryptographic digest to input perturbations. |
| **Invariant** | `ValidationErrors` structure containing location, error code, severity level, message, and repair hints | None | Guarantees standard, parseable validation failure metadata. |
| **Metamorphic** | `test_brce_metamorphic_canonical_stability` (changing non-semantic whitespace or table sorting preserves canonical hash) | None | Proves mathematical stability of configuration content representation. |
| **Boundary** | Built-in domain/IP checker boundaries, port range boundary validator checks | None | Ensures strict enforcement of numeric and string limits. |
| **Conservation** | `test_every_final_field_has_winning_layer` (tracing winner maps back to individual layer source) | None | Guarantees complete tracking of field provenance. |
| **Determinism** | `test_witness_is_deterministic`, `test_layer_order_digest_is_deterministic` | None | Ensures reproducible configuration loading results. |
| **Idempotence** | `test_brce_idempotence_canonical` (`save_canonical` stabilization) | None | Guarantees stability over repeated serialization cycles. |
| **Replay** | `ConfigWitness` derivation logic verifying inputs, environment configuration, validation errors, and output canonical TOML | None | Allows external deciders to verify the legitimacy of configuration admission. |
| **Provenance** | `test_env_override_report_records_prefix_mapping` | None | Fully audits environment variable adjustments to final config fields. |
| **Witness** | `test_path_witness_emitted` | None | Confirms receipt issuance on admission. |

---

### Failset

All 22 counterexample detectors from `VERIFIER_REPORT.md` pass verification. The cardinality is exactly 0:

| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|:---:|:---:|:---:|:---:|
| `1. parse_valid_treated_as_trusted` | Yes | Yes | Yes | **PASS** |
| `2. implicit_source_used` | Yes | Yes | Yes | **PASS** |
| `3. missing_required_file_not_error` | Yes | Yes | Yes | **PASS** |
| `4. ambiguous_layer_order` | Yes | Yes | Yes | **PASS** |
| `5. unreported_layer_override` | Yes | Yes | Yes | **PASS** |
| `6. env_override_without_prefix` | Yes | Yes | Yes | **PASS** |
| `7. env_override_not_reported` | Yes | Yes | Yes | **PASS** |
| `8. unknown_field_accepted_in_trusted_mode` | Yes | Yes | Yes | **PASS** |
| `9. validation_not_run` | Yes | Yes | Yes | **PASS** |
| `10. validation_error_without_path` | Yes | Yes | Yes | **PASS** |
| `11. fatal_error_downgraded` | Yes | Yes | Yes | **PASS** |
| `12. path_traversal_accepted` | Yes | Yes | Yes | **PASS** |
| `13. null_byte_path_accepted` | Yes | Yes | Yes | **PASS** |
| `14. source_relative_path_unresolved` | Yes | Yes | Yes | **PASS** |
| `15. nondeterministic_save` | Yes | Yes | Yes | **PASS** |
| `16. comment_preservation_claim_unproven` | Yes | Yes | Yes | **PASS** |
| `17. rewrite_without_validation` | Yes | Yes | Yes | **PASS** |
| `18. witness_missing_source_digest` | Yes | Yes | Yes | **PASS** |
| `19. witness_missing_env_report` | Yes | Yes | Yes | **PASS** |
| `20. witness_missing_validation_report` | Yes | Yes | Yes | **PASS** |
| `21. witness_nondeterministic` | Yes | Yes | Yes | **PASS** |
| `22. downstream_policy_inside_star_toml` | Yes | Yes | Yes | **PASS** |

---

### Standing Decision

**ALIVE**: $q$ computation is bounded, witnessed, replayable, and failset-zero for this surface.
