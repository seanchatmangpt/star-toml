# Implementation Guide: Implementing v26.6.29 Full ST-JSON Admission Release

> [!WARNING]
> **CRITICAL INCOMING AGENT DIRECTIVE: DISTRUST PREVIOUS CLAIMS OF COMPLETENESS.**
> While the test suites compile and run, the previous implementation contains significant gaps, stubs, and static mock data. Do NOT assume a feature is "done" simply because there is a green tick in a markdown matrix or because `cargo test` passes. You must audit the actual code execution paths. Follow these instructions strictly to implement the true, production-grade configuration standing system.

---

## 🏛️ Codebase Gaps & Audit Points (Where to focus)

### 1. Document Diagnostics Incompleteness
* **Location**: [`src/docs.rs`](file:///Users/sac/star-toml/src/docs.rs) inside `validate_docs_integrity` at [`L241-369`](file:///Users/sac/star-toml/src/docs.rs#L241-L369).
* **The Gap**: The validator only evaluates 9 of the 20 required `DOC` diagnostics. The remaining 11 checks (`DOC-008` and `DOC-010` through `DOC-020`) are missing from the code return.
* **Instruction**:
  1. Modify the `DocDiagnostic` struct ([`src/docs.rs#L231-L238`](file:///Users/sac/star-toml/src/docs.rs#L231-L238)) to add the missing fields: `trigger`, `negative_fixture`, `test_coverage`, and `release_verifier_coverage_if_fatal`.
  2. Implement actual checks for all missing diagnostics:
     * `DOC-008` (ReceiptPolicyUndocumented): Ensure receipt boundaries exist if required by policy.
     * `DOC-010` (GeneratedDocsOutOfDate): Check if files on disk match the projected canonical Markdown/JSON model.
     * `DOC-011` (ManualDocsClaimStanding): Scan files on disk for missing or manually edited headers without valid receipt footprints.
     * `DOC-012` (DocsGeneratedFromUnadmittedFacts): Block if config schema facts failed admission checks.
     * `DOC-014` (StableDomainMissingMachineReadableExport): Assert `config_metadata.json` exists.
     * `DOC-015` (ValidatorDocumentedButNotEnforced) to `DOC-018` (AuthorityBoundaryDocumentedButNotEnforced): Align documented validator list against compiled schema validators and loaders.
     * `DOC-019` (ProjectionTargetMissingRequiredDomain): Validate target domains in template.
     * `DOC-020` (ReceiptDigestMismatch): Cryptographically verify digests.

### 2. Schema Admission Refusal Enforcement
* **Location**: [`src/loader.rs`](file:///Users/sac/star-toml/src/loader.rs) inside `load_admitted_strict` at [`L121-150`](file:///Users/sac/star-toml/src/loader.rs#L121-L150).
* **The Gap**: There is currently no active check that verifies whether a JSON Schema candidate domain has been admitted by the schema pipeline before configuration validation is run against it.
* **Instruction**:
  1. Add schema candidate admission validation to the configuration loading pipeline.
  2. Fail loading with `SCH-JSON-008` if config validation is run against an unadmitted schema.
  3. Fail documentation generation in `docs.rs` with `SCH-JSON-009` if the schema source did not pass admission.

### 3. Star-Toml Docs Tooling Hardcoding
* **Location**: [`src/bin/star_toml_docs.rs`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs) at [`L13-41`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs#L13-L41) and [`L77-85`](file:///Users/sac/star-toml/src/bin/star_toml_docs.rs#L77-L85).
* **The Gap**: The schema source is hardcoded to a mock `system` schema, and the output receipt utilizes a hardcoded timestamp, generator version, and signer operator key.
* **Instruction**:
  1. Implement dynamic schema discovery (read from configured workspace TOML or JSON Schema directories) instead of using the hardcoded `system_config_schema`.
  2. Retrieve dynamic timestamp records and signer credentials from environment variables or active runtime configuration.
  3. Ensure that the `export` command also writes the exported JSON Schema layout (`docs/schema.json`) to disk.

### 4. Release Verifier Report Mapping
* **Location**: [`src/bin/release_verifier.rs`](file:///Users/sac/star-toml/src/bin/release_verifier.rs) at [`L660-684`](file:///Users/sac/star-toml/src/bin/release_verifier.rs#L660-L684) and report display blocks.
* **The Gap**: The verifier lacks any stage checking the JSON Schema bridge status (`q_json_schema_bridge`), and the CLI printout format does not display the 10 required DoD sections.
* **Instruction**:
  1. Add a validation stage (Check 21) executing bridge standing checks.
  2. Format the console printout to list the exactly matching 10 sections table (JSON Schema Bridge Standing, Schema Admission, Config Admission, Generated Docs Standing, Receipt Binding, Projection Freshness, DOC Diagnostics, SCH-JSON Diagnostics, DfCM Fixture Coverage, Final Standing) ending with `ADMITTED` or `REFUSED`.
  3. Allow check 15 (`cargo_package_without_clean_git`) to succeed locally if run with a `--dev` flag or override environment variable, to enable testing during development.

---

## 🛠️ Step-by-Step Implementation Sequence

1. **Diagnostic Restructurings**:
   * Add the missing metadata fields to `DocDiagnostic` in `src/docs.rs` and `SchJsonDiagnostic` in `star-toml-json-schema/src/diagnostics.rs`.
2. **Ingestor & Loader Gates**:
   * Add schema validation checks inside `src/loader.rs` to intercept configuration loading.
3. **Docs Engine Upgrades**:
   * Complete all 20 diagnostic branches inside `src/docs.rs::validate_docs_integrity` to perform complete static analysis over the metadata model.
4. **Verifier Reporting & CLI Alignment**:
   * Update the release verifier CLI report formatting to output the DoD-compliant 10 sections table.
5. **Fixture Coverage Mapping**:
   * Append test cases inside `tests/docs_diagnostics.rs` and `star-toml-json-schema/tests/bridge_tests.rs` to trigger each of the 36 diagnostics.

---

## 🚦 Required Final Verification Checks

Before submitting the release as ADMITTED, run this exact sequence of commands and ensure they all succeed:

```bash
# 1. Verify Rust formatting
cargo fmt --check

# 2. Check compiler safety across targets
cargo check --all-targets

# 3. Execute all unit and integration test suites
cargo test

# 4. Compile Turtle profiles and verify SPARQL mappings
ggen sync

# 5. Run the global release standing gate verifier
cargo run --bin release_verifier
```
