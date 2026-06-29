# JIRA Epic Alignment: ST-JSON-DoD-ALIGN — Full DoD Specifications for v26.6.29

This JIRA ticket outlines the final specifications, diagnostics metadata, cryptographic receipt bindings, verifier output structures, and the 19-axis DfCM matrix required to satisfy the **v26.6.29 Full ST-JSON Admission Release** Definition of Done.

---

## 1. Full Diagnostic Metadata Matrix

Every diagnostic must expose code, name, severity, trigger, repair hint, negative fixture, test coverage, and release-verifier gate mapping:

### A. DOC-001 through DOC-020 Diagnostics Table

| Code | Name | Severity | Trigger | Repair Hint | Negative Fixture / Test | Verifier Gate |
|---|---|---|---|---|---|---|
| **DOC-001** | `ConfigDomainWithoutGeneratedDocs` | Fatal | Stable domain declared in schema is absent from docs model. | Register the domain in `DocModelBuilder`. | `tests/docs_diagnostics.rs::test_domain_missing` | Check 20: Docs Freshness |
| **DOC-002** | `ConfigFieldWithoutDescription` | Error | A configuration field lacks a description metadata string. | Add a `.description()` attribute in field declaration. | `tests/docs_diagnostics.rs::test_missing_description` | Check 20: Docs Freshness |
| **DOC-003** | `ConfigFieldWithoutValidationRule` | Error | A required configuration field has no validation rules defined. | Define numeric or type bounds for the field. | `tests/docs_diagnostics.rs::test_missing_rule` | Check 20: Docs Freshness |
| **DOC-004** | `ConfigFieldWithoutCounterexample` | Error | Validator ref fails to map to a stable counterexample code. | Link the validator to a `failure_counterexamples` code. | `tests/docs_diagnostics.rs::test_missing_counterexample` | Check 20: Docs Freshness |
| **DOC-005** | `OverridePolicyWithoutDocumentation` | Error | Override policy lacks detailed documentation metadata. | Provide tracking, reason, and scope rules. | `tests/docs_diagnostics.rs::test_override_undocumented` | Check 20: Docs Freshness |
| **DOC-006** | `ForbiddenValueUndocumented` | Warning | Field has forbidden values but description does not list them. | Mention forbidden values in the field's description. | `tests/docs_diagnostics.rs::test_forbidden_undocumented` | Check 20: Docs Freshness |
| **DOC-007** | `DefaultValueUndocumented` | Warning | Field has default value but description does not explain it. | Add default value details to the field description. | `tests/docs_diagnostics.rs::test_default_undocumented` | Check 20: Docs Freshness |
| **DOC-008** | `ReceiptPolicyUndocumented` | Error | The domain requires a receipt policy but its rules are missing. | Define receipt verification rules for the domain. | `tests/docs_diagnostics.rs::test_receipt_undocumented` | Check 20: Docs Freshness |
| **DOC-009** | `AuthorityBoundaryUndocumented` | Error | Boundary rules ("may control"/"must not claim") are empty. | Add allowed control scopes inside boundary metadata. | `tests/docs_diagnostics.rs::test_boundary_undocumented` | Check 20: Docs Freshness |
| **DOC-010** | `GeneratedDocsOutOfDate` | Fatal | Output Markdown/JSON reference files on disk are stale. | Run the `star_toml_docs export` command to sync. | `tests/docs_diagnostics.rs::test_docs_stale` | Check 20: Docs Freshness |
| **DOC-011** | `ManualDocsClaimStanding` | Fatal | Markdown reference has been modified directly without builder. | Do not manually edit reference files. Generate them. | `tests/docs_diagnostics.rs::test_manual_edits` | Check 20: Docs Freshness |
| **DOC-012** | `DocsGeneratedFromUnadmittedFacts` | Fatal | Fact source used in model has failed config admission. | Ensure configuration stands before building docs. | `tests/docs_diagnostics.rs::test_unadmitted_facts` | Check 20: Docs Freshness |
| **DOC-013** | `DocumentationModelNondeterministic` | Fatal | Fields/domains arrays are not sorted alphabetically. | Invoke `.canonicalize()` before computing model digest. | `tests/docs_diagnostics.rs::test_nondeterministic` | Check 20: Docs Freshness |
| **DOC-014** | `StableDomainMissingMachineReadableExport` | Fatal | JSON metadata file is absent from the target path. | Ensure JSON exporter generates `config_metadata.json`. | `tests/docs_diagnostics.rs::test_missing_json` | Check 20: Docs Freshness |
| **DOC-015** | `ValidatorDocumentedButNotEnforced` | Fatal | Validator is documented but has no active validation logic. | Add code checking the validator in `Validate` implementation. | `tests/docs_diagnostics.rs::test_unenforced_validator` | Check 20: Docs Freshness |
| **DOC-016** | `CounterexampleDocumentedButNotTested` | Fatal | Counterexample has no matching active test fixture. | Add a unit test triggering the counterexample. | `tests/docs_diagnostics.rs::test_untested_counter` | Check 20: Docs Freshness |
| **DOC-017** | `OverrideDocumentedButNotEnforced` | Fatal | Override policy has no corresponding loader enforcement. | Verify overrides scope during parsing. | `tests/docs_diagnostics.rs::test_unenforced_override` | Check 20: Docs Freshness |
| **DOC-018** | `AuthorityBoundaryDocumentedButNotEnforced` | Fatal | Authority boundary is documented but has no loader code check. | Assert boundary control limits in loader phase. | `tests/docs_diagnostics.rs::test_unenforced_boundary` | Check 20: Docs Freshness |
| **DOC-019** | `ProjectionTargetMissingRequiredDomain` | Fatal | Projection targets refer to domains not in the schema. | Ensure templates match registered domains. | `tests/docs_diagnostics.rs::test_missing_domain_ref` | Check 20: Docs Freshness |
| **DOC-020** | `ReceiptDigestMismatch` | Fatal | Receipt signature digest fails to match model hashes. | Re-generate documentation receipt mapping. | `tests/docs_diagnostics.rs::test_receipt_mismatch` | Check 20: Docs Freshness |

---

### B. SCH-JSON-001 through SCH-JSON-016 Diagnostics Table

| Code | Name | Severity | Trigger | Repair Hint | Negative Fixture / Test | Verifier Gate |
|---|---|---|---|---|---|---|
| **SCH-JSON-001** | `JsonSchemaFeatureDisabled` | Fatal | JSON Schema bridge is disabled but a JSON source is loaded. | Enable the `json-schema` feature flag. | `tests/bridge_tests.rs::test_feature_disabled` | Check 21: JSON Schema Standing |
| **SCH-JSON-002** | `JsonSchemaParseFailed` | Fatal | JSON Schema syntax contains parse or formatting errors. | Check JSON syntax and schema structures. | `tests/bridge_tests.rs::test_invalid_schema_parse` | Check 21: JSON Schema Standing |
| **SCH-JSON-003** | `JsonSchemaUnsupportedKeyword` | Error | Unsupported keyword (e.g. `patternProperties`) is found. | Restrict schema mapping to supported elements. | `tests/bridge_tests.rs::test_unsupported_keyword` | Check 21: JSON Schema Standing |
| **SCH-JSON-004** | `JsonSchemaReferenceUnresolved` | Error | Schema references a remote `$ref` target that is missing. | Verify references resolve locally. | `tests/bridge_tests.rs::test_reference_unresolved` | Check 21: JSON Schema Standing |
| **SCH-JSON-005** | `JsonSchemaNondeterministicImport` | Error | Fields are imported in a nondeterministic order. | Use a sorted map wrapper during ingestion. | `tests/bridge_tests.rs::test_nondeterministic` | Check 21: JSON Schema Standing |
| **SCH-JSON-006** | `JsonSchemaMissingDescriptionsForStableDomain` | Error | Stable domain or properties are missing descriptions. | Add description tags to the JSON Schema. | `tests/bridge_tests.rs::test_missing_description` | Check 21: JSON Schema Standing |
| **SCH-JSON-007** | `JsonSchemaAllowsUnboundedAdditionalProperties` | Error | `additionalProperties` is omitted or set to `true`. | Set `additionalProperties` to `false` in schema. | `tests/bridge_tests.rs::test_allows_unbounded_additional_properties` | Check 21: JSON Schema Standing |
| **SCH-JSON-008** | `ConfigValidatedAgainstUnadmittedJsonSchema` | Fatal | Configuration evaluated against unadmitted JSON Schema. | Pass the schema through the admission pipeline first. | `tests/bridge_tests.rs::test_unadmitted_validation` | Check 21: JSON Schema Standing |
| **SCH-JSON-009** | `DocsGeneratedFromUnadmittedJsonSchema` | Fatal | Documentation built from unadmitted JSON Schema facts. | Admit the schema candidate before documentation. | `tests/bridge_tests.rs::test_unadmitted_generation` | Check 21: JSON Schema Standing |
| **SCH-JSON-010** | `JsonSchemaDigestMismatch` | Fatal | JSON Schema digest does not match receipt. | Re-generate documentation and receipt binding. | `tests/bridge_tests.rs::test_digest_mismatch` | Check 21: JSON Schema Standing |
| **SCH-JSON-011** | `JsonSchemaExportLosesRequiredStandingFact` | Warning | Exporting schema loses required standing facts. | Restrict features or acknowledge lossy export. | `tests/bridge_tests.rs::test_valid_schema_import_and_export` | Check 21: JSON Schema Standing |
| **SCH-JSON-012** | `JsonSchemaCompositionAmbiguous` | Error | Composition keywords (`oneOf`/`anyOf`) overlap. | Clarify composition rules to avoid ambiguity. | `tests/bridge_tests.rs::test_composition_ambiguous` | Check 21: JSON Schema Standing |
| **SCH-JSON-013** | `JsonSchemaDefaultUndocumented` | Warning | Defaults are defined but missing description explanations. | Mention defaults in the element description. | `tests/bridge_tests.rs::test_default_undocumented` | Check 21: JSON Schema Standing |
| **SCH-JSON-014** | `JsonSchemaConstraintWithoutCounterexample` | Error | Constraint lacks corresponding `x-counterexamples`. | Add `x-counterexamples` mappings to constraints. | `tests/bridge_tests.rs::test_constraint_without_counterexample` | Check 21: JSON Schema Standing |
| **SCH-JSON-015** | `JsonSchemaAuthorityBoundaryMissing` | Error | Stable domain lacks `x-authority-boundary` definition. | Add authority boundary metadata key to schema. | `tests/bridge_tests.rs::test_authority_boundary_missing` | Check 21: JSON Schema Standing |
| **SCH-JSON-016** | `JsonSchemaUnsupportedDialect` | Fatal | Schema version dialect is unsupported. | Use a supported dialect version (e.g. Draft 7). | `tests/bridge_tests.rs::test_unsupported_dialect` | Check 21: JSON Schema Standing |

---

## 2. Cryptographic Receipt Binding Specification

To satisfy **Predicate 5 (Full Receipt Binding)**, the `ReceiptBinding` structure must pack and verify these 8 digests:

```rust
pub struct ReceiptBinding {
    pub json_schema_digest: String,          // BLAKE3 of JSON Schema candidate (if applicable)
    pub admitted_schema_digest: String,      // BLAKE3 of star-toml admitted Schema
    pub admitted_config_digest: String,      // BLAKE3 of admitted Configuration instance
    pub documentation_model_digest: String,  // BLAKE3 of serializable ConfigDocumentationModel
    pub projection_digest: String,           // BLAKE3 of final Markdown reference text
    pub generator_version: String,           // star-toml crate release version
    pub operator_signer: String,             // Public identity/key of verification operator
    pub timestamp: String,                   // ISO 8601 UTC time
}
```

* **Gate Rule**: The `release_verifier` must compute this cryptographic chain during checks. If any digest mismatches the calculated local hashes, verification fails.

---

## 3. Unified Release-Verifier Reporting Sections

The `release_verifier` binary must output a structured verification checklist consisting of exactly these 10 sections:

```text
# star-toml Release Verifier Gate

[1]  JSON Schema Bridge Standing: [PASS/FAIL]
[2]  Schema Admission:            [PASS/FAIL]
[3]  Config Admission:            [PASS/FAIL]
[4]  Generated Docs Standing:     [PASS/FAIL]
[5]  Receipt Binding:             [PASS/FAIL]
[6]  Projection Freshness:        [PASS/FAIL]
[7]  DOC Diagnostics:             [PASS/FAIL]
[8]  SCH-JSON Diagnostics:        [PASS/FAIL]
[9]  DfCM Fixture Coverage:       [PASS/FAIL]
[10] Final Standing:              [ADMITTED/REFUSED]
```

* **Binary Verdict Enforcement**: There is no "partially admitted" state. If any section fails, final standing must be `REFUSED`.

---

## 4. Unified 19-Axis DfCM Fixture Matrix

The verification test suite must execute happy-path, boundary, and negative test fixtures across all 19 axes of the product capability:

| Axis | Happy Fixture | Boundary Fixture | Negative Fixture |
|---|---|---|---|
| **1. SchemaDialect** | Draft 7 compatible | Draft 6 fallback | Unsupported dialects |
| **2. ObjectShape** | Simple properties map | Nested object mapping | Non-object root |
| **3. PropertyCount** | Multiple fields mapped | 0-field empty object | Large combinatorial fields |
| **4. RequiredState** | Explicit required list | All optional fields | Required fields missing |
| **5. TypeShape** | Standard types mapped | Int/float conversions | Unknown type definitions |
| **6. EnumShape** | Enum value arrays | Single-item enum | Enum values type mismatch |
| **7. DefaultState** | Defaults documented | Default empty string | Default value type mismatch |
| **8. DescriptionState**| Fields have descriptions | Short summary metadata | Missing description fields |
| **9. AdditionalPropertiesState** | Explicitly false | Custom boundary declared | Unbounded additional fields |
| **10. ReferenceState**| Local ref resolved | Nested pointer resolved | Unresolved remote `$ref` |
| **11. CompositionState**| Type restrictions mapped | Complex types merged | Ambiguous `oneOf` overlaps |
| **12. ConstraintState** | Validation bounds defined| Min/Max limits | Constraints without codes |
| **13. AuthorityBoundaryState** | Boundary ID defined | Escalation rules defined | Missing boundary metadata |
| **14. DocumentationCoverage**| All domains documented | Empty config documented | Missing domain references |
| **15. DigestState** | Deterministic sorting | Stable canonical outputs | Digest hash mismatches |
| **16. ExportRepresentability**| Round-trip mappings | Complex constraints map | Lossy export warnings |
| **17. AdmissionState** | Schema stands before load | Schema overrides verified | Config loaded on unadmitted |
| **18. FreshnessState** | Disk files in sync | Empty file checks | Out-of-sync content drift |
| **19. ReceiptState** | Digests matching receipt | Dynamic signer timestamp| Receipt signature mismatch |
