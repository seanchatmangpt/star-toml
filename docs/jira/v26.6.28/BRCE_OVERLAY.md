# BRCE Standing Correction Overlay for Star TOML Swarm

This document corrects the swarm’s verification model. It establishes the rules of **bounded standing** for the Star TOML `26.6.28` release, replacing "testing" language with formal standing mathematical definitions.

---

## 1. Governing Equations

The configuration layer of any project must be treated as project law:

$$A = \mu(O^*)$$

Where:
* $O$ = raw observation (`O_config`)
* $O^*$ = admitted observation (`O*_config`)
* $\mu$ = lawful manufacturing transformation (`μ_star-toml`)
* $A$ = artifact with standing ($A_{config} \in \{ \text{ValidatedConfig, ValidationReport, CanonicalConfig, ConfigWitness} \}$)

For `star-toml`, the raw configuration is:

$$\text{O\_config} = \text{raw TOML} + \text{sources} + \text{layers} + \text{env} + \text{schema} + \text{validation rules} + \text{path policy} + \text{rewrite policy}$$

The bounding envelope is:

$$B_{config} = \langle \text{Sources, LayerOrder, EnvPolicy, TypeSchema, ValidationRules, PathPolicy, RewritePolicy, WitnessPolicy} \rangle$$

Config admission is:

$$O^*_{config} = \text{Admit}_{star-toml}(O_{config}, B_{config})$$

The standing bit $q_{config}$ is defined as:

$$q_{config} = 1 \iff \text{no detector in } C_{config} \text{ fires} \wedge \text{all required witnesses exist} \wedge \text{lifecycle is lawful} \wedge \text{invariants hold} \wedge \text{deterministic replay succeeds}$$

`q_config` is **bounded admissibility**, not simple boolean truth.

---

## 2. Replacing "Testing" with "Standing"

Do not ask:
* *Does this pass?*
* *Does this fail?*
* *Are tests present?*

Instead, ask:
* *What is the bound $B$?*
* *What is the raw observation $O$?*
* *What code admits $O$ into $O^*$?*
* *What transformation $\mu$ is lawful?*
* *What artifact $A$ is produced?*
* *What detectors $C$ exist?*
* *What witnesses $W$ exist?*
* *Can $q$ be computed?*
* *Can $q$ be replayed?*
* *Can $q$ be falsified by a live detector?*

Use the admissibility tuple:

$$\text{AdmissibilityTuple} = \langle B, O, O^*, \mu, A, C, W, q \rangle$$
$$q = f(B, O, \mu, C, W)$$
$$q = 1 \iff \text{BoundSatisfied}(B,O) \wedge \text{TransformLawful}(\mu) \wedge \text{WitnessComplete}(W) \wedge \text{CounterexamplesAbsent}(C)$$

The failset cardinality is defined as:

$$\text{failset\_cardinality} = |\{c \in C : c(x_t) = 1\}|$$

The counterexample set is a **failset**, not a mood or risk estimate.

---

## 3. Standing is Indexed

Standing is bound-relative: there is no universal standing. It is always indexed:

$$q_{config}(B_{config}, O_{config}, \mu_{star-toml}, C_{config}, W_{config})$$

Every agent must identify:
```markdown
## Bound Index
- B surface inspected:
- O surface inspected:
- μ surface inspected:
- C detectors inspected:
- W witnesses inspected:
- q evidence found:
```

---

## 4. Required BRCE Evidence Categories

Source code evidence must be classified under these categories:
1. **Truth Tests**: Positive examples (e.g. valid TOML loads). Do not establish standing by themselves.
2. **Falsification Tests**: Adversarial examples (e.g. invalid TOML fails). Do not establish standing by themselves.
3. **Counterfactual Tests**: Causal perturbation tests (e.g. env overrides altering digest; changing env variables to unknown keys makes $q=0$; path traversal makes $q=0$).
4. **Invariant Tests**: Structural guarantees (e.g. validation errors must include `loc`, `code`, `severity`, `message`, `repair_hint`).
5. **Metamorphic Tests**: Expected relationships under transformation (e.g. table reordering or whitespace does not alter canonical digest; winning layer change updates witness).
6. **Boundary Tests**: Explicit bounds (e.g. max port passes, max+1 fails).
7. **Conservation Tests**: Conservation of inputs (e.g. every final field has a winning source layer; no missing provenance).
8. **Determinism Tests**: Identical inputs yield identical outputs (`ConfigWitness`).
9. **Idempotence Tests**: Repeated transformations stabilize (`save_canonical(load_admitted(x)) == save_canonical(load_admitted(save_canonical(load_admitted(x))))`).
10. **Replay Tests**: Recomputing witness from bounds, sources, layers, env, validation, and canonical output.
11. **Provenance Tests**: Reporting env override key, path, coerced type, and value digest.
12. **Witness Tests**: Witness completeness rules (No source digest $\implies$ no witness; no env report $\implies$ no witness; no validation report $\implies$ no witness; no canonical digest $\implies$ no witness).

---

## 5. Lifecycle Reachability Graph

The lifecycle is an ordered graph, not a flat list of functions:

```text
Raw
→ BoundedSources
→ Merged
→ EnvResolved
→ Deserialized<T>
→ Validated<T>
→ Frozen<T>
→ CanonicalSaved
→ Witnessed
```

* An invalid lifecycle edge implies $q_{config} = 0$.
* Trusted save requires `Validated<T> ∨ AdmittedConfig<T>`.
* The edge `save_canonical(Raw)` is strictly forbidden in trusted mode.

---

## 6. Merge & Canonicalization Laws

### Merge Algebraic Laws:
* $\text{merge}(\text{table\_a}, \text{table\_b}) = \text{recursive\_by\_key}(\text{table\_a}, \text{table\_b})$
* $\text{merge}(\text{array\_a}, \text{array\_b}) = \text{array\_b}$
* $\text{merge}(\text{scalar\_a}, \text{scalar\_b}) = \text{scalar\_b}$
* merge is deterministic over ordered layers: $\text{defaults} < \text{files} < \text{env}$.

### Canonicalization Laws:
* $\text{canonical}(\text{canonical}(x)) = \text{canonical}(x)$
* $H(\text{canonical}(x)) = H(\text{canonical}(\text{canonical}(x)))$
* Canonical output is machine TOML. It does not imply comment preservation.
* A comment-preservation claim without a tested implementation is a counterexample: `comment_preservation_claim_unproven`.

---

## 7. Witness Integrity, Fitness & Repair limits

* **Witness completeness**: must cover $B_{config}$, Sources, Layers, Env, Validation, CanonicalOutput, merged value digest, typed config digest, canonical output digest, and $q_{config}$. An incomplete witness implies $q_{config} = 0$.
* **Fitness vs Standing**: $\text{fitness} = PassedChecks / TotalChecks$. It is a conformance signal. Standing requires $\text{failset\_cardinality} = 0 \wedge \text{WitnessComplete}(W) \wedge \text{lifecycle lawful} \wedge \text{deterministic replay}$. Thus, $\text{fitness} \neq \text{standing}$.
* **Repair Hints**: Suggestions, not authority: $\text{RepairHint} \neq \text{Authority}$. Admission must run again on repaired configs to check standing.
* **Macros & Schema**: `#[derive(Validate)]` emits validation code, not authority; `schema!` checks shape, not safety. Macros do not admit config.
* **LSP**: Live diagnostics that warn early. The gate decides standing later. LSP output does not produce $q_{config}$. Runtime must not depend on LSP.

---

## 8. Swarm Reporting Formats

### Agent Report Format
Every agent inspecting source code must report:
```markdown
## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | | |
| O | | |
| O* | | |
| μ | | |
| A | | |
| C | | |
| W | | |
| q | | |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | | | |
| Falsification | | | |
| Counterfactual | | | |
| Invariant | | | |
| Metamorphic | | | |
| Boundary | | | |
| Conservation | | | |
| Determinism | | | |
| Idempotence | | | |
| Replay | | | |
| Provenance | | | |
| Witness | | | |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|

### Standing Decision
Do not write “passes.” Write one of:
- ALIVE: q computation is bounded, witnessed, replayable, and failset-zero for this surface.
- PARTIAL_ALIVE: some q inputs exist, but witness/lifecycle/detector coverage is incomplete.
- BLOCKED: cannot assess due to missing files, missing dependency, or inaccessible source.
- BUILD_BROKEN: source surface exists but build state prevents admission confidence.
- UNKNOWN: insufficient evidence.
- UNSUPPORTED: no implementation exists for this standing surface.
```

### Agent 10 (BRCE Standing Auditor) Outputs
Agent 10 merges all reports into the following matrices:

```markdown
# BRCE Standing Matrix

| Surface | B present | O captured | O* admitted | μ lawful | A emitted | C detectors complete | W complete | q computable | Replay possible | Status |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|

# Failset Cardinality Report

| Counterexample detector | Fires? | Evidence | Required work |
|---|---:|---|---|

# Witness Completeness Report

| Witness component | Present? | Digest-covered? | Replayable? | Required work |
|---|---:|---:|---:|---|
```

No agent may declare standing using prose. Only bounded evidence supports standing:

$$\Sigma_{prose} \not\vdash q$$
