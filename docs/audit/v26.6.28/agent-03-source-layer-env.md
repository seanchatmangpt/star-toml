# Audit Report: Source, Layer, and Env Override Reports (v26.6.28)

**Audit Target**: `star-toml` Source, Layer, and Env Override reports under typestate-safe loading, deep merging, prefix filtering, type coercion, and cryptographic witness hashing.  
**Auditor**: Agent 03  
**Status**: **ALIVE** (failset-zero verified)

---

## 1. Technical Analysis of Loader Infrastructure

The configuration loading subsystem is implemented in [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs) and utilizes the sequential builder pattern via [Loader](file:///Users/sac/star-toml/src/loader.rs#L140-L215). It implements a stateful typestate pipeline transitions from `Raw` to `BoundedSources` to `EnvResolved`, culminating in `Deserialized<T>`, `Validated<T>`, and finally `Frozen<T>`.

### How the Loader Captures Parameters:

1. **Defaults**: Captured as a string slice with a Static label through `Loader::layer_str` (mapped to `ConfigLayer::Str(String, &'static str)`). These are processed in `load_bounded` by hashing the content and deep merging.
2. **Files**: Discovered and loaded via:
   - `Loader::layer_file` (Required: `ConfigLayer::File(PathBuf)`): Fails immediately with `Error::FileNotFound` if missing on disk.
   - `Loader::layer_file_if_exists` (Optional: `ConfigLayer::FileIfExists(PathBuf)`): Skips missing files but logs them in the `SourceReport` as `found: false`.
   - `Loader::find_file` (Optional/Discovery: `ConfigLayer::FindFile(String)`): Walks parent folders starting from the current working directory.
3. **Environment Parameters**: Resolved in [apply_env](file:///Users/sac/star-toml/src/loader.rs#L962-L1029) by filtering the OS environment using a configured `env_prefix` (case-insensitive strip). Single underscores separate words, and double underscores (`__`) map to nested paths. Scalar strings are coerced to `Bool`, `Integer`, `Float`, or `Str` based on the target type, with any coercion failures resulting in config admission rejection.

### How Blake3 Digests are Computed:

All content hashing uses the BLAKE3 algorithm wrapped by [blake3_hex](file:///Users/sac/star-toml/src/reports.rs#L186-L188) in [src/reports.rs](file:///Users/sac/star-toml/src/reports.rs):
- **Source Digests**: Hashed raw contents of default strings and files (`blake3_hex(content.as_bytes())`).
- **Layer Digests**: Hashed raw TOML layer content string.
- **Layer Order Digest**: Hashed concatenation of preceding layer digests in merge order (`layer_order_acc.push_str(&digest)`). This forms a cryptographic chain of layer application sequence.
- **Env Override Raw Digest**: Hashed raw value of environment variables.
- **Env Override Coerced Digest**: Hashed coerced string representations (`toml_val.to_string()`).
- **Config Witness Hash**: Cryptographic witness generated in [ConfigWitness::compute](file:///Users/sac/star-toml/src/loader.rs#L1440-L1488) by joining:
  1. Source digests sorted by `source_id`.
  2. The last layer's `layer_order_digest`.
  3. Accepted environment override entries sorted as `"key=path:raw_digest"`.
  4. The validation fitness float representation (`{:.6}`).
  5. The BLAKE3 hex of the canonical TOML output bytes.
  
  These five parts are joined with `|` and hashed with BLAKE3 to produce the final `ConfigWitness::hash`.

---

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | Source kinds, stable layer order precedence (`Defaults < Files < Env`), prefix option, coercion types in [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs#L27-L40,L140-L215). | None. Bounds are fully declared. |
| O | Ambient environment variables, TOML input strings, files on disk. | None. Raw observations are fully inspectable. |
| O* | Parsed TOML Values, mapped and coerced environment variables filtered by prefix. | None. Admitted observations are lawful. |
| μ | Sequential evaluation in `load_bounded`, prefix matching in `apply_env`, `deep_merge_traced` in [src/merge.rs](file:///Users/sac/star-toml/src/merge.rs#L39-L53). | None. Transformation matches algebraic laws. |
| A | `Config<BoundedSources>`, `Config<EnvResolved>`, `SourceReport`, `LayerReport`, `EnvOverrideReport`, `WinnerMap`, `ConfigWitness` in [src/reports.rs](file:///Users/sac/star-toml/src/reports.rs). | None. Structural outputs exist. |
| C | 22 counterexample checks including 7 audited falsifiers in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs#L71-L333). | None. All detectors are integrated. |
| W | Cryptographic BLAKE3 digests of files, environment string values, and final `ConfigWitness::hash`. | None. Witnesses are complete. |
| q | `q_config` is computed and replayed when `failset_cardinality = 0` and the witness is validated. | None. Standing is computable. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | [tests/brce.rs:L94-107](file:///Users/sac/star-toml/tests/brce.rs#L94-L107) (`test_load_frozen_succeeds_with_valid_config`), [tests/brce.rs:L642-654](file:///Users/sac/star-toml/tests/brce.rs#L642-L654) (`test_brce_fitness_score`), [tests/brce.rs:L498-506](file:///Users/sac/star-toml/tests/brce.rs#L498-L506) (`test_relative_resolved_against_source_parent`). | None. | Positive examples establish capability. |
| Falsification | [tests/brce.rs:L137-145](file:///Users/sac/star-toml/tests/brce.rs#L137-L145) (`test_missing_required_file_fails`), [tests/brce.rs:L214-239](file:///Users/sac/star-toml/tests/brce.rs#L214-L239) (`test_unprefixed_admitted_env_override_fails`), [tests/brce.rs:L555-572](file:///Users/sac/star-toml/tests/brce.rs#L555-L572) (`test_load_admitted_strict_rejects_unknown_fields`). | None. | Confirms invalid inputs cause rejection. |
| Counterfactual | [tests/brce.rs:L398-417](file:///Users/sac/star-toml/tests/brce.rs#L398-L417) (`test_load_frozen_env_override_applied`), [tests/brce.rs:L443-461](file:///Users/sac/star-toml/tests/brce.rs#L443-L461) (`test_witness_changes_on_source_change`). | None. | Verifies perturbation affects standing. |
| Invariant | [tests/brce.rs:L79-91](file:///Users/sac/star-toml/tests/brce.rs#L79-L91) (`test_load_frozen_requires_validation`), [tests/brce.rs:L657-675](file:///Users/sac/star-toml/tests/brce.rs#L657-L675) (`test_brce_repair_hint_nonempty`), [tests/brce.rs:L261-274](file:///Users/sac/star-toml/tests/brce.rs#L261-L274) (`test_array_replacement_not_merge`), [tests/brce.rs:L276-285](file:///Users/sac/star-toml/tests/brce.rs#L276-L285) (`test_scalar_replacement`). | None. | Enforces structural guarantees. |
| Metamorphic | [tests/brce.rs:L579-596](file:///Users/sac/star-toml/tests/brce.rs#L579-L596) (`test_brce_metamorphic_canonical_stability`). | None. | Ensures stability under transformation. |
| Boundary | [tests/adversarial.rs](file:///Users/sac/star-toml/tests/adversarial.rs) and unit validators. | None. | Establishes explicit safety ranges. |
| Conservation | [tests/brce.rs:L322-346](file:///Users/sac/star-toml/tests/brce.rs#L322-L346) (`test_every_final_field_has_winning_layer`). | None. | Guarantees no missing provenance. |
| Determinism | [tests/brce.rs:L423-440](file:///Users/sac/star-toml/tests/brce.rs#L423-L440) (`test_witness_is_deterministic`), [tests/brce.rs:L373-391](file:///Users/sac/star-toml/tests/brce.rs#L373-L391) (`test_layer_order_digest_is_deterministic`), [tests/brce.rs:L617-639](file:///Users/sac/star-toml/tests/brce.rs#L617-L639) (`test_brce_env_coercion_deterministic`). | None. | Same inputs always yield same hash. |
| Idempotence | [tests/brce.rs:L598-616](file:///Users/sac/star-toml/tests/brce.rs#L598-L616) (`test_brce_idempotence_canonical`). | None. | Repeated pipelines stabilize. |
| Replay | Explicitly generated via `verifier_report` binary. | None. | Guarantees audit replayability. |
| Provenance | [tests/brce.rs:L188-211](file:///Users/sac/star-toml/tests/brce.rs#L188-L211) (`test_env_override_report_records_prefix_mapping`), [tests/brce.rs:L168-181](file:///Users/sac/star-toml/tests/brce.rs#L168-L181) (`test_source_report_str_layer`), [tests/brce.rs:L352-370](file:///Users/sac/star-toml/tests/brce.rs#L352-L370) (`test_layer_report_records_all_layers`). | None. | Records source and env details. |
| Witness | Coverage of files and variables in `ConfigWitness::compute`. | None. | Assures cryptographic audit completeness. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `implicit_source_used` | Yes ([src/loader.rs](file:///Users/sac/star-toml/src/loader.rs#L1089)) | Yes ([tests/brce.rs#L214-L239](file:///Users/sac/star-toml/tests/brce.rs#L214-L239)) | Yes | PASS |
| `missing_required_file_not_error` | Yes ([src/loader.rs#L1137-L1153](file:///Users/sac/star-toml/src/loader.rs#L1137-L1153)) | Yes ([tests/brce.rs#L137-L145](file:///Users/sac/star-toml/tests/brce.rs#L137-L145)) | Yes | PASS |
| `optional_missing_file_invisible` | Yes ([src/loader.rs#L1207-L1223](file:///Users/sac/star-toml/src/loader.rs#L1207-L1223)) | Yes ([tests/brce.rs#L148-L165](file:///Users/sac/star-toml/tests/brce.rs#L148-L165)) | Yes | PASS |
| `env_override_without_prefix` | Yes ([src/loader.rs#L967-L974](file:///Users/sac/star-toml/src/loader.rs#L967-L974)) | Yes ([tests/brce.rs#L214-L239](file:///Users/sac/star-toml/tests/brce.rs#L214-L239)) | Yes | PASS |
| `env_override_not_reported` | Yes ([src/loader.rs#L992-L1012](file:///Users/sac/star-toml/src/loader.rs#L992-L1012)) | Yes ([tests/brce.rs#L188-L211](file:///Users/sac/star-toml/tests/brce.rs#L188-L211)) | Yes | PASS |
| `unreported_layer_override` | Yes ([src/loader.rs#L1124,L1194,L1264](file:///Users/sac/star-toml/src/loader.rs#L1124)) | Yes ([tests/brce.rs#L305-L346](file:///Users/sac/star-toml/tests/brce.rs#L305-L346)) | Yes | PASS |
| `ambiguous_layer_order` | Yes ([src/loader.rs#L1089](file:///Users/sac/star-toml/src/loader.rs#L1089)) | Yes ([tests/brce.rs#L288-L302](file:///Users/sac/star-toml/tests/brce.rs#L288-L302)) | Yes | PASS |

### Standing Decision
- **ALIVE**: q computation is bounded, witnessed, replayable, and failset-zero for this surface.
