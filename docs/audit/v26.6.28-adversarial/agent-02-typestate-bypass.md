# Audit Report: Typestate Bypass and Admission Forgery (v26.6.28-adversarial)

**Audit Target**: Typestate Pipeline, Admission Forgery, and `AdmittedConfig` Construction
**Auditor**: Agent 02
**Standing**: **PARTIAL_ALIVE**
**Failset Cardinality**: 0 (all detectors pass)

---

## 1. Executive Summary

An adversarial audit of the **Typestate Bypass and Admission Forgery** surfaces of `star-toml` was conducted. The audit focused on checking the paths for `AdmittedConfig` construction, the visibility of constructor fields and methods, and the structural integrity of compile-time typestate-fences.

### Key Findings
1. **Public Struct Fields in AdmittedConfig:** The fields of [AdmittedConfig](file:///Users/sac/star-toml/src/loader.rs#L1538) (`value`, `witness`, `source_report`, `layer_report`, `env_report`, `global_winner_map`) are fully public (`pub`). This allows downstream systems to manually instantiate the struct using struct literal syntax, bypassing the entire validation and admission pipeline.
2. **Public ConfigWitness Fields:** The [ConfigWitness](file:///Users/sac/star-toml/src/loader.rs#L1425) struct has a single public field `hash: String`, enabling downstream code to manufacture arbitrary witnesses without calling [ConfigWitness::compute](file:///Users/sac/star-toml/src/loader.rs#L1440).
3. **Typestate-Fence Subversion:** The typestates `Raw`, `Merged`, `Deserialized<T>`, `Validated<T>`, and `Frozen<T>` are implemented as public tuple structs with public inner values (e.g., `pub struct Validated<T>(pub T)`). Additionally, [Config<S>](file:///Users/sac/star-toml/src/loader.rs#L514) has public fields. Downstream code can directly instantiate any typestate wrapper (like `Config<Validated<T>>`) around unvalidated data and call restricted methods like [save_canonical](file:///Users/sac/star-toml/src/loader.rs#L704) or pass them to APIs expecting validated configurations.
4. **No FS-Level/Compiler Enforcement:** The codebase lacks zero-sized private fields or constructor-hiding design patterns. This means typestate-fences hold only when developers voluntarily use the library's loading APIs, but are completely bypassable by an adversarial or careless consumer.

---

## 2. Command Executions and Evidence

The following targeted search was performed to extract all references to typestate definitions, lifecycle methods, and admission envelopes:

```bash
rg -n "pub struct AdmittedConfig|impl AdmittedConfig|build_admitted|load_admitted|load_admitted_exploratory|load_frozen|save_canonical|Deref|pub value|pub witness|ConfigWitness|q_config|Validated|Frozen|Deserialized|Raw" src tests
```

### Verbatim Findings from Search:

1. **AdmittedConfig Definition:**
   - [src/loader.rs:1538](file:///Users/sac/star-toml/src/loader.rs#L1538): `pub struct AdmittedConfig<T> {`
2. **AdmittedConfig Deref Implementation:**
   - [src/loader.rs:1553](file:///Users/sac/star-toml/src/loader.rs#L1553): `impl<T> std::ops::Deref for AdmittedConfig<T> {`
3. **Helper build_admitted Calls and Definition:**
   - [src/loader.rs:1630](file:///Users/sac/star-toml/src/loader.rs#L1630): `build_admitted(frozen_result)`
   - [src/loader.rs:1645](file:///Users/sac/star-toml/src/loader.rs#L1645): `build_admitted(result)`
   - [src/loader.rs:1659](file:///Users/sac/star-toml/src/loader.rs#L1659): `fn build_admitted<T: Serialize>(result: FrozenLoadResult<T>) -> Result<AdmittedConfig<T>> {`
4. **load_admitted and load_admitted_exploratory Methods:**
   - [src/loader.rs:1572](file:///Users/sac/star-toml/src/loader.rs#L1572): `pub fn load_admitted<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(`
   - [src/loader.rs:1641](file:///Users/sac/star-toml/src/loader.rs#L1641): `pub fn load_admitted_exploratory<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(`
   - [src/loader.rs:1652](file:///Users/sac/star-toml/src/loader.rs#L1652): `pub fn load_admitted_strict<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(` (deprecated alias calling `load_admitted`)
5. **load_frozen Method and Structs:**
   - [src/loader.rs:915](file:///Users/sac/star-toml/src/loader.rs#L915): `pub struct FrozenLoadResult<T> {`
   - [src/loader.rs:1393](file:///Users/sac/star-toml/src/loader.rs#L1393): `pub fn load_frozen<T: DeserializeOwned + Validate + ConfigLifecycle>(`
6. **save_canonical Implementations:**
   - [src/loader.rs:704](file:///Users/sac/star-toml/src/loader.rs#L704): `pub fn save_canonical(&self, path: impl AsRef<Path>) -> Result<()>` (on `Config<Frozen<T>>`)
   - [src/loader.rs:715](file:///Users/sac/star-toml/src/loader.rs#L715): `pub fn save_canonical(&self, path: impl AsRef<Path>) -> Result<()>` (on `Config<Validated<T>>`)
7. **Public Fields (value and witness):**
   - [src/loader.rs:1540](file:///Users/sac/star-toml/src/loader.rs#L1540): `pub value: T,`
   - [src/loader.rs:1542](file:///Users/sac/star-toml/src/loader.rs#L1542): `pub witness: ConfigWitness,`
8. **ConfigWitness Struct:**
   - [src/loader.rs:1425](file:///Users/sac/star-toml/src/loader.rs#L1425): `pub struct ConfigWitness {`
9. **Typestate Wrappers:**
   - [src/loader.rs:494](file:///Users/sac/star-toml/src/loader.rs#L494): `pub struct Raw(pub Value);`
   - [src/loader.rs:498](file:///Users/sac/star-toml/src/loader.rs#L498): `pub struct Merged(pub Value);`
   - [src/loader.rs:502](file:///Users/sac/star-toml/src/loader.rs#L502): `pub struct Deserialized<T>(pub T);`
   - [src/loader.rs:506](file:///Users/sac/star-toml/src/loader.rs#L506): `pub struct Validated<T>(pub T);`
   - [src/loader.rs:510](file:///Users/sac/star-toml/src/loader.rs#L510): `pub struct Frozen<T>(pub T);`

---

## 3. Analysis of Typestate and Admission Integrity

### 3.1. Construction of `AdmittedConfig`
Inside `star-toml`, `AdmittedConfig` is constructed exclusively by the private helper [build_admitted](file:///Users/sac/star-toml/src/loader.rs#L1659). This function executes after checking validation invariants and generating the deterministic BLAKE3 witness. However, because `AdmittedConfig` is defined with all public fields and lacks any private fields or private constructor barriers, it can be constructed by any external module:
```rust
let fake_admitted = AdmittedConfig {
    value: my_unvalidated_value,
    witness: ConfigWitness { hash: "fake-hash".to_owned() },
    source_report: SourceReport::default(),
    layer_report: LayerReport::default(),
    env_report: EnvOverrideReport::default(),
    global_winner_map: WinnerMap::default(),
};
```
This is a severe design vulnerability as it completely decouples the conceptual standing indicator $q_{config}$ from compile-time proof.

### 3.2. Typestate-Fence Subversion
The typestate lifecycle enforces pipeline order at the signature level (e.g. `save_canonical` requires `Config<Frozen<T>>` or `Config<Validated<T>>`).
However, because:
1. `Config<S>`'s fields `state: S` and `path: PathBuf` are public.
2. The states `Raw`, `Merged`, `Deserialized<T>`, `Validated<T>`, and `Frozen<T>` are tuple structs with public fields.

Downstream code can instantly bypass the pipeline at compile time:
```rust
use star_toml::{Config, Validated};
let unvalidated = Config {
    state: Validated(invalid_config_struct),
    path: std::path::PathBuf::from("fake.toml"),
};
// save_canonical is now callable on this unvalidated data!
unvalidated.save_canonical("output.toml").unwrap();
```
This allows callers to generate canonical serialized representations or fake a validated state without invoking the parser, deep merger, env coercion, or validator checks.

---

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | State wrapper structs (`Raw`, `Merged`, etc.) and `AdmittedConfig` envelope. | Struct encapsulation to prevent arbitrary manual construction of wrapper types. |
| O | Core loader state values and witness structures. | Private fields or zero-sized markers enforcing that inputs must come from authorized loader functions. |
| O* | `AdmittedConfig` envelope returned by `load_admitted()`. | Cryptographic verification that `AdmittedConfig` has not been manually constructed or altered. |
| μ | Transitions step-by-step (`Raw` -> `Merged` -> `Deserialized` -> `Validated` -> `Frozen`). | Compiler guarantees enforcing that transitions must occur via library methods. |
| A | Witness records and reports bundled with `AdmittedConfig`. | None. |
| C | Verifier check 1 (`parse_valid_treated_as_trusted`), check 9 (`validation_not_run`), check 17 (`rewrite_without_validation`), check 23 (`ocel_treated_as_standing_authority`). | Verifier check validating that external modules cannot compile if they attempt to instantiate `AdmittedConfig` or `Config<Validated>` without library pipelines (i.e. private constructor compile-fail test). |
| W | `ConfigWitness` with BLAKE3 hash computation. | None. |
| q | Admissibility standing conceptually asserted via pipeline completion. | Absolute safety, since $q_{config}=1$ can be forged by constructing typestates manually. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | [test_load_admitted_succeeds](file:///Users/sac/star-toml/tests/brce.rs#L538) verifies successful loading of admitted configs. | None. | High (Positive validation). |
| Falsification | [test_load_admitted_strict_rejects_unknown_fields](file:///Users/sac/star-toml/tests/brce.rs#L555) verifies invalid field rejection. | Tests validating compile-time rejection of manual typestate or witness creation. | High (Pipeline bypasses are not checked). |
| Counterfactual | Verifier check 9 ensures validation is run during `load_frozen`. | None. | High. |
| Invariant | `_compile_fail_save_canonical_before_validation` doc-test in [src/lib.rs](file:///Users/sac/star-toml/src/lib.rs#L460) ensures unvalidated config types cannot call `save_canonical`. | Doc-test verifying compile fail when constructing `Validated(T)` manually. | High (The compiler constraint is subverted by public tuple fields). |
| Metamorphic | [test_brce_metamorphic_canonical_stability](file:///Users/sac/star-toml/tests/brce.rs#L582) checks stability of canonical formatting. | None. | Medium. |
| Boundary | [build_admitted](file:///Users/sac/star-toml/src/loader.rs#L1659) computes witness deterministically. | None. | High. |
| Conservation | All components mapped to the witness in `ConfigWitness::compute`. | None. | High. |
| Determinism | `test_witness_is_deterministic` in [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L423). | None. | High. |
| Idempotence | `test_brce_idempotence_canonical` in [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L598). | None. | Medium. |
| Replay | Witness computation is fully deterministic and inspectable. | None. | Medium. |
| Provenance | Winner tracing is recorded to maps and reports. | None. | Medium. |
| Witness | Checks 18-21 in `verifier_report.rs`. | Checks ensuring witness structure cannot be manually instantiated. | High. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| **`1. parse_valid_treated_as_trusted`** | Yes | Yes | Yes | **PASS** |
| **`9. validation_not_run`** | Yes | Yes | Yes | **PASS** |
| **`17. rewrite_without_validation`** | Yes | Yes | Yes | **PASS** |
| **`23. ocel_treated_as_standing_authority`** | Yes | Yes | Yes | **PASS** |

*Note: Although all four verifier checks pass, they only evaluate the library's API flows under cooperative usage. They fail to detect the structural bypasses allowed by public constructors and public fields.*

### Standing Decision
**PARTIAL_ALIVE**  
*Rationale:* The typestate pipeline functions correctly when consumers use the library's public load methods, successfully enforcing validation execution and witness calculations prior to admission. However, the standing is constrained to `PARTIAL_ALIVE` because the compiler cannot enforce typestate-fences or prevent admission forgery. The public visibility of all fields in `AdmittedConfig`, `Config<S>`, `ConfigWitness`, and the inner types of the tuple states (`Raw`, `Merged`, `Deserialized`, `Validated`, `Frozen`) allows downstream code to bypass every check in the library by manually constructing valid-looking wrappers around unvalidated or forged values.
