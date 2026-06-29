# Jira Board for Star TOML v26.6.29 (Hardening & Documentation Release)

This directory contains the formal Jira tickets outlining the requirements, acceptance criteria, and verification methods for transitioning **star-toml** into a fully audited config admission engine with deterministic generated documentation.

## JIRA Board Roadmap (v26.6.29)

| Ticket ID | Type | Title | Summary |
|---|---|---|---|
| **[ST-112](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-112_release_standing.md)** | Story | Release Standing Gate | Integrates a global `release_verifier` enforcing the 16 release gate counterexamples over the codebase. |
| **[ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md)** | Story | Generated Configuration Documentation | Defines the stable documentation model, fact ingestor, projection engines, freshness checkers, and 20 new refusal conditions (`DOC-001` - `DOC-020`). |
| **[ST-JSON](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md)** | Epic | JSON Schema Crate Bridge | Implements the optional JSON Schema candidate importer, exporter, bridge standing validation, and 16 refusal conditions (`SCH-JSON-001` to `SCH-JSON-016`). |
| **[ST-JSON-ALIGN](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON-DoD_Alignment.md)** | Ticket | Epic Alignment to DoD | Maps v26.6.29 Full DoD definitions, diagnostics, receipts, and DfCM axes. |
| **[INSTRUCTIONS](file:///Users/sac/star-toml/docs/jira/v26.6.29/IMPLEMENTATION_INSTRUCTIONS.md)** | Guide | Agent Implementation Instructions | Comprehensive instructions and LOC gap mapping for the incoming development agent. |

---

## Traceability Matrix: Documentation Counterexamples to Jira Tickets

The **Documentation Counterexamples Set** defines the release standing criteria for documentation validity. The following matrix shows which Jira tickets specify the safety rules to mitigate each failure condition:

| Counterexample Name | Danger | Mitigated by Ticket | Verification Phase |
|---|---|---|---|
| `DOC-001 ConfigDomainWithoutGeneratedDocs` | Stable domain undocumented | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Release Verifier Gate |
| `DOC-002 ConfigFieldWithoutDescription` | Config field missing description | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Ingestion Validation |
| `DOC-003 ConfigFieldWithoutValidationRule` | Field validation undocumented | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Ingestion Validation |
| `DOC-004 ConfigFieldWithoutCounterexample` | Validation rule lacks error code | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Ingestion Validation |
| `DOC-005 OverridePolicyWithoutDocumentation` | Override policy undocumented | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Loader Settings Audit |
| `DOC-006 ForbiddenValueUndocumented` | Forbidden value missing from docs | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Schema Metadata Audit |
| `DOC-007 DefaultValueUndocumented` | Field default missing from docs | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Schema Metadata Audit |
| `DOC-008 ReceiptPolicyUndocumented` | Receipt requirements undocumented | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Release Verifier Gate |
| `DOC-009 AuthorityBoundaryUndocumented` | Domain authority boundaries undocumented | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Domain Boundary Audit |
| `DOC-010 GeneratedDocsOutOfDate` | Documentation drift / stale files | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Freshness Check |
| `DOC-011 ManualDocsClaimStanding` | Documentation edited directly/manually | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Freshness & Provenance Check |
| `DOC-012 DocsGeneratedFromUnadmittedFacts` | Documentation built from unverified configs | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Ingestion Validation |
| `DOC-013 DocumentationModelNondeterministic` | Inconsistent file formats or hashes | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Determinism Audit |
| `DOC-014 StableDomainMissingMachineReadableExport`| JSON metadata file absent | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Release Verifier Gate |
| `DOC-015 ValidatorDocumentedButNotEnforced` | Validator documented but lacks code enforcement | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Runtime Validation Alignment |
| `DOC-016 CounterexampleDocumentedButNotTested` | Counterexample lacks active negative test fixture | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Integration & Fixture Audit |
| `DOC-017 OverrideDocumentedButNotEnforced` | Override policy missing active code/test check | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Invariant Verification |
| `DOC-018 AuthorityBoundaryDocumentedButNotEnforced`| Boundaries lack code checks | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Invariant Verification |
| `DOC-019 ProjectionTargetMissingRequiredDomain` | Projection templates reference invalid domains | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Template Validation |
| `DOC-020 ReceiptDigestMismatch` | Receipt digest does not match output digests | [ST-113](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-113_generated_documentation.md) | Receipt Verification |

---

## Traceability Matrix: JSON Schema Diagnostics to Epic Stories

The following matrix maps the `SCH-JSON` diagnostics to their respective JIRA ticket rules:

| Diagnostic Code | Diagnostic Name | Mitigated by Ticket | Verification Phase |
|---|---|---|---|
| `SCH-JSON-001` | `JsonSchemaFeatureDisabled` | [ST-JSON-001](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Feature Gate Check |
| `SCH-JSON-002` | `JsonSchemaParseFailed` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Importer Parser |
| `SCH-JSON-003` | `JsonSchemaUnsupportedKeyword` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Keyword Validation |
| `SCH-JSON-004` | `JsonSchemaReferenceUnresolved` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Reference Resolver |
| `SCH-JSON-005` | `JsonSchemaNondeterministicImport` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Determinism Audit |
| `SCH-JSON-006` | `JsonSchemaMissingDescriptionsForStableDomain` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Metadata Audit |
| `SCH-JSON-007` | `JsonSchemaAllowsUnboundedAdditionalProperties` | [ST-JSON-006](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Unbounded Check |
| `SCH-JSON-008` | `ConfigValidatedAgainstUnadmittedJsonSchema` | [ST-JSON-002](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Standing Check |
| `SCH-JSON-009` | `DocsGeneratedFromUnadmittedJsonSchema` | [ST-JSON-009](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Standing Check |
| `SCH-JSON-010` | `JsonSchemaDigestMismatch` | [ST-JSON-010](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Digest Verification |
| `SCH-JSON-011` | `JsonSchemaExportLosesRequiredStandingFact` | [ST-JSON-012](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Export Warning |
| `SCH-JSON-012` | `JsonSchemaCompositionAmbiguous` | [ST-JSON-012](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Composition Warning |
| `SCH-JSON-013` | `JsonSchemaDefaultUndocumented` | [ST-JSON-003](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Default Audit |
| `SCH-JSON-014` | `JsonSchemaConstraintWithoutCounterexample` | [ST-JSON-005](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Counterexample Verification |
| `SCH-JSON-015` | `JsonSchemaAuthorityBoundaryMissing` | [ST-JSON-007](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Boundary Verification |
| `SCH-JSON-016` | `JsonSchemaUnsupportedDialect` | [ST-JSON-013](file:///Users/sac/star-toml/docs/jira/v26.6.29/ST-JSON_json_schema_bridge.md) | Dialect Verification |

---

## 🏛️ v26.6.29 Implementation Gaps & Required LOC Modifications

The following matrix maps the identified code gaps to the exact files and lines of code (LOC) that must be modified to achieve full ADMITTED status:

### 1. Documentation Diagnostics Audit Expansion
* **Target File**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs)
* **Target LOC**: Under `validate_docs_integrity` at [`src/docs.rs#L241-L369`](file:///Users/sac/star-toml/src/docs.rs#L241-L369).
* **Required Modifications**:
  * **`DOC-008` (ReceiptPolicyUndocumented)**: Add validation for `domain.receipt_policy` properties.
  * **`DOC-010` (GeneratedDocsOutOfDate)**: Instantiate this check dynamically comparing generated projection digests with file digests inside the validator function.
  * **`DOC-011` (ManualDocsClaimStanding)**: Assert that files on disk contain a valid `ReceiptBinding` header signature matching calculated model state.
  * **`DOC-012` (DocsGeneratedFromUnadmittedFacts)**: Track source configuration admission status inside the ingestor model and flag unadmitted facts.
  * **`DOC-014` (StableDomainMissingMachineReadableExport)**: Verify that the machine-readable JSON metadata file exists at the expected workspace location.
  * **`DOC-015` (ValidatorDocumentedButNotEnforced)**: Compare documented validators in `domain.validators` against runtime validation definitions inside `src/validation.rs`.
  * **`DOC-016` (CounterexampleDocumentedButNotTested)**: Assert that every counterexample code has a matching unit/integration test structure or metadata declaration.
  * **`DOC-017` (OverrideDocumentedButNotEnforced)** & **`DOC-018` (AuthorityBoundaryDocumentedButNotEnforced)**: Audit runtime boundaries defined in `src/loader.rs` against documented metadata.
  * **`DOC-019` (ProjectionTargetMissingRequiredDomain)**: Ensure the template contains only valid domains declared in the configuration schema.
  * **`DOC-020` (ReceiptDigestMismatch)**: Validate BLAKE3 signature mismatch inside the validator return.

### 2. JSON Schema Bridge Integration into Release Verifier
* **Target File**: [`src/bin/release_verifier.rs`](file:///Users/sac/star-toml/src/bin/release_verifier.rs)
* **Target LOC**: Around `docs_integrity_and_freshness` at [`src/bin/release_verifier.rs#L660-L684`](file:///Users/sac/star-toml/src/bin/release_verifier.rs#L660-L684).
* **Required Modifications**:
  * Implement check 21 (`json_schema_bridge_standing`) calling `star_toml_json_schema::validate_json_schema_standing` on imported schema candidates, verifying that no unadmitted JSON Schema shapes claim standing.

### 3. Dynamic Discovery and Receipt Generation
* **Target File**: [`src/bin/star_toml_docs.rs`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs)
* **Target LOC**:
  * [`src/bin/star_toml_docs.rs#L13-L41`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs#L13-L41) (`system_config_schema` definition)
  * [`src/bin/star_toml_docs.rs#L77-L85`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs#L77-L85) (static `ReceiptBinding` instantiation)
* **Required Modifications**:
  * Implement dynamic schema discovery (scanning schema directories or config TOML folders) rather than hardcoding the `system` schema.
  * Read the operator identity, signer credentials, and dynamic timestamps from system environment/context instead of utilizing hardcoded constants.

### 4. Full Schema Admission Gates & Refusal Enforcement
* **Target File**: [`src/loader.rs`](file:///Users/sac/star-toml/src/loader.rs)
* **Target LOC**: Inside `load_admitted_strict` at [`src/loader.rs#L121-L150`](file:///Users/sac/star-toml/src/loader.rs#L121-L150).
* **Required Modifications**:
  * Assert schema admission status and digest matching against candidate schemas prior to validating configurations against them (enforcing `ConfigValidatedAgainstUnadmittedJsonSchema` / `SCH-JSON-008`).
* **Target File**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs)
* **Target LOC**: Inside `DocModelBuilder::build` at [`src/docs.rs#L220-L240`](file:///Users/sac/star-toml/src/docs.rs#L220-L240).
* **Required Modifications**:
  * Prevent documentation compilation if the schema candidate did not pass schema admission (enforcing `DocsGeneratedFromUnadmittedJsonSchema` / `SCH-JSON-009`).

### 5. Documentation Engine Coverage Checks
* **Target File**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs)
* **Target LOC**: Inside `validate_docs_integrity` at [`src/docs.rs#L241-L369`](file:///Users/sac/star-toml/src/docs.rs#L241-L369).
* **Required Modifications**:
  * Implement explicit assertions for:
    * `DomainCoverage` (all schema domains documented)
    * `FieldCoverage` (all domain fields documented)
    * `ValidatorCoverage` (all field constraints mapped to a documented validator)
    * `CounterexampleCoverage` (all validator failure paths linked to a documented counterexample)
    * `OverrideCoverage` (all override policies documented)
    * `AuthorityBoundaryCoverage` (all domain boundaries documented)

### 6. Enriched Diagnostic Structures
* **Target File**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs)
* **Target LOC**: Struct `DocDiagnostic` definition at [`src/docs.rs#L231-L238`](file:///Users/sac/star-toml/src/docs.rs#L231-L238).
* **Required Modifications**:
  * Add the following missing metadata fields: `trigger`, `negative_fixture`, `test_coverage`, and `release_verifier_coverage_if_fatal`.
* **Target File**: [`star-toml-json-schema/src/diagnostics.rs`](file:///Users/sac/star-toml/star-toml-json-schema/src/diagnostics.rs)
* **Target LOC**: Struct `SchJsonDiagnostic` definition at [`star-toml-json-schema/src/diagnostics.rs#L3-L10`](file:///Users/sac/star-toml/star-toml-json-schema/src/diagnostics.rs#L3-L10).
* **Required Modifications**:
  * Add fields: `negative_fixture`, `test_coverage`, and `release_verifier_coverage_if_fatal`.

### 7. Cryptographic Receipt Binding Fields
* **Target File**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs)
* **Target LOC**: Struct `ReceiptBinding` definition at [`src/docs.rs#L138-L147`](file:///Users/sac/star-toml/src/docs.rs#L138-L147).
* **Required Modifications**:
  * Expand struct to explicitly bind:
    * `json_schema_digest: String`
    * `admitted_schema_digest: String`
    * `admitted_config_digest: String`
    * `documentation_model_digest: String`
    * `projection_digest: String`

### 8. Full Projection Support
* **Target File**: [`src/bin/star_toml_docs.rs`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs)
* **Target LOC**: Inside `export` block at [`src/bin/star_toml_docs.rs#L89-L108`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs#L89-L108).
* **Required Modifications**:
  * Add code to generate and write the exported JSON Schema definition files to `docs/schema.json` during documentation export.

### 9. Unified Release-Verifier Reporting Sections
* **Target File**: [`src/bin/release_verifier.rs`](file:///Users/sac/star-toml/src/bin/release_verifier.rs)
* **Target LOC**: Inside the output report generator at [`src/bin/release_verifier.rs#L50-L150`](file:///Users/sac/star-toml/src/bin/release_verifier.rs#L50-L150).
* **Required Modifications**:
  * Structure report output to show pass/fail verdicts for all ten DoD sections:
    1. JSON Schema Bridge Standing
    2. Schema Admission
    3. Config Admission
    4. Generated Docs Standing
    5. Receipt Binding
    6. Projection Freshness
    7. DOC Diagnostics
    8. SCH-JSON Diagnostics
    9. DfCM Fixture Coverage
    10. Final Standing (Must strictly resolve to `ADMITTED` or `REFUSED`).

### 10. Complete DfCM Fixtures coverage for DOC and SCH-JSON codes
* **Target File**: [`tests/docs_diagnostics.rs`](file:///Users/sac/star-toml/tests/docs_diagnostics.rs) & [`star-toml-json-schema/tests/bridge_tests.rs`](file:///Users/sac/star-toml/star-toml-json-schema/tests/bridge_tests.rs).
* **Required Modifications**:
  * Append negative test runner functions triggering every single diagnostic code from `DOC-001` to `DOC-020` and `SCH-JSON-001` to `SCH-JSON-016` (currently, many codes are defined but lack execution coverage in unit tests).


