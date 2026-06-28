# ST-112: Release Standing Gate

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for the Star TOML **Release Standing Gate** (`release_verifier`). Under the **Vision 2030** design discipline, config release caveats must be treated as unbounded axes of uncertainty. Rather than managing caveats as prose-based warnings or loose documentation notes, the framework must convert every caveat into a named, checked counterexample. Config release standing is refused if the failset of these counterexamples is non-empty.

---

## The DfCM Release Law

The mathematical formula governing config admission and packaging validation is:

$$q_{release} = 1 \iff q_{code} \land q_{docs} \land q_{api} \land q_{examples} \land q_{lsp} \land q_{publish} \land |F| = 0$$

Where:
* $F$ is the failset of caveat counterexamples.
* $q_{code}$: 100% of unit, integration, and E2E tests are passing.
* $q_{docs}$: 100% of documentation claims map to verified test/code evidence.
* $q_{api}$: 100% of public API surface is explicitly mapped to a lifecycle phase with misuse tests.
* $q_{examples}$: 100% of DfCM examples pass and run in admitted-only paths.
* $q_{lsp}$: 100% of claimed LSP features have functional handlers and safety gates.
* $q_{publish}$: Cargo package checks, dependency ordering, and clean repository statuses are verified.

If any caveat or documentation drift is identified, the corresponding counterexample is registered in $F$:

$$\text{Design Debt } \lor \text{ API Rough Edge } \lor \text{ Doc Drift } \lor \text{ Hidden Defect} \implies F \neq \emptyset \implies q_{release} = 0$$

---

## 1. Caveat-to-Counterexample Translation Matrix

All qualitative caveats are mapped to executable, verifiable detectors:

| Caveat | DfCM Counterexample | Detector ID |
|--------|---------------------|-------------|
| **Design Debt** | Architecture claim without invariant | `architecture_claim_without_invariant` |
| **API Rough Edge** | Public API item without example/misuse test | `public_api_without_example` |
| **Documentation Drift** | Prose claim lacking matching verification test | `doc_claim_not_backed_by_test` |
| **Young-Project Defect** | Structural variant axis lacking a test fixture | `variant_axis_without_fixture` |
| **Release Overclaim** | Claimed feature exists in code without release gate | `implemented_claim_without_gate` |
| **LSP Overclaim** | LSP capabilities register features lacking handlers | `lsp_feature_claim_without_handler` |
| **Security Overclaim** | Threat claim (e.g. symlink traverse) without test gate | `threat_claim_without_detector` |
| **Witness Overclaim** | Witness component hash omitted from calculations | `witness_claim_without_hash_input` |
| **OCEL Confusion** | Event history treated as standing authority | `ocel_treated_as_standing_authority` |
| **Deferred Feature Confusion** | Deferred feature acceptance criteria marked complete | `deferred_feature_marked_complete` |

---

## 2. Hardening Requirements

### A. Design Debt Non-Standing (`architecture_claim_without_invariant`)
Design debt exists when architectural assertions lack compiler-enforced invariants. Every architectural claim must follow the strict proof chain:

$$\text{Claim} \longrightarrow \text{Invariant} \longrightarrow \text{Code Boundary} \longrightarrow \text{Test} \longrightarrow \text{Documentation Anchor}$$

* **LSP Advisory Separation**: The LSP crate must have zero ability to construct or bypass `AdmittedConfig<T>` or `ConfigWitness`.
* **OCEL Isolation**: The lifecycle event log exports must remain strictly diagnostic and cannot compute or alter $q_{config}$.
* **No Untrusted Parse**: Core examples must reject raw parsing as trusted configuration, enforcing transition to `AdmittedConfig<T>`.

If any link in the chain is broken, then $q_{docs} = 0$.

### B. Public API Lifecycle Mapping (`public_api_without_lifecycle_position`)
Every public function, struct, enum, and macro must be explicitly assigned to exactly one lifecycle phase to prevent rough edges:

| Public Item | Lifecycle Phase | Standing Requirement |
|-------------|-----------------|----------------------|
| `TrustedLoader` | Builder Phase | Constructor + default file merge example |
| `load_admitted` | Terminal Phase | Strict field checks + failset tests |
| `load_admitted_exploratory` | Terminal Escape Phase | Permissive parsing + warning trace tests |
| `Validate` | Validation Phase | Macro parsing + out-of-range bounds tests |
| `Validator` | Validation Phase | Diagnostic accumulation + fitness tests |
| `ConfigWitness` | Witnessing Phase | Hash computation + deterministic order tests |
| `AdmittedConfig<T>` | Terminal Envelope Phase | Immutability wrapper + Deref tests |
| `save_canonical` | Serialization Phase | Alphabetic sort + Validated/Frozen-only tests |

If a public item lacks a documented lifecycle role, example coverage, or negative misuse tests, $q_{api} = 0$.

### C. Documentation Claim Verification (`doc_claim_not_backed_by_test`)
To prevent doc drift, every claim in the `README.md`, JIRA specs, LSP guidelines, or OWL/Turtle Ontologies must be categorized as:

$$\text{Claim Category} \in \{ \text{Implemented}, \text{Deferred}, \text{Conceptual}, \text{Forbidden}, \text{Future} \}$$

Any claim marked `Implemented` must be linked to at least one:
* Unit test
* Integration test
* Example binary
* Verifier detector
* Cargo-tree dependency audit
* Forbidden-symbol link scan
* Package dry-run validation

No implementation claim may reside purely in prose:

$$\sum \text{Prose} \not\vdash q_{release}$$

### D. Variant Axis Fixtures (`variant_axis_without_fixture`)
Every major release axis must be covered by a four-fixture matrix:
1. **Happy Fixture**: Valid inputs, standard path.
2. **Boundary Fixture**: Extreme bounds (min/max range limits, empty strings).
3. **Negative Fixture**: Invalid inputs, structural failure patterns.
4. **Cross-Axis Fixture**: Interlocking constraints (e.g. env overrides modifying validation limits).

The required axes mapping:

| Axis | Required Fixtures |
|------|-------------------|
| **Source** | Required file, missing optional file, inline defaults |
| **Layer** | Default fallback, file layers, merged profile profiles |
| **Env** | Prefixed overrides, ignored unprefixed, double-underscore nesting |
| **Type** | Known fields, strict unknown fields, exploratory mode |
| **Validation** | Range limits, string lengths, SemVer syntax, custom predicates |
| **PathPolicy** | Lexical sandboxing, RelativeOnly, null byte checks, forbidden prefixes |
| **Witness** | BLAKE3 digests, canonical TOML byte sequence, fitness scoring |
| **OCEL** | Sequence order preservation, ConfigRun instantiation |
| **LSP** | Diagnostic spans, duplicate key quick-fixes, syntax sync recovery |
| **Publish** | Package dry-runs, clean git trees, cyclic dependency checks |

If any cell in the axis matrix is empty, $q_{examples} = 0$.

---

## 3. Release Detector Engine (`release_verifier`)

The `release_verifier` component enforces the non-standing rules by running audits over the project workspace. It fails the release gate if any of the following counterexamples are detected:

1. `architecture_claim_without_invariant`: Architectural claims lacking structural or runtime invariants.
2. `public_api_without_example`: Public functions/types lacking doc-tests or examples.
3. `public_api_without_lifecycle_position`: Public API items not explicitly assigned to a lifecycle phase.
4. `doc_claim_not_backed_by_test`: Prose claims lacking test/fixture evidence.
5. `deferred_feature_marked_complete`: Stale deferred items showing up as implemented.
6. `implemented_claim_without_gate`: Code implementation lacks a corresponding check gate.
7. `lsp_feature_claim_without_handler`: LSP capabilities claiming formatting or extension checks without active code handlers.
8. `lsp_authority_boundary_violation`: LSP code importing internal authority builders.
9. `ocel_treated_as_standing_authority`: Treating event traces as proof of config standing.
10. `raw_parse_treated_as_trusted`: Examples bypassing the `AdmittedConfig` pipeline wrapper.
11. `example_uses_non_admitted_path`: Examples showing configuration use without validation.
12. `variant_axis_without_fixture`: Axis matrix missing happy, boundary, or negative fixtures.
13. `detector_without_negative_fixture`: Verifier checks lacking corresponding negative test triggers.
14. `witness_claim_without_hash_input`: Witnessing logic omitting configured layers or environments.
15. `cargo_package_without_clean_git`: Packaging with dirty unstaged work or uncommitted changes.
16. `dependency_order_not_publishable`: Circular references or workspace version mismatches.

---

## 4. Acceptance Criteria & Release Gate Output

The final packaging gate outputs the formal admission state of the release candidate:

### Successful Gate
```text
Release: star-toml v26.6.28

q_code:      1
q_docs:      1
q_api:       1
q_examples:  1
q_lsp:       1
q_publish:   1

failset_cardinality: 0

standing: ADMITTED
```

### Refused Gate
```text
Release: star-toml v26.6.28

q_code:      1
q_docs:      0
q_api:       1
q_examples:  1
q_lsp:       0
q_publish:   1

failset_cardinality: 2

standing: REFUSED

[1] Counterexample: lsp_feature_claim_without_handler
    Claim: "Document Formatting is implemented"
    Evidence: No document formatting provider found in server initialize block
    Repair: Mark "Deferred" in docs/LSP.md and docs/jira/v26.6.28/ST-110_error_topology_lsp.md

[2] Counterexample: doc_claim_not_backed_by_test
    Claim: "Unprefixed environment variables trigger hard admission failure"
    Evidence: tests/brce.rs asserts that unprefixed overrides are ignored and succeed
    Repair: Correct ST-105 specification to reflect ambient override ignore behavior
```

---

## 5. The Verification Ladder

The release passes through ten ascending verification levels:

```text
Level 0: Compiles                             → Workspace cargo check --all-targets passes
Level 1: Tests pass                           → cargo test suite passes (82 unit tests)
Level 2: Examples pass                        → All examples compile and execute successfully
Level 3: Counterexamples active               → All 23 verifier detectors execute and return PASS
Level 4: Docs claims mapped to tests          → Every README/JIRA assertion is backed by a test
Level 5: Public API mapped to lifecycle        → API surface is fully assigned to states
Level 6: LSP projected but not authoritative → LSP crate has zero dependency link to loader internals
Level 7: Package dry-run passes               → cargo package runs cleanly
Level 8: Clean git state                      → git status is clean without untracked changes
Level 9: Release verifier failset = 0         → release_verifier executes with empty failset
```

Only Level 9 candidates are admitted for Crates.io publication.
