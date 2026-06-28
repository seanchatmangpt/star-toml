# ST-102: Trusted API & AdmittedConfig Structure

## Description
This ticket defines the design, implementation, and verification requirements for the ergonomic trusted configuration loading interface. It introduces the [`star_toml::trusted()`](file:///Users/sac/star-toml/src/lib.rs#L463) entry point, the [`TrustedLoader`](file:///Users/sac/star-toml/src/loader.rs#L769) builder, the [`TrustedConfig<T>`](file:///Users/sac/star-toml/src/loader.rs#L750-L759) container, and the [`AdmittedConfig<T>`](file:///Users/sac/star-toml/src/loader.rs#L1561-L1568) wrapper structure. The primary objective is to enforce the correct configuration admission lifecycle order, ensuring that callers can only access frozen, validated configuration data along with a complete audit receipt consisting of source reports, layer reports, env-override reports, winning-field maps, and a cryptographic witness.

## Key Requirements

1. **The `star_toml::trusted()` Entry Point**:
   - Provide a public, zero-argument function [`star_toml::trusted()`](file:///Users/sac/star-toml/src/lib.rs#L463) that serves as the unique entry point for performing config admission.
   - This function must return a [`TrustedLoader`](file:///Users/sac/star-toml/src/loader.rs#L769) builder initialized with no configured layers.

2. **The `TrustedLoader` Builder**:
   - Provide a fluent builder pattern to configure config layers:
     - `layer_str(content, label)`: Add an inline string layer.
     - `layer_file(path)`: Add a required file layer.
     - `layer_file_if_exists(path)`: Add an optional file layer.
     - `find_file(filename)`: Add a layer by searching parent directories.
     - `env_prefix(prefix)`: Add environment variable overrides filtered by prefix.
   - Enforce compiler-level lifecycle ordering during execution of the terminal method:
     - The terminal methods `load_admitted::<T>()` and `load_admitted_strict::<T>()` must execute the transition sequence: `Discover -> Load -> Merge -> EnvOverride -> Deserialize -> Validate -> Normalize -> Freeze -> Generate Reports -> Construct Witness`.
     - Intermediate unchecked states (such as Raw or Deserialized config) must remain completely internal to the loader and must not be exposed to the caller.

3. **The `AdmittedConfig<T>` Return Value**:
   - The terminal `load_admitted::<T>()` (or strict variant `load_admitted_strict::<T>()`) method of the builder must return a `Result<AdmittedConfig<T>, Error>`. (Note: the default `load::<T>()` method returns a `Result<TrustedConfig<T>, Error>`, which does not compute the full cryptographic witness).
   - `AdmittedConfig<T>` represents the final receipt of successful configuration admission and must expose the following:
     - `value()`: The final configuration value of type `T`.
     - `witness()`: The cryptographic `ConfigWitness` proving the authenticity of the config state.
     - `source_report()`: The `SourceReport` detailing the source tracking metadata.
     - `layer_report()`: The `LayerReport` detailing the layer merge details.
     - `env_report()`: The `EnvOverrideReport` detailing environment variables.
     - `global_winner_map()`: The `WinnerMap` tracking leaf field lineage.
   - Implement `Deref` pointing to `T` on `AdmittedConfig<T>` to allow convenient access to the validated configuration values.

4. **Cryptographic Witness (`ConfigWitness`)**:
   - A cryptographic checksum designed to verify that the final configuration was constructed using the exact sequence of permitted layers and overrides.
   - It must hash:
     - The canonical TOML string representation of the frozen config.
     - The contents and paths of all loaded sources.
     - The active environment variable overrides.
     - The validation report metadata.
   - Ensure the hash algorithm is deterministic and stable across multiple compiler runs and executions.

5. **Validation Report (`ValidationReport`)**:
   - A structured report reflecting the semantic checks carried out during admission.
   - It must contain:
     - `fitness`: A floating-point score (0.0 to 1.0) indicating the ratio of passed validation checks.
     - `checks_run`: The total number of checks performed.
     - `checks_passed`: The total number of checks successfully passed.
     - `errors`: A list of all reported errors, mapping each error precisely to its location path.

6. **Source Report (`SourceReport`)**:
   - An audit trail listing every layer that participated in config loading.
   - It must track:
     - The path or string label of each layer.
     - The status of each layer (loaded, skipped, missing, or overridden).
     - The content digest for file and string layers.

---

## Acceptance Criteria

- [ ] Calling `star_toml::trusted()` returns a `TrustedLoader` builder.
- [ ] The terminal methods `load_admitted` / `load_admitted_strict` on `TrustedLoader` run the full lifecycle validation flow and return an `AdmittedConfig<T>` only if all checks succeed.
- [ ] If any phase of the admission lifecycle fails (e.g., a required file is missing, parsing fails, or validation errors of `Error` severity are found), the loading halts and returns the corresponding error variant without producing an `AdmittedConfig<T>`.
- [ ] `AdmittedConfig<T>` wraps `T` and implements `Deref<Target = T>` to provide read-only access to the final configuration values.
- [ ] The cryptographic witness (`ConfigWitness`) is present, fully populated, and verified to change if any input TOML source, active environment variable override, or validation error changes.
- [ ] The witness is completely deterministic; loading the identical configuration state repeatedly must produce the identical witness hash.
- [ ] The `ValidationReport` lists precise path locations for all validation failures and computes the correct fitness score.
- [ ] The `SourceReport` contains details of all configured files, inline string layers, and env overrides, with precise digests for all read contents.

---

## Verification Method

### Unit Tests
- **Builder Lifecycle Sequence**: Verify that individual transition methods inside the loader succeed only under correct ordering, and that intermediate states are not accessible.
- **Witness Stability**: Test that loading identical configuration values results in identical `ConfigWitness` hashes.
- **Witness Sensitivity**: Test that changing a value in an input file, changing an environment variable override, or introducing a validation warning changes the output `ConfigWitness`.
- **Validation Metrics**: Assert that `ValidationReport` correctly counts checks run, checks passed, and calculates fitness scores under both fully valid and partially warning states.
- **Source Trail**: Assert that the `SourceReport` accurately tracks missing optional files, present optional files, required files, and env variables.

### Integration Tests
- **Successful Admission**: Load a configuration using a full stack of defaults, files, and environment overrides, asserting that `star_toml::trusted().load_admitted()` successfully returns `AdmittedConfig<T>` with all expected reports.
- **Failed Admission Handling**: Verify that loading a configuration that fails validation does not return `AdmittedConfig<T>` but instead returns `Err(Error::Invalid(ValidationErrors))` containing the exact validation report.
- **State Guarantee**: Validate that a compiled program cannot mutate the configuration fields inside `AdmittedConfig<T>` (compiler safety checks/static assertions).
