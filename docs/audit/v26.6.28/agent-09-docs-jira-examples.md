# BRCE Standing Audit Report: Docs, Jira, Examples Alignment
**Agent ID**: Agent 09
**Target File**: `docs/audit/v26.6.28/agent-09-docs-jira-examples.md`
**Local Time**: 2026-06-27T21:32:00-07:00

---

## Bound Index
- B surface inspected: `B_config` bounding envelope definitions (Sources, LayerOrder, EnvPolicy, TypeSchema, ValidationRules, PathPolicy, RewritePolicy, WitnessPolicy).
- O surface inspected: Raw observation data inputs (`BROKEN_CONFIG` in `examples/validate.rs`, and test cases in `tests/e2e_tests.rs` and `tests/brce.rs`).
- μ surface inspected: Transformations (`Loader::load`, `Loader::load_validated`, `TrustedLoader::load`, `TrustedLoader::load_admitted`, `TrustedLoader::load_admitted_strict`, `clean_path`, `resolve_and_validate`, `from_str`).
- C detectors inspected: The 22 counterexample detectors listed on the JIRA board/Verifier Report.
- W witnesses inspected: `ConfigWitness` hashing inputs, `SourceReport`, `LayerReport`, `EnvOverrideReport`, `WinnerMap`, and `PathWitness`.
- q evidence found: `q_config` evaluation, unit/doc tests, verifier reports, and integration test coverage (`tests/brce.rs`).

---

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | `B_config` declared in `docs/jira/v26.6.28/README.md` and implemented across `src/loader.rs`, `src/path.rs`, `src/validation.rs`. | None. |
| O | Raw TOML, config sources, environmental settings, and schemas tested in `tests/brce.rs` and `tests/adversarial.rs`. | None. |
| O* | Bounded admission logic in `TrustedLoader::load_admitted` and `load_admitted_strict`. | In `examples/validate.rs`, the admitted phase `O*` is bypassed using `from_str`. |
| μ | `merge_layers`, `deserialize`, `validate`, `freeze`, and `save_canonical` implemented in `src/loader.rs` and `src/path.rs`. | None. |
| A | `AdmittedConfig<T>`, `ConfigWitness`, `PathWitness` structures generated on successful validation. | The docs claim `AdmittedConfig` exposes the inner type wrapped in `Config<Frozen<T>>`, but the implementation exposes `T` directly via `pub value: T`. |
| C | 22 counterexample checks verified by `cargo run --bin verifier_report` generating `VERIFIER_REPORT.md` with `failset_cardinality = 0`. | None. |
| W | `ConfigWitness` and `PathWitness` populated and verified in tests. | None. |
| q | Bounded admissibility flag `q_config` check in verifier report and test suite. | None. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | `test_load_admitted_succeeds` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L543) verifies successful load. | None. | Shows basic capabilities exist. |
| Falsification | `test_missing_required_file_fails` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L85) ensures missing files halt load. | None. | Validates correct rejection of invalid configurations. |
| Counterfactual | `test_witness_changes_on_source_change` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L450) checks causal changes. | None. | Confirms environment/source updates propagate to witness. |
| Invariant | `ValidationError` tracks `loc`, `code`, `severity`, etc., in [validation.rs](file:///Users/sac/star-toml/src/validation.rs#L302). | None. | Guarantees diagnostic reporting completeness. |
| Metamorphic | `test_brce_metamorphic_canonical_stability` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L120) verifies whitespace changes do not affect digest. | None. | Validates serialization stability. |
| Boundary | `test_check_path` and range tests in [validation.rs](file:///Users/sac/star-toml/src/validation.rs) and [brce.rs](file:///Users/sac/star-toml/tests/brce.rs). | None. | Verifies boundary parameters. |
| Conservation | `test_every_final_field_has_winning_layer` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L624) verifies lineage conservation. | None. | Guarantees lineage traceability. |
| Determinism | `test_witness_is_deterministic` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L429) checks witness determinism. | None. | Assures replayability. |
| Idempotence | `test_brce_idempotence_canonical` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L140) verifies convergence. | None. | Guarantees serialization idempotence. |
| Replay | Witness re-computation verified in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L405). | None. | Ensures authenticity of the state. |
| Provenance | `env_override_report` in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L543) records env mapping overrides. | None. | Guarantees override traceability. |
| Witness | `ConfigWitness` checks in [brce.rs](file:///Users/sac/star-toml/tests/brce.rs). | None. | Ensures witness integrity. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `admitted_config_deferred_falsifier` | Yes (`AdmittedConfig` in `src/loader.rs`) | Yes (`tests/brce.rs`) | Yes | FIRED |
| `ocel_standing_authority_falsifier` | Yes (`STAR_TOML_V26_6_28_ADMISSION_RECEIPT.md`) | Yes (`tests/ocel_export.rs`) | Yes | FIRED |
| `complete_safety_claim_falsifier` | Yes (`src/path.rs`) | Yes (`tests/brce.rs`) | Yes | FIRED |
| `trusted_usage_bypass_load_admitted_falsifier` | Yes (`tests/e2e_tests.rs`) | Yes (`tests/e2e_tests.rs`) | Yes | FIRED |
| `example_raw_parse_bypass_falsifier` | Yes (`examples/validate.rs`) | Yes (`examples/validate.rs`) | Yes | FIRED |

### Standing Decision
PARTIAL_ALIVE

---

## Detailed Falsification & Alignment Audit Findings

### 1. AdmittedConfig Mismatch (Deferred vs Implemented)
- **Documented Claim**: `docs/ontology/star-toml.core.ttl` line 518 asserts: `"AdmittedConfig<T>, ConfigWitness, and q_config are deferred until witness and standing layers exist."`
- **Actual Capability**: Both `AdmittedConfig<T>` (line 1537) and `ConfigWitness` (line 1425) are fully implemented and functional in `src/loader.rs`, complete with witness calculations (`ConfigWitness::compute`), strict loading options (`load_admitted_strict`), and a full suite of integration tests in `tests/brce.rs` verifying their functionality.
- **Typestate Gap**: `ST-102` requirement 3 claims that `AdmittedConfig<T>` contains `config: Config<Frozen<T>>` to prevent access to the inner type `T`. However, in the actual implementation, `AdmittedConfig<T>` has a public field `pub value: T` directly, exposing `T` without the `Config<Frozen<T>>` wrapper.

### 2. OCEL Standing Authority Mismatch
- **Documented Claim**: `STAR_TOML_V26_6_28_ADMISSION_RECEIPT.md` line 66 claims that `OCEL` under `Authority` column has a function to `Grant standing`.
- **Actual Capability**: OCEL export (implemented in `src/ocel.rs` under the `wasm4pm-compat` feature) only records/logs the configuration process history. It has no authority to grant standing, which is explicitly corrected elsewhere in the receipt (line 41: `"records lifecycle history only — does not compute q_config"`) and in `docs/ontology/star-toml.ocel.ttl` line 45: `"OCEL records lifecycle/process history. It does not independently grant config standing."`. Thus, claiming that OCEL "grants standing" is false and contradictory.

### 3. Absolute Safety Mismatch
- **Documented Claim**: `docs/jira/v26.6.28/ST-108_path_bounds.md` description claims that it ensures path references do not lead to directory traversals, security escapes, or access to forbidden system paths, implying complete safety.
- **Actual Capability**: The path safety check is performed lexically in `clean_path` in `src/path.rs` (line 131) and does not hit the filesystem. As a result, it does not resolve or check symlinks, which could easily escape the sandbox. Furthermore, it lacks check coverage for other environment/platform-specific paths. Thus, claiming "complete safety" is a falsifier because the lexical sandbox check can be bypassed by filesystem symlinks.

### 4. Trusted Usage Bypass of `load_admitted`
- **Documented Claim**: `docs/jira/v26.6.28/ST-102_trusted_api.md` key requirement 3 states that the terminal `load::<T>()` method on `TrustedLoader` returns `Result<AdmittedConfig<T>, Error>`.
- **Actual Capability**: In `src/loader.rs` line 826, `TrustedLoader::load` actually returns `Result<TrustedConfig<T>>`. To obtain the actual `AdmittedConfig<T>`, the caller must call the separate method `load_admitted::<T>()` (line 1569) or `load_admitted_strict::<T>()` (line 1584). The `e2e_tests.rs` code (line 601) and unit tests also use `.load::<T>()` directly to obtain a `TrustedConfig` rather than an `AdmittedConfig`, bypassing `load_admitted()` entirely.

### 5. Raw Parse Bypass in Example Code
- **Documented Claim**: The `README.md` and JIRA board emphasize configuration validation as a linear typestate-enforced lifecycle (`Raw -> Merged -> Deserialized -> Validated -> Frozen`) designed to prevent accessing configuration values before semantic checks.
- **Actual Capability**: In `examples/validate.rs`, the code uses `from_str(BROKEN_CONFIG)` (which is a raw parse bypass implemented in `src/loader.rs` line 379) and then manually calls `app.check()` rather than utilizing the `star_toml::trusted()` / `AdmittedConfig<T>` pipeline. This treats the unchecked, untrusted deserialized struct as the law and completely bypasses typestate guarantees.
