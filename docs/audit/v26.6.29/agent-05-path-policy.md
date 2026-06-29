# Path Policy and PathWitness Audit Report

**Crate:** `star-toml`  
**Version:** `26.6.29`  
**Auditor:** Agent 05  
**Date:** 2026-06-28  

---

## Bound Index
- B surface inspected: `PathPolicy` (Sandbox, RelativeOnly, BlockForbidden), Null byte check, traversal limits, forbidden prefixes.
- O surface inspected: Config paths, source paths, filesystem state.
- μ surface inspected: `resolve_and_validate()`, `clean_path()`, `check_path_safe()`.
- C detectors inspected: `path_traversal_accepted`, `null_byte_path_accepted`, `source_relative_path_unresolved`, `forbidden_path_accepted`.
- W witnesses inspected: `PathWitness` struct schema and generation.
- q evidence found: `tests/brce.rs::test_path_traversal_fails`, `tests/brce.rs::test_null_byte_fails`, `tests/brce.rs::test_forbidden_path_fails`, `tests/brce.rs::test_relative_resolved_against_source_parent`, `tests/brce.rs::test_path_witness_emitted`, `tests/adversarial.rs::test_path_adversarial`.

---

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | Code in [src/path.rs](file:///Users/sac/star-toml/src/path.rs) checking null bytes, `..` traversals, and forbidden prefixes on Unix and Windows styles. | None. |
| O | Path string config values, `source_path` in `Validator` ([src/validation.rs](file:///Users/sac/star-toml/src/validation.rs)). | None. |
| O* | Validated config values when `check_path_safe` succeeds. | None. |
| μ | `resolve_and_validate()` and `clean_path()` in [src/path.rs](file:///Users/sac/star-toml/src/path.rs). | None. |
| A | `PathWitness` struct in [src/path.rs](file:///Users/sac/star-toml/src/path.rs) with `rejection_code` and `sandbox_root` fields. | None. |
| C | Verifier check 12, 13, 14, 22 in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs). | None. |
| W | `v.path_witnesses` list inside `Validator`. | None. |
| q | Standing proof is complete ($q = 1$ since failset is empty). | None. |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs) tests basic relative resolution and witness generation. | None. | High (Positive validation). |
| Falsification | [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs) tests standard/Windows traversal, null bytes, and forbidden paths. | None. | Critical (Vulnerabilities blocked). |
| Counterfactual | Verifier checks 12, 13, 14, 22 in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs). | None. | High. |
| Invariant | `PathWitness` struct matches fields. | None. | Critical (Ontology compliance). |
| Metamorphic | Workspace tests validating canonical stability. | None. | High. |
| Boundary | Blocklist check in `BlockForbidden` handles nested forbidden paths. | None. | High. |
| Conservation | Mapped provenance in `PathWitness`. | None. | Medium. |
| Determinism | `verifier_report` check 15/21. | None. | Medium. |
| Idempotence | Checked in tests. | None. | Low. |
| Replay | Sandbox root parameters recorded in `PathWitness`. | None. | High. |
| Provenance | `source_path` in `PathWitness`. | None. | Medium. |
| Witness | Witness generation code in `src/validation.rs`. | None. | High. |

---

## Failset

| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| **`path_traversal_accepted`** | Yes | Yes | Yes. Correctly catches Unix and Windows separator traversals (e.g. `foo\\..\\bar`) and resolves symlinks. | **PASS** |
| **`null_byte_path_accepted`** | Yes | Yes | Yes. Screened immediately in [src/path.rs](file:///Users/sac/star-toml/src/path.rs). | **PASS** |
| **`source_relative_path_unresolved`** | Yes | Yes | Yes. Source directories are resolved against absolute paths to preserve CWD-invariance. | **PASS** |
| **`forbidden_path_accepted`** | Yes | Yes | Yes. Blocked paths check nested components and absolute paths correctly. | **PASS** |
| **`PathWitness missing evidence fields`** | Yes | Yes | Yes. `PathWitness` contains `rejection_code` and `sandbox_root` to satisfy the ontology requirements. | **PASS** |

---

## Standing Decision

- **ALIVE**: All path policy checks, CWD invariance, Windows-style separator sanitization, and `PathWitness` ontology fields are fully implemented, verified, and passing in `v26.6.29`.
