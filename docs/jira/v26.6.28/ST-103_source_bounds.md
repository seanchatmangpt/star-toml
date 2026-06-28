# ST-103: Source Bounds

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for implementing and enforcing **Source Bounds** within the configuration admission substrate. Under the **Vision 2030** guidelines, all configuration sources must be explicitly declared, strictly authorized, content-hashed (digestible), and reported in a metadata summary. Silent file resolution or magic fallback mechanisms are rejected, and missing required sources must result in immediate admission failure rather than partial loading.

## Key Requirements

1. **Explicit and Allowed Sources**:
   - The engine must reject any ambient or implicit sources that are not explicitly registered in the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191).
   - Config sources must be restricted to an allowed set of variants defined in the loading model (e.g. `built_in_defaults`, `required_file`, `optional_file`, `found_file`, `env_override`, and `raw_string`).
   - If an unknown, unregistered, or disallowed configuration source is encountered, the loader must immediately abort config admission and return an error.

2. **Digestible Sources**:
   - To guarantee provenance, reproducibility, and auditability, the engine must digest the exact content of every loaded configuration source.
   - For file sources, defaults, and environment variable overrides, a cryptographic content hash (e.g. SHA-256 or FNV-1a hash) must be computed over the raw inputs.

3. **Reported Sources (`SourceReport`)**:
   - The final configuration output package must contain a structured source metadata report, represented by [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725).
   - This report must list all configured sources, recording:
     - The source's label or identifier.
     - The status of the source (e.g. `Found`, `Missing`, or `Overridden`).
     - The resolved absolute filepath (where applicable).
     - The calculated content digest.
     - A boolean flag indicating whether the source was required.

4. **Failure on Unknown or Missing Required Files**:
   - **Unknown/Disallowed Source**: Attempting to feed an unauthorized source to the loader triggers an admission check failure (e.g., returning an `UnknownSource` validation error).
   - **Missing Required File**: If a config source marked as "required" (such as a file loaded via `layer_file()`) is absent from the filesystem, the engine must fail immediately with a [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) or custom `MissingRequiredFile` variant. It must never silently ignore the missing file, fall back to defaults, or succeed partially.
   - **Missing Optional File**: If a config source is marked optional (e.g., loaded via `layer_file_if_exists()`), it is registered in the [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725) as missing but does not raise an error.

## Acceptance Criteria
- [ ] Attempts to load an invalid or missing required file source trigger a compile-time type constraint error or run-time [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) (or equivalent `MissingRequiredFile`) immediately.
- [ ] Attempting to configure or inject an unauthorized/unregistered source into the admission pipeline halts execution with a configuration failure.
- [ ] Every successful config load operation produces a [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725) that lists all attempted and resolved configuration sources.
- [ ] The [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725) includes a verified content hash/digest for each successfully loaded configuration source.
- [ ] If an optional source file is absent, it does not fail admission but is reported as missing in the final report, with no digest generated.
- [ ] The loader does not perform any magic discovery of configuration files (e.g. from hardcoded system locations or parent folders) unless explicitly added to the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191).

## Counterexamples Covered
- `implicit_source_used`: Silently loading or falling back to undocumented config files. All configuration origins must be declared in the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191) registry.
- `missing_required_file_not_error`: Missing files marked "required" failing to raise errors. Any required configuration file that is absent must halt the admission pipeline.

## Verification Method

### Unit Tests
- **Allowed Registry Verification**: Call the config loader with an invalid/unregistered source type, and verify that the loader returns a configuration error.
- **Required File Absence**: Call the loader with a required file path that does not exist on disk, and assert that it returns [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) (or equivalent `MissingRequiredFile`).
- **Optional File Absence**: Call the loader with an optional file path that does not exist, and assert that the loader completes successfully and records the file status as missing in [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725).
- **Digest Correctness**: Verify that loading a file with known content produces a digest that matches the expected content hash (e.g. SHA-256).

### Integration Tests
- Run the full [TrustedLoader](file:///Users/sac/star-toml/src/loader.rs#L765-L790) builder sequence using a default string, a local file, and an environment override, verifying that the generated [ConfigSourceReport](file:///Users/sac/star-toml/src/loader.rs#L722-L725) correctly lists all three sources, their paths (or labels), and their calculated digests.

### Chaos Tests
- Delete a required configuration file mid-process or before loading and verify that config admission is rejected.
- Introduce an ambient/implicit config file in the current working directory and verify that the engine ignores it unless explicitly registered.
