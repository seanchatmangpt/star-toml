# ST-110: Error Topology and LSP Protocol

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for the Star TOML Error Topology engine and the Diagnostics Language Server (`star-toml-lsp`). It establishes the stable variant hashing mechanism (FNV-1a fingerprinting) for failure pattern clustering, precise span/source tracking, duplicate key detection, partial parse recovery, and a project-specific extension API. 

Crucially, this specification delineates the relationship between the LSP and the runtime, enforcing a strict separation of concerns to prevent runtime bloat while providing rich IDE support.

---

## Architectural Separation of Concerns

To maintain the security, determinism, and performance of the Star TOML admission substrate, the runtime engine and the Language Server Protocol (LSP) implementation must be strictly decoupled:

1. **Unidirectional Dependency**:
   - The runtime library (`star-toml`) must have **zero** dependency on the LSP protocol (`lsp-types`, `jsonrpc-core`, etc.). It remains a pure, lightweight, synchronous config validator suitable for embedded or resource-constrained production environments.
   - The LSP crate (`star-toml-lsp`) is a consumer of the runtime library. It imports `star-toml` to execute parse and validation loops, overlaying diagnostic mappings on top of the syntax tree.
2. **Execution Context**:
   - **Runtime**: Focused entirely on config admission (`Admit(Config)`), verifying bounds, generating cryptographic witnesses, and throwing hard, structured errors. It behaves deterministically.
   - **LSP Server**: Operates as an asynchronous, long-running JSON-RPC daemon. It handles partial parse states, maintains memory-resident files (using the LSP text document synchronization protocol), and tracks line/column locations of values in the original raw source document.
3. **The Topology Bridge**:
   - Error fingerprinting (FNV variant hashing) is implemented in the core runtime `star_toml::error` module, allowing production logs to emit stable variant IDs.
   - The LSP uses these same variant IDs to categorize error messages, map them to diagnostic codes, and provide repair hints to developers interactively.

---

## Key Requirements

### 1. Error Topology and Variant Hashing (`star_toml::error`)
- **FNV-1a Variant Hash**:
  - The runtime must calculate a stable, inputs-independent `variant_id` representing the configuration failure pattern.
  - The variant hash is computed as the FNV-1a hash of a sorted array of error codes and path locations:
    $$variant\_id = \text{FNV-1a}(\text{sorted}(\{\text{error\_code} + \text{"@"} + \text{path\_location}\}))$$
  - This ensures that different concrete malformed inputs that violate the same schema rules at the same paths produce the same `variant_id` (enabling variant clustering in telemetry and CI logs).
- **Validation Fitness Metric**:
  - Expose a mathematical validation fitness score:
    $$\text{fitness} = \frac{\text{PassedChecks}}{\text{TotalChecks}}$$
    Where `TotalChecks` is the union of all structural, type, and semantic validation rules evaluated for the document.
- **Auto-Generated Repair Hints**:
  - Design a non-authoritative hint generator that inspects error types and suggests remediation (e.g., suggesting a close match for a misspelled field name or recommending the required environment prefix).
  - Repair hints must be explicitly marked as suggestions rather than authoritative rules.

### 2. Diagnostic Spans and Duplicate Key Detection
- **Precise Span Mapping**:
  - The parser/LSP wrapper must track exact file positions (byte offsets, line and column numbers) for every key-value pair and array element.
  - Line and column indices must be 0-indexed in the internal LSP representation and mapped correctly to the 1-indexed human-readable reports.
- **Duplicate Key Detection**:
  - Detect duplicate keys within tables/inline tables during parsing.
  - Instead of silently overwriting or failing parsing immediately, collect duplicate keys as syntax diagnostics with line/col spans of all offending definitions (both first definition and subsequent duplicates).

### 3. Partial Parse Recovery
- **Non-Fatal Parse Failures**:
  - When encountering syntactic errors (e.g., unclosed strings, missing equals signs, unclosed brackets), the parser must not crash or halt compilation.
  - It must perform recovery by synchronizing to the next line or structural boundary (e.g., the next table header `[` or key-value pair).
- **Downstream Semantic Evaluation**:
  - Even if syntax errors occur, the LSP must parse all valid configuration blocks and run downstream validation checks on those sections (e.g., verifying types and environment bounds on successfully parsed keys).

### 4. LSP Extension APIs for Project-Specific Checks
- **Extensible Validation Pipeline**:
  - Expose public hooks/traits in `star-toml-lsp` (such as `LspExtensionProvider` or similar interface) allowing downstream projects to implement custom LSPs (e.g., `{{project}}-config-lsp`).
  - Downstream extensions must be able to inject domain-level policy checks (e.g., checking if specific dependencies are marked private or verifying business-specific bounds like `max_replicas <= 10`).
  - The extension API must allow custom checks to return diagnostics complete with severity, custom diagnostic codes, and precise spans.

---

## Acceptance Criteria

- [ ] **Stable Fingerprint Clustering**: Multiple independent parser executions with different concrete inputs (e.g., different invalid hostnames in a field, different out-of-bounds integers in another) that violate the exact same constraints at the same configuration paths yield identical `variant_id` hashes.
- [ ] **Determinism in Sorting**: The FNV-1a hash calculation must sort error locations/codes prior to hashing to guarantee evaluation-order independence.
- [ ] **Span Resolution**: For any configuration validation error (e.g., `MissingRequiredField`), the LSP can resolve the exact line/col span where the parent table is defined, or where the invalid type is specified.
- [ ] **Duplicate Key Diagnostics**: Every occurrence of a duplicate key yields a diagnostic of severity `Error` showing the span of the duplicate key and references the span of the first definition.
- [ ] **Resilience**: A TOML document with a syntax error at line 5 (e.g., missing quotes) still runs semantic checks on line 10, producing semantic diagnostics for line 10 if there are type/validation bounds violations.
- [ ] **Unidirectional Dependency**: The `cargo tree` of the workspace confirms that the core `star-toml` library has zero dependencies on LSP crates.
- [ ] **Extension Registration**: Downstream applications can instantiate the LSP server and register custom policy checkers, producing custom diagnostics that appear in the JSON-RPC response.

---

## Counterexamples Covered

- `validation_not_run_due_to_parse_error`: Halt checking the entire document when a minor syntax error is found, thus hiding downstream type or validation bounds failures.
- `validation_error_without_path_or_span`: Semantic validation engine reporting failures without linking them to specific files, paths, or precise line and column spans.
- `variant_id_drift`: Computing variant hashes that vary based on the order in which validations were executed (e.g., due to multi-threaded evaluation or HashMap iteration order).
- `lsp_dependency_in_runtime`: Importing `lsp-types` or JSON-RPC handlers into the core `star-toml` crate, inflating compile times and executable size for runtime admission containers.

---

## Verification Method

### 1. Unit Tests (Core & Runtime)
- **Stable Hash Tests**: Provide a set of mock error logs with varying evaluation orders and verify they produce identical FNV-1a variant IDs.
- **Fitness Tests**: Assert that the fitness ratio scales correctly under different ratios of passing and failing checks.
- **Repair Hints Test**: Verify that the generated hints for common mistakes (e.g. key `databse` instead of `database`) recommend correct alternatives.

### 2. Integration & LSP Tests
- **Partial Parsing Tests**: Feed malformed TOML files to the LSP parse engine and assert that syntax diagnostics are returned alongside semantic diagnostics for the valid parts.
- **Duplicate Key Tests**: Parse a TOML string with duplicate keys and assert that two distinct diagnostics are generated with their respective spans.
- **LSP Extension Test**: Implement a dummy extension provider that checks a custom key `allow_debug = false`. Parse a TOML with `allow_debug = true` and assert that the custom diagnostic is emitted over JSON-RPC.

### 3. Architecture Audit
- **Dependency Graph Check**: A CI step that runs `cargo tree -p star-toml` and validates that `lsp-types`, `tower-lsp`, or other IDE-specific libraries are completely absent from the runtime's dependency tree.
