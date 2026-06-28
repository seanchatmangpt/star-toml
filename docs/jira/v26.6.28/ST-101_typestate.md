# ST-101: Typestate Lifecycle Abstraction

## Description
Enforces the 2030 Trusted Lifecycle for configuration loading and admission in the Rust type system. This story ensures that the parser syntax is never treated as standing, and that configurations transition through explicit, compiler-enforced typestates before application bootstrap.

---

## Typestate Lifecycle Specifications

### 1. `Config<Raw>`
- **Requirements**:
  - Represents the initial parsed state containing a raw, unchecked configuration value (e.g. `toml::Value`).
  - No merging of layers or environment variable overrides has taken place.
  - Must not expose any interface to deserialize into typed configuration structures or read fields as application configuration.
- **Acceptance Criteria**:
  - Can only be instantiated via explicit parsing methods (e.g. `Parser::parse_str` or `Parser::parse_file`).
  - Attempting to access inner values or pass it to components expecting validated configs triggers compile-time errors.
  - Extends zero authority to the application.
- **Verification Methods**:
  - Compiler safety check verifying that `Config<Raw>` lacks any methods for type extraction or semantic access.
  - Unit tests verifying raw parsing returns `Config<Raw>` and successfully encapsulates the raw AST.

### 2. `Config<Merged>`
- **Requirements**:
  - Represents the unified config state after combining all authorized layers (e.g. default, file-based overrides) and environment variables.
  - Merging must follow deterministic bounds (recursive table merge, array/scalar replacement).
  - Must retain tracking metadata (source lineage/winning layer) for each field.
- **Acceptance Criteria**:
  - Transition from `Config<Raw>` to `Config<Merged>` is only possible by invoking `merge_layers()`.
  - Must fail transition if an unauthorized source or a missing required source is present.
  - No typed deserialization can occur before merging is completed.
- **Verification Methods**:
  - Unit tests asserting that multi-layer merging and environment-coercion produce a single `Config<Merged>` containing the correct precedence values.
  - Integration tests verifying that the lineage lookup map is correctly populated.

### 3. `Config<Deserialized<T>>`
- **Requirements**:
  - Represents the structured configuration state after mapping the merged value into type `T`.
  - Deserialization must be strict, rejecting unknown fields to prevent silent drift or configuration pollution.
  - No semantic validation has been performed yet; the configuration must not be accessible for application bootstrap.
- **Acceptance Criteria**:
  - Transition from `Config<Merged>` to `Config<Deserialized<T>>` occurs via `deserialize()`.
  - Triggers a compilation or runtime failure if schema constraints or field mappings fail.
- **Verification Methods**:
  - Unit tests verifying that deserialization fails when unknown keys are present in the TOML structure.
  - Negative tests showing that a `Config<Deserialized<T>>` cannot be used to initialize the application engine.

### 4. `Config<Validated<T>>`
- **Requirements**:
  - Represents the configuration state after passing all semantic validation rules (e.g., semantic bounds, IP formats, cross-field integrity).
  - A validation report must be generated and attached.
- **Acceptance Criteria**:
  - Transition from `Config<Deserialized<T>>` to `Config<Validated<T>>` occurs via `validate()`.
  - If any validation rule fails, the transition must abort and return an error containing the precise validation report.
- **Verification Methods**:
  - Unit and integration tests demonstrating that valid configurations transition smoothly while invalid configurations trigger structured semantic error reporting.

### 5. `Config<Frozen<T>>`
- **Requirements**:
  - Represents the final, read-only, immutable configuration that is safe to share across threads.
  - No mutation of values or metadata can occur after this stage.
- **Acceptance Criteria**:
  - Transition from `Config<Validated<T>>` to `Config<Frozen<T>>` occurs via `freeze()`.
  - Wrapped in `AdmittedConfig<T>` alongside its witness, validation report, and source report.
- **Verification Methods**:
  - Thread safety assertions (e.g. verifying the type implements `Send` and `Sync`).
  - Unit tests confirming immutable access through thread boundaries.

---

## Counterexamples Covered

### 1. `parse_valid_treated_as_trusted`
- **Danger**:
  - An application developer might parse valid TOML syntax and immediately treat it as a trusted config, bypassing merging, env override checks, strict deserialization, validation, and freezing.
- **Mitigation & Check**:
  - The type system prohibits accessing the inner type `T` or bootstrapping any engine using any state other than `Config<Frozen<T>>` within `AdmittedConfig<T>`. 
  - There is no API path to extract `T` or reference inner keys from `Config<Raw>`, `Config<Merged>`, or `Config<Deserialized<T>>`.

### 2. `validation_not_run`
- **Danger**:
  - Bypassing the validation phase and accessing configuration values before semantic checks have been completed.
- **Mitigation & Check**:
  - The state transitions are linear and compiler-enforced: `Raw -> Merged -> Deserialized<T> -> Validated<T> -> Frozen<T>`.
  - `Config<Frozen<T>>` can only be constructed from `Config<Validated<T>>`. This compile-time linear dependency guarantees validation is executed prior to freezing and usage.

---

## Verification Ladder Alignment
- **Unit & Integration Phase**: Assert state transition functions compile, succeed, and fail under specified input scenarios.
- **Compiler Safety Phase**: Utilize static/trybuild tests to assert that improper state access (e.g., raw-to-frozen bypass) fails compilation.
