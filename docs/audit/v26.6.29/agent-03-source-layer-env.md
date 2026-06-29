# Audit Report: Source, Layer, and Env Override Reports (v26.6.29)

**Audit Target**: `star-toml` Source, Layer, and Env Override reports under typestate-safe loading, deep merging, prefix filtering, type coercion, and cryptographic witness hashing.  
**Auditor**: Agent 03  
**Status**: **ALIVE** (failset-zero verified)
**Date:** 2026-06-28

---

## 1. Technical Analysis of Loader Infrastructure

The configuration loading subsystem is implemented in [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs) and utilizes the sequential builder pattern via `Loader`. It implements a stateful typestate pipeline transitions from `Raw` to `BoundedSources` to `EnvResolved`, culminating in `Deserialized<T>`, `Validated<T>`, and finally `Frozen<T>`.

### How the Loader Captures Parameters:

1. **Defaults**: Captured as a string slice with a Static label through `Loader::layer_str`. These are processed in `load_bounded` by hashing the content and deep merging.
2. **Files**: Discovered and loaded via:
   - `Loader::layer_file` (Required): Fails immediately if missing on disk.
   - `Loader::layer_file_if_exists` (Optional): Skips missing files but logs them in the `SourceReport` as `found: false`.
   - `Loader::find_file` (Optional/Discovery): Walks parent folders starting from the current working directory.
3. **Environment Parameters**: Resolved in `apply_env` by filtering the OS environment using a configured `env_prefix` (case-insensitive strip). Single underscores separate words, and double underscores (`__`) map to nested paths. Scalar strings are coerced to `Bool`, `Integer`, `Float`, or `Str` based on the target type, with any coercion failures resulting in config admission rejection.

### How Blake3 Digests are Computed:

All content hashing uses the BLAKE3 algorithm:
- **Source Digests**: Hashed raw contents of default strings and files.
- **Layer Digests**: Hashed raw TOML layer content string.
- **Layer Order Digest**: Hashed concatenation of preceding layer digests in merge order. This forms a cryptographic chain of layer application sequence.
- **Env Override Raw Digest**: Hashed raw value of environment variables.
- **Env Override Coerced Digest**: Hashed coerced string representations.
- **Config Witness Hash**: Cryptographic witness generated in `ConfigWitness::compute` by joining:
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
| B | Source kinds, stable layer order precedence (`Defaults < Files < Env`), prefix option, coercion types in `src/loader.rs`. | None. Bounds are fully declared. |
| O | Ambient environment variables, TOML input strings, files on disk. | None. Raw observations are fully inspectable. |
| O* | Mapped and coerced environment variables filtered by prefix. | None. Admitted observations are lawful. |
| μ | Sequential evaluation in `load_bounded`, prefix matching in `apply_env`, `deep_merge_traced` in `src/merge.rs`. | None. Transformation matches algebraic laws. |
| A | `Config<BoundedSources>`, `Config<EnvResolved>`, `SourceReport`, `LayerReport`, `EnvOverrideReport`, `WinnerMap`, `ConfigWitness`. | None. Structural outputs exist. |
| C | 23 counterexample checks including 7 audited falsifiers in `src/bin/verifier_report.rs`. | None. All detectors are integrated. |
| W | Cryptographic BLAKE3 digests of files, environment string values, and final `ConfigWitness::hash`. | None. Witnesses are complete. |
| q | `q_config` is computed and replayed when `failset_cardinality = 0` and the witness is validated. | None. Standing is computable. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | `tests/brce.rs` (`test_load_frozen_succeeds_with_valid_config`, `test_brce_fitness_score`, `test_relative_resolved_against_source_parent`). | None. | Positive examples establish capability. |
| Falsification | `tests/brce.rs` (`test_missing_required_file_fails`, `test_unprefixed_admitted_env_override_fails`, `test_load_admitted_strict_rejects_unknown_fields`). | None. | Confirms invalid inputs cause rejection. |
| Counterfactual | `tests/brce.rs` (`test_load_frozen_env_override_applied`, `test_witness_changes_on_source_change`). | None. | Verifies perturbation affects standing. |
| Invariant | `tests/brce.rs` (`test_load_frozen_requires_validation`, `test_brce_repair_hint_nonempty`, `test_array_replacement_not_merge`, `test_scalar_replacement`). | None. | Enforces structural guarantees. |
| Metamorphic | `tests/brce.rs` (`test_brce_metamorphic_canonical_stability`). | None. | Ensures stability under transformation. |
| Boundary | `tests/adversarial.rs` and unit validators. | None. | Establishes explicit safety ranges. |
| Conservation | `tests/brce.rs` (`test_every_final_field_has_winning_layer`). | None. | Guarantees no missing provenance. |
| Determinism | `tests/brce.rs` (`test_witness_is_deterministic`, `test_layer_order_digest_is_deterministic`, `test_brce_env_coercion_deterministic`). | None. | Same inputs always yield same hash. |
| Idempotence | `tests/brce.rs` (`test_brce_idempotence_canonical`). | None. | Repeated pipelines stabilize. |
| Replay | Explicitly generated via `verifier_report` binary. | None. | Guarantees audit replayability. |
| Provenance | `tests/brce.rs` (`test_env_override_report_records_prefix_mapping`, `test_source_report_str_layer`, `test_layer_report_records_all_layers`). | None. | Records source and env details. |
| Witness | Coverage of files and variables in `ConfigWitness::compute`. | None. | Assures cryptographic audit completeness. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| `implicit_source_used` | Yes | Yes | Yes | PASS |
| `missing_required_file_not_error` | Yes | Yes | Yes | PASS |
| `optional_missing_file_invisible` | Yes | Yes | Yes | PASS |
| `env_override_without_prefix` | Yes | Yes | Yes | PASS |
| `env_override_not_reported` | Yes | Yes | Yes | PASS |
| `unreported_layer_override` | Yes | Yes | Yes | PASS |
| `ambiguous_layer_order` | Yes | Yes | Yes | PASS |

### Standing Decision
- **ALIVE**: q computation is bounded, witnessed, replayable, and failset-zero for this surface.
