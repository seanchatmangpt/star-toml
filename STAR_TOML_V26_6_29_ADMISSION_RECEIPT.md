# star-toml v26.6.29 — Full ST-JSON Admission Receipt

**Release**: v26.6.29 — Full ST-JSON Admission Release  
**Date**: 2026-06-28  
**Operator**: Sean Chatman <sean@chatmangpt.com>  
**Git tag**: v26.6.29  

---

## Standing Formula

```
q_v26_6_29 = q_st_json ∧ q_config_test ∧ q_wasm_export ∧ q_release_verifier ∧ q_publish_readiness
```

**Final standing: ADMITTED**

---

## Release Verifier Output (23/23 — no --dev flag)

| # | Check | Status |
|---|---|---|
| 1 | architecture_claim_without_invariant | PASS |
| 2 | public_api_without_example | PASS |
| 3 | public_api_without_lifecycle_position | PASS |
| 4 | doc_claim_not_backed_by_test | PASS |
| 5 | deferred_feature_marked_complete | PASS |
| 6 | implemented_claim_without_gate | PASS |
| 7 | lsp_feature_claim_without_handler | PASS |
| 8 | lsp_authority_boundary_violation | PASS |
| 9 | ocel_treated_as_standing_authority | PASS |
| 10 | raw_parse_treated_as_trusted | PASS |
| 11 | example_uses_non_admitted_path | PASS |
| 12 | variant_axis_without_fixture | PASS |
| 13 | detector_without_negative_fixture | PASS |
| 14 | witness_claim_without_hash_input | PASS |
| 15 | cargo_package_without_clean_git | PASS |
| 16 | dependency_order_not_publishable | PASS |
| 17 | version_alignment_drift | PASS |
| 18 | git_release_tag_missing | PASS |
| 19 | example_runtime_panic | PASS |
| 20 | docs_integrity_and_freshness | PASS |
| 21 | config_test_crate_present | PASS |
| 22 | config_test_suite_passes | PASS |
| 23 | json_schema_bridge_standing | PASS |

**Total: 23 / Passed: 23 / Failed: 0**

---

## Track Completion

### Track 1 — ST-JSON Release (q_st_json = 1)

- 20 DOC diagnostics (DOC-001 through DOC-020) with full metadata
- 8-field `ReceiptBinding` struct (`json_schema_digest`, `admitted_schema_digest`, `admitted_config_digest`, `documentation_model_digest`, `projection_digest`, `generator_version`, `timestamp`, `signer_or_operator`)
- `compute_model_digest()` excludes receipts to prevent bootstrapping cycle
- DOC-010 / DOC-011 / DOC-014 moved to CLI layer (filesystem checks removed from pure model validation)
- SCH-JSON-008 gate in `TrustedLoader::load_admitted()` via `require_json_schema_admission(bool)`
- 10-section release verifier with `--dev` flag and `ADMITTED`/`REFUSED` final standing
- Check 21 (`json_schema_bridge_standing`) runs `cargo test -p star-toml-json-schema --test bridge_tests`

### Track 2 — star-toml-config-test (q_config_test = 1)

- New crate: `star-toml-config-test` in root workspace
- Assertion helpers: `assert_schema_admitted`, `assert_schema_refused`, `assert_docs_fresh`, `assert_receipt_bound`, `assert_receipt_missing`, `assert_receipt_mismatch_detected`, `assert_doc_diagnostic`, `assert_no_doc_diagnostics`
- `CounterexampleFixture` runner and `DfCMAxis`/`run_dfcm_matrix` runner
- 10-category conformance suite in `tests/conformance.rs` — 10/10 tests pass
- Checks 22 (`config_test_crate_present`) and 23 (`config_test_suite_passes`) added to release verifier

### Track 3 — star-toml-wasm (q_wasm_export = 1)

- New crate: `star-toml-wasm` at `../star-toml-wasm/` — separate workspace (NOT in root Cargo.toml)
- 6 exported functions: `validate_schema`, `validate_config`, `validate_json_schema_bridge`, `generate_docs_metadata`, `verify_receipt`, `explain_diagnostic`
- All functions return structured `WasmResult` JSON (`status`, `diagnostics`, `digests`, `receipts`, `standing_predicates`)
- `[features] wasm = ["wasm-bindgen"]` gates the WASM ABI so native `cargo test` always works
- 11/11 native equivalence smoke tests pass (`tests/smoke.rs`)
- Native tests verified: schema admit/refuse, config admit/refuse, docs metadata, receipt verify, explain diagnostic

**Known deferrals (Track 3 Phase 2)**:
- JS/Python WASM host tests after `wasm-pack build`
- `wasm-pack` not required for this receipt — native equivalence tests satisfy `q_wasm_export`

---

## Digest Binding

| Artifact | Digest |
|---|---|
| Git tag | v26.6.29 |
| star-toml version | 26.6.29 |
| star-toml-derive version | 26.6.29 |
| star-toml-json-schema version | 26.6.29 |
| star-toml-lsp version | 26.6.29 |
| star-toml-config-test version | 26.6.29 |
| star-toml-wasm version | 26.6.29 |

---

## BRCE Satisfaction

```
q_config = 1  iff  BoundSatisfied ∧ TransformLawful ∧ WitnessComplete ∧ CounterexamplesAbsent
```

- **BoundSatisfied**: Release verifier 23/23 — all authority boundary, override policy, and receipt checks pass
- **TransformLawful**: DOC model determinism test passes; `compute_model_digest()` is cycle-free
- **WitnessComplete**: `check_14` (`witness_claim_without_hash_input`) PASS
- **CounterexamplesAbsent**: DOC-006, DOC-007 checks enforce counterexample coverage in model; `check_13` (`detector_without_negative_fixture`) PASS

**q_v26_6_29 = 1 — ADMITTED**
