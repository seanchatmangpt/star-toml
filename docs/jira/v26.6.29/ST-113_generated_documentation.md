# ST-113: Generated Configuration Documentation

## Description
This ticket defines the requirements, acceptance criteria, data models, and verification methods for the Star TOML **Generated Configuration Documentation** engine. Under the **Vision 2030** design discipline, configuration documentation is not handwritten narrative authority; it is a deterministic projection of admitted configuration law. A configuration domain without fresh, generated documentation fails release-standing.

---

## The DfCM Documentation Law

The mathematical formula governing configuration documentation standing is:

$$q_{docs} = 1 \iff \text{DomainCoverage} \land \text{FieldCoverage} \land \text{ValidatorCoverage} \land \text{CounterexampleCoverage} \land \text{OverrideCoverage} \land \text{AuthorityBoundaryCoverage} \land \text{Freshness} \land \text{Determinism} \land \text{ReceiptBinding}$$

Where:
* **Freshness** is satisfied when:
  $$\text{documentation\_model\_digest} == \text{digest}(\text{project}(\text{admitted\_config\_facts}, \text{export\_version}))$$
* **Determinism** is satisfied when identical inputs, schema metadata, export version, and projection target produce bit-identical output digests.

If $q_{docs} = 0$, then release-standing is refused:

$$\text{StaleDocs} \lor \text{MissingCoverage} \lor \text{UnadmittedFacts} \lor \text{ReceiptMismatch} \implies q_{docs} = 0 \implies q_{release} = 0$$

---

## 1. Data Model Mapping (Rust Structs)

To satisfy **P0.1**, the system defines the documentation data model in `src/docs.rs` mapped to the following Rust types:

### A. Config Documentation Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDocumentationModel {
    pub version: String,
    pub generated_at: String,
    pub input_digest: String,      // BLAKE3 hash of source configuration schema + facts
    pub model_digest: String,      // BLAKE3 hash of the serialized domains, diagnostics, and receipts
    pub domains: Vec<ConfigDomain>,
    pub diagnostics: Vec<String>,  // Stable diagnostic codes from validation
    pub receipts: Vec<ReceiptBinding>,
}
```

### B. Config Domain
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDomain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub authority_boundary: AuthorityBoundary,
    pub fields: Vec<ConfigField>,
    pub validators: Vec<ConfigValidator>,
    pub counterexamples: Vec<Counterexample>,
    pub override_policy: OverridePolicy,
    pub receipt_binding: Option<ReceiptBinding>,
    pub stability: String,         // e.g. "stable", "experimental", "deprecated"
}
```

### C. Config Field
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub id: String,
    pub domain_id: String,
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub default: Option<String>,
    pub allowed_values: Vec<String>,
    pub forbidden_values: Vec<String>,
    pub description: String,
    pub validator_refs: Vec<String>,
    pub counterexample_refs: Vec<String>,
    pub override_policy_ref: Option<String>,
    pub source_binding: String,
    pub stability: String,
}
```

### E. Config Validator
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidator {
    pub id: String,
    pub name: String,
    pub predicate: String,
    pub description: String,
    pub applies_to: Vec<String>,   // List of field IDs
    pub failure_counterexamples: Vec<String>, // List of counterexample codes
    pub severity: String,          // e.g., "Warning", "Error", "Fatal"
}
```

### F. Counterexample
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    pub code: String,              // e.g., "DOC-001"
    pub name: String,
    pub description: String,
    pub severity: String,
    pub detection_rule: String,
    pub repair_hint: String,
}
```

### G. Authority Boundary
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityBoundary {
    pub id: String,
    pub domain: String,
    pub may_control: Vec<String>,
    pub must_not_claim: Vec<String>,
    pub defer_to: Option<String>,
    pub escalation_rule: Option<String>,
}
```

### H. Override Policy
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePolicy {
    pub id: String,
    pub allowed: bool,
    pub required_reason: bool,
    pub required_scope: Vec<String>,
    pub required_expiration: Option<String>,
    pub required_tracking: Option<String>,
    pub required_replacement_plan: bool,
    pub refusal_conditions: Vec<String>,
}
```

### I. Receipt Binding
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptBinding {
    pub admitted_config_digest: String,
    pub documentation_model_digest: String,
    pub projection_digest: String,
    pub generator_version: String,
    pub timestamp: String,
    pub signer_or_operator: String,
}
```

---

## 2. Core Architectural System Components

### 2.1 Config Fact Ingestor & Model Builder
* **Fact Ingestor**: Reads configuration schemas defined dynamically via `Schema` ([src/schema.rs](file:///Users/sac/star-toml/src/schema.rs)) or compiled statically via `#[derive(Validate, Documented)]` macros in [star-toml-derive/src/lib.rs](file:///Users/sac/star-toml/star-toml-derive/src/lib.rs).
* **Model Builder**: Aggregates fields, validators, defaults, override policies, and boundaries, sorting elements alphabetically by ID to enforce **absolute determinism**.

### 2.2 Documentation Validator
* Executes internal audits on the generated documentation model before publication.
* Detects missing descriptions, unlinked validation rules, or undocumented counterexamples, returning errors using stable `DOC-` diagnostics.

### 2.3 Projection Engine
* **Human-Readable Export**: Deterministically projects the model into standard Markdown documentation (`docs/config_reference.md`).
* **Machine-Readable Export**: Serializes the documentation model into JSON (`docs/config_metadata.json`) to fulfill **P0.3**.
* **RDF/Ontology Export**: Formats metadata into Turtle syntax matching public vocabularies.

### 2.4 Receipt Binder
* Generates `ReceiptBinding` packaging:
  $$\text{ReceiptDigest} = \text{BLAKE3}(\text{admitted\_config\_facts} \mathbin{\Vert} \text{model\_digest} \mathbin{\Vert} \text{projection\_digest})$$
* Compiles the receipt block into the header of the exported reference Markdown.

### 2.5 Release Verifier Integration
* Integrates `DOC-001` through `DOC-020` into the `release_verifier` tool ([src/bin/release_verifier.rs](file:///Users/sac/star-toml/src/bin/release_verifier.rs)).
* Fails release checks if the documentation is missing, stale, incomplete, or contains unadmitted facts.

---

## 3. Refusal Conditions Matrix (DOC-001 - DOC-020)

| Diagnostic Code | Counterexample Name | Refusal Condition / Trigger | Repair & Mitigation Steps |
|-----------------|---------------------|-----------------------------|---------------------------|
| **DOC-001** | `ConfigDomainWithoutGeneratedDocs` | Stable configuration domain is declared in schema but does not appear in the documentation model. | Register the domain and run the projection builder to generate documentation. |
| **DOC-002** | `ConfigFieldWithoutDescription` | A field is declared without a description string or with an empty description. | Add a description attribute/method call to the field declaration. |
| **DOC-003** | `ConfigFieldWithoutValidationRule` | A field is declared without documentation of its validation rules. | Document the required validation constraints or type bounds for the field. |
| **DOC-004** | `ConfigFieldWithoutCounterexample` | A documented field validation rule fails to map to a stable counterexample code. | Link the validation rule to a corresponding `DOC-` or validation error code. |
| **DOC-005** | `OverridePolicyWithoutDocumentation` | An override policy is defined but its allowed scope, tracking requirements, or expiration rules are undocumented. | Provide documentation for the override policy details in the loader settings. |
| **DOC-006** | `ForbiddenValueUndocumented` | A field contains a list of forbidden values in its validation logic, but this list is missing from the documentation model. | Update the field metadata in the schema/struct definition to declare forbidden values. |
| **DOC-007** | `DefaultValueUndocumented` | A field has an operational default value that is not documented. | Declare the default value explicitly using metadata attributes. |
| **DOC-008** | `ReceiptPolicyUndocumented` | The domain requires a receipt policy but its parameters are undocumented. | Define and document the receipt policy parameters for the domain. |
| **DOC-009** | `AuthorityBoundaryUndocumented` | An authority boundary is defined for the domain but its rules ("may control" / "must not claim") are undocumented. | Document the explicit authority boundaries and escalation rules for the domain. |
| **DOC-010** | `GeneratedDocsOutOfDate` | The generated documentation files (Markdown or JSON) do not match the current schema or validator structures. | Re-run the documentation generator tool to synchronize the documentation with source code. |
| **DOC-011** | `ManualDocsClaimStanding` | Documentation is manually authored or edited directly without being backed by the deterministic fact ingestor. | Remove manual overrides and generate documentation from admitted facts. |
| **DOC-012** | `DocsGeneratedFromUnadmittedFacts` | The source facts used to build the documentation model failed verification checks or validation rules. | Correct the configuration source facts so they are successfully admitted before documenting. |
| **DOC-013** | `DocumentationModelNondeterministic` | The model serializer outputs varying layouts, non-sorted arrays, or inconsistent hashes for the same inputs. | Implement deterministic sorting (sorting domains, fields, and validators alphabetically by ID). |
| **DOC-014** | `StableDomainMissingMachineReadableExport` | A stable configuration domain lacks a corresponding JSON metadata output file. | Verify that the JSON metadata exporter runs during documentation projection. |
| **DOC-015** | `ValidatorDocumentedButNotEnforced` | A validator listed in the documentation is missing from the active runtime validation suite. | Implement the corresponding validation check in the `Validate` implementation. |
| **DOC-016** | `CounterexampleDocumentedButNotTested` | A counterexample documented in the model does not have a corresponding test case or negative fixture. | Add a unit test or integration fixture in `tests/` that triggers this counterexample. |
| **DOC-017** | `OverrideDocumentedButNotEnforced` | An override policy is documented but no runtime logic or test asserts override restrictions. | Add tests enforcing the override constraints during parsing. |
| **DOC-018** | `AuthorityBoundaryDocumentedButNotEnforced` | Authority boundaries are documented but lack active verification checks during config loading. | Implement runtime boundary validation checks. |
| **DOC-019** | `ProjectionTargetMissingRequiredDomain` | A target projection template refers to a domain that is absent from the schema. | Ensure all documented domains are correctly defined in the schema. |
| **DOC-020** | `ReceiptDigestMismatch` | The digest contained within the documentation receipt does not match the actual compiled digests. | Re-generate the documentation receipt to align the digests. |

---

## 4. LSP Integration (Advisory Projections)

The `star-toml-lsp` language server projects the stable documentation model directly onto editor surfaces:
1. **Field Hover Cards**: Renders Markdown displaying:
   * Field Name, Type, and Stability status.
   * Description.
   * Expected default value, allowed values, and forbidden values.
   * Linked validation rules and authority boundaries.
2. **Override Warnings**: Marks keys violating override scopes or containing expired overrides.
3. **Authority Boundaries**: Highlights fields attempting to cross the declared boundaries (e.g. system configurations parsed inside application-level files).

---

## 5. Verification Ladder Integration

The documentation checks are mapped to **Level 4** of the star-toml Verification Ladder:

```text
Level 4: Documentation Integrity & Freshness -> All domains are documented, DOC-001 - DOC-020 pass, freshness verified, receipt signed.
```

The release verification engine ([src/bin/release_verifier.rs](file:///Users/sac/star-toml/src/bin/release_verifier.rs)) is modified to execute the documentation validator and fail package builds if any documentation counterexample fires.

---

## 6. DfCM Test Matrix

To test the documentation engine under combinatorial variations, tests are structured across these axes:

| Axis | Happy Fixture | Boundary Fixture | Negative Fixture | Cross-Axis Fixture |
|------|---------------|------------------|------------------|--------------------|
| **DomainCount** | Single domain | Empty domain list | Missing required domain | Interlocking domains |
| **FieldCount** | Standard field list | Large field list | No fields | Nested subfields |
| **FieldType** | Scalars, integers, booleans | Complex struct fields | Unsupported field types | Vector of tables |
| **RequiredState**| Explicit required fields | All optional fields | Missing required field doc | Required fields nested |
| **DefaultState** | Default value defined | Empty default value | Mismatched default type | Defaults overwritten |
| **OverrideState**| Explicit override allowed | No override policy | Stale/expired override | Override value validation |
| **BoundaryState**| Valid authority boundaries | Root/ambient boundaries | Boundary claim violation | Boundary escalation checks |
| **FreshnessState**| Fresh files matching code | Out-of-date JSON/Markdown | Mismatched file digests | Multi-stage build drift |
| **ReceiptState** | Valid signed receipt | Receipt empty | Digest mismatch | Mismatched signer key |

---

## 7. Implementation Status Checklist

* **Data Model**: Implemented (Rust structs `ConfigDocumentationModel`, `ConfigDomain`, `ConfigField`, `ConfigValidator`, `Counterexample`, `AuthorityBoundary`, `OverridePolicy`, `ReceiptBinding`)
* **Fact Ingestor**: Implemented (dynamic `Schema` ingestion and `get_domain` fact extractor)
* **Model Builder**: Implemented (alphabetical sorting of fields/domains/validators and model canonicalization)
* **Documentation Validator**: Implemented (`validate_docs_integrity` verifying P0 integrity and executing `DOC-001` - `DOC-020` audits)
* **Projection Engine**: Implemented (Markdown/JSON projections and freshness checking)
* **Receipt Binding**: Implemented (BLAKE3 receipts and verifications)
* **CLI/Tooling**: Implemented (`star_toml_docs` binary supporting `export`, `check`, and `verify`)
* **Release Verifier Integration**: Implemented (integrated check 20 `docs_integrity_and_freshness` to `release_verifier`)
* **LSP Integration**: Implemented (enriched hover cards in `hover_at` lookup from the documentation model)
* **Test Matrix Coverage**: Implemented (`docs_model_determinism` unit test and `docs_diagnostics` integration tests)

---

## 8. Individual JIRA Tickets (ST-113 Sub-tasks)

### ST-113-001 — Define Configuration Documentation Data Models
* **Type**: Task
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Rust types `ConfigDocumentationModel`, `ConfigDomain`, `ConfigField`, `ConfigValidator`, `Counterexample`, `AuthorityBoundary`, `OverridePolicy`, and `ReceiptBinding` must be fully defined and serializable via `serde`.
* **Verification**: Unit tests demonstrating serialize/deserialize round-trips.

### ST-113-002 — Ingest Configuration Facts & Enforce Alphabetical Canonicalization
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Ingestion of dynamic `Schema` data structures into the `ConfigDomain` representation. Sorting of fields, domains, and validators must be strictly alphabetical to ensure digest determinism.
* **Verification**: Metamorphic tests in `docs_model_determinism`.

### ST-113-003 — Human-Readable Markdown Projection Engine
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Generate canonical Markdown references mapping stability, types, default values, authority boundaries, and override scopes. Output written to `docs/config_reference.md`.
* **Verification**: Markdown parsing and string matching verification tests.

### ST-113-004 — Machine-Readable JSON Projection Engine
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Export the exact canonical documentation model to `docs/config_metadata.json` to allow automated schema consumer pipelines.
* **Verification**: JSON Schema conformity checks.

### ST-113-005 — RDF/Turtle Profile Projection Engine
* **Type**: Story
* **Priority**: P1
* **Status**: **Implemented**
* **Acceptance Criteria**: Support exporting admitted configuration definitions using semantic web standard Turtle/RDF vocabularies matching `stmacro:CapabilityProfile`.
* **Verification**: `ggen sync` compiles Turtle profiles without syntax errors.

### ST-113-006 — Cryptographic Receipt Binder
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Bind BLAKE3 signature evidence of candidate input configuration, model configuration, and output projections into an immutable `ReceiptBinding`.
* **Verification**: Integrity checks in the verifier.

### ST-113-007 — Documentation Integrity Validator
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Audit generated models for missing descriptions, default state mismatch, unlinked validators, or undocumented boundaries, raising `DOC-001` through `DOC-020` diagnostics.
* **Verification**: Negative diagnostic integration test suites in `tests/docs_diagnostics.rs`.

### ST-113-008 — CLI Tooling Interface (`star_toml_docs`)
* **Type**: Task
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Expose the tool as a compiled CLI binary named `star_toml_docs` with subcommand switches `export`, `check`, and `verify`.
* **Verification**: E2E test runner executing binary commands.

### ST-113-009 — LSP dynamic advisory hovers
* **Type**: Story
* **Priority**: P1
* **Status**: **Implemented**
* **Acceptance Criteria**: Language server hover cards must dynamically fetch and render configuration facts directly from `config_metadata.json`.
* **Verification**: Hover requests mapped under language client unit tests.

### ST-113-010 — Release Verifier integration
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Fail the release verifier package builds on stale or incomplete documentation.
* **Verification**: Release verification check 20 succeeds.

### ST-113-011 — DfCM Test Matrix and negative fixtures
* **Type**: Story
* **Priority**: P0
* **Status**: **Implemented**
* **Acceptance Criteria**: Full combinatorial test coverage of documentation validation checks across the axes of required state, types, boundaries, default overrides, and freshness.
* **Verification**: Verified under package test suites.
