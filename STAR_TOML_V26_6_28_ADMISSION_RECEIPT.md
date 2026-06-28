# star-toml v26.6.28 — Admission Receipt

| Field | Value |
|---|---|
| **tag** | `v26.6.28` |
| **tag object** | `48adcc6c3cfb71be2382182fc46c433bbb2b8bb6` |
| **tip commit** | `ba0e8a238e097c76238a0650b7a34e559ac97481` |
| **standing** | **ALIVE** |
| **tests** | 158 passed / 0 failed |
| **feature tests** (`wasm4pm-compat`) | +4 passed / 0 failed |
| **counterexamples** | 22 / 22 at `failset_cardinality = 0` |
| **verifier binary** | `cargo run --bin verifier_report` → `VERIFIER_REPORT.md` |

---

## Admitted capabilities

| Work package | Symbol | Description |
|---|---|---|
| WP-1 | `BoundedSources`, `EnvResolved` | Typestate pipeline: Raw → BoundedSources → EnvResolved → Deserialized → Validated → Frozen |
| WP-2 | `SourceReport`, `LayerReport`, `EnvOverrideReport` | Provenance reports with BLAKE3 source digests |
| WP-3 | `WinnerMap`, `deep_merge_traced` | Per-leaf field provenance across merge layers |
| ST-102 | `AdmittedConfig<T>`, `load_admitted()`, `load_admitted_strict()` | Terminal admission envelope; only constructible after witness |
| ST-106 | `detect_unknown_fields`, `load_admitted_strict` | Runtime unknown-field detection via re-serialization; code `unknown_field` |
| ST-107 | `check_semver`, `check_ip_or_domain`, `check_size_format`, `check_path` | Precise-path validators in `Validator` |
| ST-108 | `PathPolicy`, `PathWitness`, `resolve_and_validate`, `check_path_safe` | Path bounds: sandbox, relative-only, forbidden-prefix |
| ST-109 | `ConfigWitness { hash: String }` | Deterministic BLAKE3 hash over source digests + layer order + env entries + fitness + canonical bytes |
| ST-111 | `verifier_report` binary, 31 BRCE tests | Counterexample ladder, 22/22 passing, `VERIFIER_REPORT.md` generated |
| OCEL | `export_events_to_ocel` (feature-gated) | Lifecycle history export via `wasm4pm-compat`; no q_config computation |

---

## Boundary invariants (verified)

```
star-toml → wasm4pm-compat   [optional dep, #[cfg(feature = "wasm4pm-compat")]]
star-toml → wasm4pm          [FORBIDDEN — never present, no circular risk]

save_canonical               [only on Config<Frozen<T>> — compile-time typestate]
AdmittedConfig<T>            [only constructible via build_admitted() after ConfigWitness]
OCEL export                  [records lifecycle history only — does not compute q_config]
q_config authority           [ConfigWitness + failset_cardinality = 0 + lifecycle validity]
```

---

## BRCE standing

```
q_config = 1  iff
  BoundSatisfied          ∧   (all required sources found, env prefix enforced)
  TransformLawful         ∧   (merge is deterministic, layer order is recorded)
  WitnessComplete         ∧   (ConfigWitness covers source + layer + env + fitness + canonical)
  CounterexamplesAbsent       (failset_cardinality = 0 across all 22 counterexamples)
```

All four conditions are satisfied at this tag.

---

## Authority separation

| Authority | Substrate | Does not do |
|---|---|---|
| `q_config` | `ConfigWitness` + `failset_cardinality = 0` | Process history |
| OCEL | `wasm4pm-compat` lifecycle events | Grant standing |
| `AdmittedConfig<T>` | Terminal artifact after witness-backed admission | Partial pipeline output |

---

*This receipt was produced by independent audit after commits `800cb39` and `aab0113`,
with stale semantic contradiction resolved in cleanup commit `ba0e8a2` before tagging.*
