# ST-101: Typestate Lifecycle Abstraction

## Description
Enforces the 2030 Trusted Lifecycle for configuration loading and admission in the Rust type system. This story ensures that the parser syntax is never treated as standing, and that configurations transition through explicit, compiler-enforced typestates before application bootstrap.

---

## Typestate Lifecycle Specifications

The typestate transitions are defined on the [`Config<S>`](file:///Users/sac/star-toml/src/loader.rs#L514-L519) struct, parameterized by their state wrapper:

### 1. `Config<Raw>`
- **Requirements**:
  - Represents the initial parsed state containing a raw, unchecked configuration value (e.g. `toml::Value`).
  - Wrapped by [`Raw`](file:///Users/sac/star-toml/src/loader.rs#L494).
  - No merging of layers or environment variable overrides has taken place.
  - Must not expose any interface to deserialize into typed configuration structures or read fields as application configuration.
- **Acceptance Criteria**:
  - Can only be instantiated via explicit parsing methods on `Loader` (e.g., [`Loader::load_raw()`](file:///Users/sac/star-toml/src/loader.rs#L241-L244)) or [`Config::new`](file:///Users/sac/star-toml/src/loader.rs#L523-L526).
  - Attempting to access inner values or pass it to components expecting validated configs triggers compile-time errors.
  - Extends zero authority to the application.
- **Verification Methods**:
  - Compiler safety check verifying that `Config<Raw>` lacks any methods for type extraction or semantic access.
  - Unit tests verifying raw parsing returns `Config<Raw>` and successfully encapsulates the raw AST.

### 2. `Config<BoundedSources>`
- **Requirements**:
  - Represents the state after all file-based layers are loaded and merged with provenance tracking, but before environment-variable overrides are applied.
  - Wrapped by [`BoundedSources`](file:///Users/sac/star-toml/src/loader.rs#L878).
  - Must record a [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs) for every source (including optional-missing files).
  - Must use [`deep_merge_traced`](file:///Users/sac/star-toml/src/merge.rs) to produce per-layer and cumulative winner maps.
- **Acceptance Criteria**:
  - Instantiated via [`Loader::load_bounded()`](file:///Users/sac/star-toml/src/loader.rs#L1080).
  - Contains full source and layer merge provenance reports but no environment-variable overrides.
- **Verification Methods**:
  - Unit and integration tests verifying that `load_bounded()` successfully tracks file layers and outputs a `Config<BoundedSources>`.

### 3. `Config<Merged>`
- **Requirements**:
  - Represents the unified config state after combining all authorized layers (e.g. default, file-based overrides) and environment variables without full provenance reports.
  - Wrapped by [`Merged`](file:///Users/sac/star-toml/src/loader.rs#L498).
  - Merging must follow deterministic bounds (recursive table merge, array/scalar replacement).
  - Must retain tracking metadata (source lineage/winning layer) for each field.
- **Acceptance Criteria**:
  - Transition from `Config<Raw>` to `Config<Merged>` is only possible by invoking [`merge()`](file:///Users/sac/star-toml/src/loader.rs#L538-L552).
  - Must fail transition if an unauthorized source or a missing required source is present.
  - No typed deserialization can occur before merging is completed.
- **Verification Methods**:
  - Unit tests asserting that multi-layer merging and environment-coercion produce a single `Config<Merged>` containing the correct precedence values.
  - Integration tests verifying that the lineage lookup map is correctly populated.

### 4. `Config<EnvResolved>`
- **Requirements**:
  - Represents the state after environment-variable overrides have been applied and recorded on top of `BoundedSources`.
  - Wrapped by [`EnvResolved`](file:///Users/sac/star-toml/src/loader.rs#L894).
  - Retains all prior reports forwarded from `BoundedSources`.
  - Captures environment-override provenance in an [`EnvOverrideReport`](file:///Users/sac/star-toml/src/reports.rs).
- **Acceptance Criteria**:
  - Transition from `Config<BoundedSources>` to `Config<EnvResolved>` is only possible by invoking [`apply_env()`](file:///Users/sac/star-toml/src/loader.rs#L962).
  - Variables matching prefix must be processed, with invalid paths rejected (e.g. returning code `"empty_path"`).
- **Verification Methods**:
  - Unit tests asserting that environment variable overrides are applied and traced correctly in `EnvOverrideReport` and `WinnerMap`.

### 5. `Config<Deserialized<T>>`
- **Requirements**:
  - Represents the structured configuration state after mapping the merged value into type `T`.
  - Wrapped by [`Deserialized<T>`](file:///Users/sac/star-toml/src/loader.rs#L502).
  - Deserialization must be strict, rejecting unknown fields to prevent silent drift or configuration pollution.
  - No semantic validation has been performed yet; the configuration must not be accessible for application bootstrap.
- **Acceptance Criteria**:
  - Transition from `Config<Merged>` to `Config<Deserialized<T>>` occurs via [`deserialize()`](file:///Users/sac/star-toml/src/loader.rs#L566-L572).
  - Transition from `Config<EnvResolved>` to `Config<Deserialized<T>>` occurs via [`deserialize()`](file:///Users/sac/star-toml/src/loader.rs#L1058-L1065).
  - Triggers a compilation or runtime failure if schema constraints or field mappings fail.
- **Verification Methods**:
  - Unit tests verifying that deserialization fails when unknown keys are present in the TOML structure.
  - Negative tests showing that a `Config<Deserialized<T>>` cannot be used to initialize the application engine.

### 6. `Config<Validated<T>>`
- **Requirements**:
  - Represents the configuration state after passing all semantic validation rules (e.g., semantic bounds, IP formats, cross-field integrity).
  - Wrapped by [`Validated<T>`](file:///Users/sac/star-toml/src/loader.rs#L506).
  - A validation report must be generated and attached.
- **Acceptance Criteria**:
  - Transition from `Config<Deserialized<T>>` to `Config<Validated<T>>` occurs via [`validate()`](file:///Users/sac/star-toml/src/loader.rs#L592-L616).
  - If any validation rule fails, the transition must abort and return an error containing the precise validation report.
- **Verification Methods**:
  - Unit and integration tests demonstrating that valid configurations transition smoothly while invalid configurations trigger structured semantic error reporting.

### 7. `Config<Frozen<T>>`
- **Requirements**:
  - Represents the final, read-only, immutable configuration that is safe to share across threads.
  - Wrapped by [`Frozen<T>`](file:///Users/sac/star-toml/src/loader.rs#L510).
  - No mutation of values or metadata can occur after this stage.
- **Acceptance Criteria**:
  - Transition from `Config<Validated<T>>` to `Config<Frozen<T>>` occurs via [`freeze()`](file:///Users/sac/star-toml/src/loader.rs#L648-L650).
  - Used to construct [`AdmittedConfig<T>`](file:///Users/sac/star-toml/src/loader.rs#L1561-L1568) alongside its cryptographic witness, source report, layer report, env report, and global winner map.
- **Verification Methods**:
  - Thread safety assertions (e.g. verifying the type implements `Send` and `Sync`).
  - Unit tests confirming immutable access through thread boundaries.

---

## Counterexamples Covered

### 1. `parse_valid_treated_as_trusted`
- **Danger**:
  - An application developer might parse valid TOML syntax and immediately treat it as a trusted config, bypassing merging, env override checks, strict deserialization, validation, and freezing.
- **Mitigation & Check**:
  - The type system prohibits accessing the inner type `T` (by value) or bootstrapping any engine using any state other than `AdmittedConfig<T>`. 
  - There is no API path to extract `T` (by value) or reference inner keys from `Config<Raw>` or `Config<Merged>`.
  - Compile-time linear dependencies guarantee validation and freezing are executed prior to usage.

### 2. `validation_not_run`
- **Danger**:
  - Bypassing the validation phase and accessing configuration values before semantic checks have been completed.
- **Mitigation & Check**:
  - The state transitions are linear and compiler-enforced:
    - Path A: `Raw -> Merged -> Deserialized<T> -> Validated<T> -> Frozen<T>`
    - Path B: `BoundedSources -> EnvResolved -> Deserialized<T> -> Validated<T> -> Frozen<T>`
  - `Config<Frozen<T>>` can only be constructed from `Config<Validated<T>>`. This compile-time linear dependency guarantees validation is executed prior to freezing and usage.

---

## Verification Ladder Alignment
- **Unit & Integration Phase**: Assert state transition functions compile, succeed, and fail under specified input scenarios.
- **Compiler Safety Phase**: Utilize static/trybuild tests to assert that improper state access (e.g., raw-to-frozen bypass, or calling `save_canonical` on states prior to validation) fails compilation (e.g. [`_compile_fail_save_canonical_before_validation`](file:///Users/sac/star-toml/src/lib.rs#L459-L460)).
