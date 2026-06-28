# ST-107: Precise Path Validation

## Description
Define formal requirements, acceptance criteria, and verification methods for semantic configuration validation bounds. This story guarantees that every configuration validation failure is tracked with a precise structural tree path and contains complete diagnostic metadata (error code, severity level, offending input, human-readable message, and repair hints), preventing untraceable failures and ensuring that fatal errors cannot be downgraded.

## Key Requirements
1. **Semantic Validation Errors (`star_toml::validation::ValidationError`)**:
   Every validation error must be encapsulated in a structured [ValidationError](file:///Users/sac/star-toml/src/validation.rs#L302-L313) containing the following fields and methods:
   - **Precise Path (`loc: Loc`)**: A structured tree location path (comprising table keys and array indices) pointing to the exact location of the failure (e.g., `database.connection.port` or `gateways[2].url`). It renders as a dotted string path when formatted. Path segment tracking is recursively pushed and popped during structural descent. Root-level errors have an empty location and render as `(root)`.
   - **Error Kind (`kind: ErrorKind`)**: A structured enum containing the specific error details (such as `Missing`, `Empty`, `OutOfRange`, `TooShort`, `TooLong`, `NotOneOf`, `Inconsistent`, or `Predicate`).
   - **Error Code (`code(&self) -> &str`)**: A method returning a stable, machine-readable string variant identifier (e.g., `"empty"`, `"out_of_range"`, `"invalid_semver"`, `"invalid_ip_or_domain"`, `"invalid_size_format"`, or other custom predicate/inconsistency codes).
   - **Severity Level (`severity: Severity`)**: Ordered validation failure levels (`Advisory`, `Warning`, `Error`, `Fatal`).
   - **Offending Input (`input: Option<String>`)**: The stringified value that triggered the validation failure (preserved for auditing and debugging).
   - **Human-Readable Message (`msg: String`)**: Descriptive text explaining the cause of the validation failure.
   - **Repair Hint (`repair_hint(&self) -> String`)**: A method returning derived alignment corrections calculated automatically to suggest the minimal edit needed to resolve the validation failure (e.g., suggesting valid ranges or allowed options).

2. **Core Validation Checkers**:
   Provide robust validation methods on the [Validator](file:///Users/sac/star-toml/src/validation.rs) pipeline:
   - **Semantic Versioning Checker (`check_semver`)**: Verifies that input strings conform strictly to the SemVer 2.0.0 specification (e.g., rejecting leading zeros in version components like `01.0.0`, ensuring exact triple syntax `X.Y.Z` without leading zeros). Note that standard SemVer 2.0.0 is validated but does not support pre-release tags or build metadata.
   - **Network Host Checker (`check_ip_or_domain`)**: Verifies that strings match either valid IPv4/IPv6 addresses or RFC-compliant Domain Name formats (e.g., rejecting leading/trailing hyphens in labels, maximum length of 253 characters, valid hostname syntax).
   - **Memory Size Format Checker (`check_size_format`)**: Verifies (read-only validation) memory size string patterns (e.g., `"10B"`, `"512KB"`, `"1024MB"`, `"1GB"`, `"2TB"`) in a case-insensitive manner, enforcing presence of valid numeric prefixes and correct units, without spaces or decimal places.
   - **Cross-Field Consistency Checker (`check_consistent`)**: Resolves dependencies between sibling fields (e.g., verifying that a security certificate path is provided if SSL is enabled) and records the set of related fields in the error details (`ErrorKind::Inconsistent`).

3. **Inadmissible Behavior & Boundary Safety**:
   - **Root-Level / Pathless Validation Errors**: The system supports validation errors at the root level of the configuration (with an empty path `Loc(vec![])`), which are rendered as `(root)`. This is used, for example, during strict-mode loading when unknown keys at the root level are rejected, or when general root-level constraints are checked.
   - **Severity Levels**: Any check generating an error with `Severity::Fatal` flags the validation report as failed (`has_fatal() == true`), which halts the config admission process and prevents loading the configuration. Errors with `Severity::Fatal` cannot be downgraded or ignored.

## Acceptance Criteria
- [x] Every semantic validation failure produces a [ValidationError](file:///Users/sac/star-toml/src/validation.rs#L302-L313) carrying a valid `loc` property containing a structural path (e.g., `servers[0].hosts[1]`), or an empty path representing the root.
- [x] Root-level or general structural validation errors without a specific subfield are recorded with an empty `loc` path and rendered as `(root)`.
- [x] The `ValidationError::repair_hint` method returns derived corrections for all built-in checker variants (e.g., `OutOfRange` suggests valid ranges, `NotOneOf` lists allowed values).
- [x] The `check_semver` checker rejects invalid version formats (e.g., `1.0`, `1.0.0.0`, `01.0.0`) and returns an `invalid_semver` error code.
- [x] The `check_ip_or_domain` checker accepts valid domains/IPs and rejects malformed domains (e.g., `a..b`, `a.b_c.d`, labels longer than 63 characters) and returns an `invalid_ip_or_domain` error code.
- [x] The `check_size_format` checker supports units `B`, `KB`, `MB`, `GB`, `TB` in a case-insensitive manner and rejects invalid formats like decimal numbers (`1.5GB`) or spaces (`512 MB`) and returns an `invalid_size_format` error code.
- [x] Sibling field dependencies can be verified using `check_consistent`, and any failure lists the relevant dependent fields in the `ErrorKind::Inconsistent` details.
- [x] A validation check marked as `Severity::Fatal` flags the validator report as fatal, halting config admission, and cannot be downgraded to advisory/warning levels.

## Counterexamples Covered
- `validation_error_without_path`: General/root level failures are allowed and represented by `(root)`.
- `fatal_error_downgraded`: Softening fatal checks into warning messages.

## Verification Method
- **Unit Tests**:
  - Verify that the `check_semver`, `check_ip_or_domain`, and `check_size_format` checkers successfully validate all correct string formats and correctly report errors on invalid formats.
  - Verify that `ValidationError::repair_hint` generates precise suggestions matching the failed checks.
  - Verify that structural path segments are correctly tracked during recursive validation of nested structs and arrays.
- **Integration Tests**:
  - Validate a full configuration payload containing invalid semver strings, malformed IP addresses, and size format issues, confirming that the resulting `ValidationErrors` report lists precise paths for all errors.
  - Assert that a `Fatal` error sets `has_fatal()` to true, preventing configuration admission, and that the error cannot be downgraded.
