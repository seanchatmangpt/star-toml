# ST-106: Type Bounds

## Description
Establishes the validation boundaries and strict/permissive deserialization behaviors for configuration types. This ticket ensures that the config admission substrate enforces strict type validation based on the mode of operation (trusted vs. untrusted/exploratory). It guarantees that trusted configurations do not suffer from unrecognized field pollution, while untrusted/exploratory configurations gracefully admit partial schema alignment while fully reporting and logging undocumented fields.

## Key Requirements
1. **Strict Deserialization in Trusted Mode**:
   - In trusted config mode (e.g., when loaded via the [trusted()](file:///Users/sac/star-toml/src/lib.rs#L434) builder or [TrustedLoader](file:///Users/sac/star-toml/src/loader.rs#L765)), deserialization of TOML configuration inputs must reject unknown keys.
   - Enabling `deny_unknown_fields` behavior (conceptually equivalent to Serde's `#[serde(deny_unknown_fields)]` macro attribute) must be the default behavior for all trusted config loads.
   - Any unknown keys encountered must immediately abort the admission pipeline and return a structured [Error::Parse](file:///Users/sac/star-toml/src/error.rs) containing precise location context for the failure.

2. **Permissive Deserialization & Reporting in Untrusted/Exploratory Mode**:
   - In untrusted or exploratory loading modes, unknown keys must not trigger an immediate deserialization error.
   - The loader must successfully deserialize all known schema fields.
   - Any unrecognized keys must be captured, parsed, and logged as warnings via the standard logging interface.
   - Captured unknown keys must be exposed programmatically to the caller through the loading report, such as the [ValidationReport](file:///Users/sac/star-toml/src/loader.rs#L729) or [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L720). Unrecognized fields must never be silently discarded.

3. **Precise Path Diagnostics**:
   - When deserialization fails due to an unknown key under trusted mode, the resulting error must identify the exact key-path location (e.g. `database.connection.pool_size` or `servers[0].unknown_setting`).
   - The error code/diagnostic fingerprint should stabilize as `unknown_field`.

## Acceptance Criteria
- [ ] By default, [TrustedLoader::load](file:///Users/sac/star-toml/src/loader.rs#L822) rejects any configuration that contains keys not defined on the target Rust configuration type.
- [ ] If an unknown key is encountered in trusted mode, loading fails and returns a path-precise error pointing directly to the unknown field.
- [ ] When the loader is configured in untrusted or exploratory mode, configurations containing unknown keys load successfully.
- [ ] In untrusted/exploratory mode, all unknown keys present in the TOML input are extracted and populated into the [ValidationReport](file:///Users/sac/star-toml/src/loader.rs#L729) or diagnostic log.
- [ ] No unknown configuration fields are silently ignored or discarded under any operation mode.

## Counterexamples Covered
- `unknown_field_accepted_in_trusted_mode`: Allowing undocumented keys to be accepted and loaded in trusted mode without raising an error or reporting the drift.

## Verification Method
- **Unit & Integration Tests**:
  - Implement test cases loading a target Rust struct containing known fields (`name`, `port`) from a TOML string with an extra unknown field (`debug_mode = true`).
  - Verify that under the default [trusted()](file:///Users/sac/star-toml/src/lib.rs#L434) pipeline, this load fails with a parse error indicating the presence of `debug_mode`.
  - Verify that under untrusted/exploratory loading, the load succeeds and the resulting validation report lists `debug_mode` as an unrecognized field.
- **Chaos Tests**:
  - Run the verification tool to ensure that the `unknown_field_accepted_in_trusted_mode` counterexample check is integrated into the release verifier and produces a passing report.
