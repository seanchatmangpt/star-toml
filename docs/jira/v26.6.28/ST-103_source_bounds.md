# ST-103: Source Bounds

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for implementing and enforcing **Source Bounds** within the configuration admission substrate. Under the **Vision 2030** guidelines, all configuration sources must be explicitly declared, strictly authorized, content-hashed (digestible), and reported in a metadata summary. Silent file resolution or magic fallback mechanisms are rejected, and missing required sources must result in immediate admission failure rather than partial loading.

## Key Requirements

1. **Explicit and Allowed Sources**:
   - The engine must reject any ambient or implicit sources that are not explicitly registered in the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191).
   - Config sources must be restricted to an allowed set of variants defined in the loading model, represented by [`SourceKind`](file:///Users/sac/star-toml/src/reports.rs#L16-L25) (e.g., `Str` for built-in defaults/raw strings, `File` for required files, `OptionalFile` for optional files, and `FindFile` for files found by directory traversal).
   - If an unknown, unregistered, or disallowed configuration source is encountered, the loader must immediately abort config admission and return an error.

2. **Digestible Sources**:
   - To guarantee provenance, reproducibility, and auditability, the engine must digest the exact content of every loaded configuration source.
   - For file sources and defaults, a cryptographic content hash (specifically a BLAKE3 hex digest) must be computed over the raw inputs.

3. **Reported Sources (`SourceReport`)**:
   - The final configuration output package, represented by [`AdmittedConfig`](file:///Users/sac/star-toml/src/loader.rs#L1561-L1568), must contain a structured source metadata report, represented by [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56).
   - This report must list all configured sources, recording:
     - The source's label or identifier (`label`).
     - Whether the source was found (`found`).
     - The raw filepath (`path`) as supplied to the loader, if file-based.
     - The calculated BLAKE3 content digest (`digest`), present if found.
     - A boolean flag indicating whether the source was required (`required`).

4. **Failure on Unknown or Missing Required Files**:
   - **Unknown/Disallowed Source**: Attempting to feed an unauthorized source to the loader triggers an admission check failure (e.g., returning an `UnknownSource` validation error).
   - **Missing Required File**: If a config source marked as "required" (such as a file loaded via `layer_file()`) is absent from the filesystem, the engine must fail immediately with a [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) or custom `MissingRequiredFile` variant. It must never silently ignore the missing file, fall back to defaults, or succeed partially.
   - **Missing Optional File**: If a config source is marked optional (e.g., loaded via `layer_file_if_exists()`), it is registered in the [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56) as missing but does not raise an error.

## Acceptance Criteria
- [x] Attempts to load an invalid or missing required file source trigger a compile-time type constraint error or run-time [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) (or equivalent `MissingRequiredFile`) immediately.
- [x] Attempting to configure or inject an unauthorized/unregistered source into the admission pipeline halts execution with a configuration failure.
- [x] Every successful config load operation produces a [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56) that lists all attempted and resolved configuration sources.
- [x] The [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56) includes a verified BLAKE3 content hash/digest for each successfully loaded configuration source.
- [x] If an optional source file is absent, it does not fail admission but is reported as missing in the final report, with no digest generated.
- [x] The loader does not perform any magic discovery of configuration files (e.g. from hardcoded system locations or parent folders) unless explicitly added to the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191).

## Counterexamples Covered
- `implicit_source_used`: Silently loading or falling back to undocumented config files. All configuration origins must be declared in the [Loader](file:///Users/sac/star-toml/src/loader.rs#L136-L191) registry.
- `missing_required_file_not_error`: Missing files marked "required" failing to raise errors. Any required configuration file that is absent must halt the admission pipeline.

## Verification Method

### Unit Tests
- **Allowed Registry Verification**: Call the config loader with an invalid/unregistered source type, and verify that the loader returns a configuration error.
- **Required File Absence**: Call the loader with a required file path that does not exist on disk, and assert that it returns [Error::FileNotFound](file:///Users/sac/star-toml/src/error.rs#L15) (or equivalent `MissingRequiredFile`).
- **Optional File Absence**: Call the loader with an optional file path that does not exist, and assert that the loader completes successfully and records the file status as missing in [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56).
- **Digest Correctness**: Verify that loading a file with known content produces a digest that matches the expected BLAKE3 content hash.

### Integration Tests
- Run the full [TrustedLoader](file:///Users/sac/star-toml/src/loader.rs#L765-L790) builder sequence using a default string, a local file, and an environment override, verifying that the generated [`SourceReport`](file:///Users/sac/star-toml/src/reports.rs#L54-L56) correctly lists all three sources, their paths (or labels), and their calculated digests.

### Chaos Tests
- Delete a required configuration file mid-process or before loading and verify that config admission is rejected.
- Introduce an ambient/implicit config file in the current working directory and verify that the engine ignores it unless explicitly registered.
