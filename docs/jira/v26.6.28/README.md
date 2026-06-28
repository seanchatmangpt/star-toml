# Jira Board for Star TOML v26.6.28 (Vision 2030 Release)

This directory contains the formal Jira tickets outlining the requirements, acceptance criteria, and verification methods for transitioning **star-toml** into the **Vision 2030 Config Admission Substrate**.

## 📌 BRCE Standing Correction Overlay
The verification semantics for all tickets and agents must strictly follow the **[BRCE Standing Correction Overlay](file:///Users/sac/star-toml/docs/jira/v26.6.28/BRCE_OVERLAY.md)**. This overlay replaces traditional "testing" terminology with "standing" mathematics, establishes required evidence categories, defines the lifecycle reachability graph, merge/canonicalization algebraic laws, and specifies the audit reports.

## The BRCE Mathematical Standing Framework

In accordance with the **Bounded Receipted Chatman Equation (BRCE)**, configuration is not just checked for syntax or positive happy-path examples. It earns standing as project law only through explicit mathematical bounds, invariant tracking, metamorphic stability, idempotence, and witness replay.

### 1. The Admissibility Tuple
We represent the configuration state space as a formal tuple:

$$\text{AdmissibilityTuple} = \langle B, O, O^*, \mu, A, C, W, q \rangle$$

Where:
* **$B$** is the declared admission boundary: $B_{config} = \langle \text{Sources, LayerOrder, EnvPolicy, TypeSchema, ValidationRules, PathPolicy, RewritePolicy, WitnessPolicy} \rangle$.
* **$O$** is the raw configuration observation: $O_{config}$.
* **$O^*$** is the admitted config: $O^*_{config} = \text{Admit}_{star-toml}(O_{config}, B_{config})$.
* **$\mu$** is the lawful transformation: $\mu_{star-toml}(O^*_{config})$.
* **$A$** is the generated artifact with standing: $A_{config} \in \{ \text{ValidatedConfig, ValidationReport, CanonicalConfig, ConfigWitness} \}$.
* **$C$** is the counterexample detector set: $C_{config}$.
* **$W$** is the witness set: $W_{config}$.
* **$q$** is the quality/standing bit: $q_{config} \in \{0, 1\}$.

### 2. Admissibility Rules
Configuration is admitted if and only if it sits within the declared bounds and no counterexample is triggered:

$$O^*_{config} = 1 \iff Parse = 1 \wedge LayerMerge = 1 \wedge Deserialize = 1 \wedge Validate = 1 \wedge PathSafe = 1 \wedge DeterministicRewrite = 1 \wedge Witness = 1 \wedge CounterexampleSet = \emptyset$$

### 3. The Standing Quality Signal ($q_{config}$)
The standing bit represents bounded admissibility, not simple boolean boolean truth. It is defined as:

$$q_{config} = 1 \iff \text{BoundSatisfied}(B, O) \wedge \text{TransformLawful}(\mu) \wedge \text{WitnessComplete}(W) \wedge \text{CounterexamplesAbsent}(C)$$
$$q_{config} = 0 \iff \exists c \in C \text{ such that } c(x_t) = 1$$

The overall crate release condition is:

$$V_{star-toml,26.6.28} = 1 \iff \neg\exists c \in C_{star\_toml} : c(x_t) = 1 \iff \text{failset\_cardinality} = 0$$

Where **failset\_cardinality** $= |\{c \in C : c(x_t) = 1\}|$. Standing is strictly monotonic with evidence, not prose: $\Sigma_{prose} \not\vdash q$.

### 4. Config Witness
The witness is receipt-ready cryptographic evidence:

$$\text{ConfigWitness} = H(B_{config}, \text{Sources, Layers, Env, Validation, CanonicalOutput})$$

---

## JIRA Board Roadmap (v26.6.28)

| Ticket ID | Type | Title | Summary |
|---|---|---|---|
| **[ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md)** | Story | Core Typestate Lifecycle | Defines the 5-stage lifecycle `Config<Raw>` $\to$ `Config<Merged>` $\to$ `Config<Deserialized<T>>` $\to$ `Config<Validated<T>>` $\to$ `Config<Frozen<T>>` to prevent unvalidated loads. |
| **[ST-102](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-102_trusted_api.md)** | Story | Trusted API & AdmittedConfig | Implements the entry `star_toml::trusted()` builder and the output `AdmittedConfig<T>` struct packaging value, witness, and source reports. |
| **[ST-103](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-103_source_bounds.md)** | Story | Source Bounds | Specifies explicit, labeled, allowed, digestible, and reported sources, and validates presence of required files. |
| **[ST-104](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-104_layer_bounds.md)** | Story | Layer Bounds & Win-Layer Tracing | Implements deterministic merge precedence (`Defaults < Files < Env`) and win-layer metadata lookup for every field. |
| **[ST-105](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-105_env_bounds.md)** | Story | Environment Bounds | Implements prefix-filtering, double-underscore nested table mapping, and deterministic type coercion for environment variables. |
| **[ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md)** | Story | Type Bounds | Enforces `deny_unknown_fields` by default in trusted mode, and logs/reports unrecognized fields in exploratory modes. |
| **[ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md)** | Story | Precise Path Validation | Specifies post-deserialization validation with errors capturing path `loc`, stable `code`, `severity`, and `repair_hint`, plus semver, size, IP/domain, and cross-field checkers. |
| **[ST-108](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-108_path_bounds.md)** | Story | Path Bounds | Guard rails rejecting traversals (`..`), null bytes, forbidden paths, resolving relative paths relative to source configuration files, and generating `PathWitness`. |
| **[ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md)** | Story | Rewrite & Witness Bounds | Implements deterministic alpha-sorted `save_canonical` outputs, comment preservation round-trip verification, and `ConfigWitness` hashing. |
| **[ST-110](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-110_error_topology_lsp.md)** | Story | Error Topology & LSP Protocol | Establishes variant hashing (`variant_id = H(sorted(loc:code))`) and a diagnostics-only LSP server (spans, duplicate key detection, recovery, extensions). |
| **[ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md)** | Story | BRCE Verification Ladder | Outlines the 26 mathematical validation, chaos, stress, and invariant tests, and the Verifier Report generator. |

---

## Traceability Matrix: Vision 2030 Sections to Jira Tickets

| Vision 2030 Section | Key Concept | Target Jira Ticket |
|---|---|---|
| **1. The Root Claim** | Project Law ($A = \mu(O^*)$) | [ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md) |
| **2. 2030 Failure Being Solved** | ConfigDrift detection, BadConfig = BadLaw | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) |
| **3. Parse Is Not Admission** | Syntax $\neq$ Authority | [ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md) |
| **4. The Scope of Star TOML** | Substrate boundary (no project-specific policy inside core) | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) |
| **5. BRCE and Star TOML** | Boundary tuple $B_{config}$ definition | [ST-103](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-103_source_bounds.md) $\dots$ [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) |
| **6.1 Source Bounds** | Explicit config source reporting / checking | [ST-103](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-103_source_bounds.md) |
| **6.2 Layer Bounds** | Deterministic merge priority / tracking winners | [ST-104](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-104_layer_bounds.md) |
| **6.3 Env Bounds** | Prefix check, double-underscore nesting, coercion | [ST-105](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-105_env_bounds.md) |
| **6.4 Type Bounds** | Deny unknown fields in trusted mode | [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md) |
| **6.5 Validation Bounds** | Path-precise, semver, IP/domain, cross-field rules | [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) |
| **6.6 Path Bounds** | Reject traversal (`..`), null bytes, generate PathWitness | [ST-108](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-108_path_bounds.md) |
| **6.7 Rewrite Bounds** | Deterministic `save_canonical`, prove comment preservation | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) |
| **6.8 Witness Bounds** | Witness hashing of inputs, settings, validation, output | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) |
| **7. Trusted Lifecycle** | Typestate: Raw $\to$ Merged $\to$ Deserialized $\to$ Validated $\to$ Frozen | [ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md) |
| **8. Trusted API** | Ergonomics of `trusted()` builder | [ST-102](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-102_trusted_api.md) |
| **9. Macros** | derive macro & schema! limits | [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) |
| **10. Error Topology** | FNV variant hashing, fitness, repair hints | [ST-110](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-110_error_topology_lsp.md) |
| **11. Star TOML LSP** | Spans, line/col lookup, duplicate keys, extension APIs | [ST-110](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-110_error_topology_lsp.md) |
| **12. Stack Alignment** | Integration gates with cargo-cicd, receipts, OCEL | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) |
| **13. DfCM Design** | Config maximalism enforcement | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) |

---

## Traceability Matrix: Counterexample Set to Jira Tickets

The **Counterexample Set (Section 16)** defines the release criteria. The following matrix shows which Jira tickets implement the safety logic to prevent each failure state:

| Counterexample Name | Danger | Mitigated by Ticket | Verification Phase |
|---|---|---|---|
| `parse_valid_treated_as_trusted` | Treating unvalidated syntax as law | [ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md) | E2E Integration |
| `implicit_source_used` | Silent fallback / magic discovery | [ST-103](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-103_source_bounds.md) | Unit & Integration |
| `missing_required_file_not_error` | Required file absent but loading succeeds | [ST-103](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-103_source_bounds.md) | Unit & Chaos |
| `ambiguous_layer_order` | Non-deterministic layer overlay precedence | [ST-104](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-104_layer_bounds.md) | Unit & Integration |
| `unreported_layer_override` | Silent value override by higher priority layer | [ST-104](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-104_layer_bounds.md) | Unit & Integration |
| `env_override_without_prefix` | Ambient env vars polluting config | [ST-105](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-105_env_bounds.md) | Unit & Chaos |
| `env_override_not_reported` | Ambient env override applied but not in witness | [ST-105](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-105_env_bounds.md) | Unit & Integration |
| `unknown_field_accepted_in_trusted_mode` | Permissive deserialization inside security bounds | [ST-106](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-106_type_bounds.md) | Unit & Integration |
| `validation_not_run` | Bypassing validation phase in the lifecycle | [ST-101](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-101_typestate.md) | E2E Integration |
| `validation_error_without_path` | Semantic error lacking precise tree position | [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) | Unit & Integration |
| `fatal_error_downgraded` | Softening a Fatal severity failure to Warning | [ST-107](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-107_validation_bounds.md) | Unit |
| `path_traversal_accepted` | Path pointing outside source root / using `..` | [ST-108](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-108_path_bounds.md) | Chaos & Stress |
| `null_byte_path_accepted` | C-string termination attacks inside file paths | [ST-108](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-108_path_bounds.md) | Chaos & Stress |
| `source_relative_path_unresolved` | Path resolved relative to CWD instead of config | [ST-108](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-108_path_bounds.md) | Unit & Integration |
| `nondeterministic_save` | Writing TOML back to disk with map key drift | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Unit & Stress |
| `comment_preservation_claim_unproven` | Claiming comment-safe rewrites without checks | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Integration |
| `rewrite_without_validation` | Saving unvalidated/inadmissible config to disk | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | E2E Integration |
| `witness_missing_source_digest` | Witness digest ignores source metadata | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Unit & Integration |
| `witness_missing_env_report` | Witness digest ignores env var override maps | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Unit & Integration |
| `witness_missing_validation_report` | Witness digest ignores validation error history | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Unit & Integration |
| `witness_nondeterministic` | Running the same config results in different hashes | [ST-109](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-109_rewrite_witness_bounds.md) | Unit & Benchmarks |
| `downstream_policy_inside_star_toml` | Embedding application logic in config parser | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) | Architecture Audit |
| `ocel_treated_as_standing_authority` | Treating OCEL lifecycle log as independent config standing authority | [ST-111](file:///Users/sac/star-toml/docs/jira/v26.6.28/ST-111_verification_ladder.md) | Integration & Features |
