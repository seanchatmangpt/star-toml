# ST-106: Type Bounds

## Description
Establishes the validation boundaries and strict/permissive deserialization behaviors for configuration types. This ticket ensures that the config admission substrate enforces strict type validation based on the mode of operation (trusted vs. untrusted/exploratory). It guarantees that trusted configurations do not suffer from unrecognized field pollution, while untrusted/exploratory configurations gracefully admit partial schema alignment while fully reporting and logging undocumented fields.

## Key Requirements
1. **Strict Deserialization in Trusted Mode**:
   - In trusted config mode (specifically when loaded via the terminal method [`load_admitted`](file:///Users/sac/star-toml/src/loader.rs#L1626)), deserialization of TOML configuration inputs must reject unknown keys by default.
   - Enabling `deny_unknown_fields` behavior is enforced on the terminal `load_admitted` pipeline loader by default.
   - Any unknown keys encountered return `Error::Invalid(ValidationErrors)` containing precise location context for the failure.

2. **Permissive Deserialization & Reporting in Untrusted/Exploratory Mode**:
   - In untrusted or exploratory loading modes (via [`load_admitted_exploratory`](file:///Users/sac/star-toml/src/loader.rs#L1645)), unknown keys must not trigger an immediate error.
   - The loader must successfully deserialize all known schema fields.
   - Exploratory mode discards unrecognized keys.

3. **Precise Path Diagnostics**:
   - When deserialization fails due to an unknown key under trusted mode, the resulting error must identify the exact key-path location (e.g. `database.connection.pool_size` or `servers[0].unknown_setting` where `servers[0]` is represented as `LocSegment::Key("servers[0]")`).
   - The error code/diagnostic fingerprint should stabilize as `unknown_field`.

## Acceptance Criteria
- [x] By default, [TrustedLoader::load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1626) rejects any configuration that contains keys not defined on the target Rust configuration type.
- [x] If an unknown key is encountered in trusted mode, loading fails and returns a path-precise error pointing directly to the unknown field.
- [x] When the loader is configured in untrusted or exploratory mode, configurations containing unknown keys load successfully.
- [x] In exploratory mode, the load succeeds and unknown keys are ignored.
- [x] Unknown configuration fields are ignored under exploratory mode.


## Counterexamples Covered
- `unknown_field_accepted_in_trusted_mode`: Allowing undocumented keys to be accepted and loaded in trusted mode without raising an error or reporting the drift.

## Verification Method
- **Unit & Integration Tests**:
  - Implement test cases loading a target Rust struct containing known fields (`name`, `port`) from a TOML string with an extra unknown field (`debug_mode = true`).
  - Verify that under the default [load_admitted](file:///Users/sac/star-toml/src/loader.rs#L1626) pipeline, this load fails indicating the presence of `debug_mode`.
  - Verify that under untrusted/exploratory loading, the load succeeds.
- **Chaos Tests**:
  - Run the verification tool to ensure that the `unknown_field_accepted_in_trusted_mode` counterexample check is integrated into the release verifier and produces a passing report.
