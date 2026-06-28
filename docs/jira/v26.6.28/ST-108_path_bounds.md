# ST-108: Path Bounds

## Description
Enforces strict filesystem safety constraints on all path-bearing fields in configuration files. This ticket implements the path bounds sub-system (`star_toml::path`), ensuring that path references do not lead to directory traversals, security escapes (such as null bytes), or access to forbidden system paths. It also guarantees that relative paths are always resolved with respect to the source configuration file rather than the current working directory, and generates a structured `PathWitness` to audit all path admission decisions.

## Key Requirements
1. **Path Traversal Guard**:
   - Paths must be validated to prevent directory traversal attacks.
   - Any path containing parent directory markers (`..`) or resolving to a location outside the allowed config directory scope must be rejected with a `PathTraversalDetected` error.
2. **Null Byte Prevention**:
   - Paths must be checked for C-style null byte terminations or characters (`\0` or `0x00`).
   - If a null byte is detected anywhere in the raw path string, the system must immediately reject the path with a validation error to prevent null-byte injection attacks.
3. **Forbidden Paths Restriction**:
   - The engine must enforce a blocklist of forbidden paths and system directories (e.g., `/etc/`, `/dev/`, `/proc/`, `/sys/`, `/var/run/`, `.git/`, etc.).
   - Access to paths containing these components or pointing directly to them must be blocked.
4. **Source-Relative Resolution**:
   - All relative paths specified within a configuration file must be resolved relative to the directory containing the source configuration file (i.e. `resolved_path = source_file.parent() / raw_path`).
   - Resolving relative paths relative to the process's ambient current working directory (CWD) is strictly forbidden.
5. **PathWitness Generation**:
   - The validation process must output a structured `PathWitness` representation for every path evaluated.
   - The `PathWitness` struct must contain:
     - `raw_path`: The original raw string from the config file.
     - `source_path`: The absolute path of the configuration file defining the path.
     - `resolved_path`: The canonicalized, absolute resolved path on the filesystem.
     - `policy`: The specific safety policy applied (e.g., sandbox restrictions, relative-only).
     - `accepted`: A boolean indicating whether the path passed validation.

## Acceptance Criteria
- [ ] Any configuration value representing a path that contains directory traversal segments (`..`) is rejected during the path validation phase.
- [ ] Any configuration path value containing a null byte (`\0`) is immediately rejected.
- [ ] Attempting to resolve paths pointing to forbidden system directories (e.g. `/etc/passwd`) fails validation.
- [ ] Relative paths are resolved using the parent directory of the configuration file as the base, not the process's current working directory (CWD).
- [ ] Path validation successfully generates a `PathWitness` structure containing the correct `raw_path`, `source_path`, `resolved_path`, `policy`, and `accepted` status.
- [ ] The validation fails with path-precise error diagnostic mappings including stable variant fingerprints (e.g., `PathTraversalDetected`).

## Counterexamples Covered
- `path_traversal_accepted`: Path pointing outside source root / using `..` to escape sandbox boundaries.
- `null_byte_path_accepted`: Failing to reject C-style null byte characters, leading to potential path truncation and file-system level security bypasses.
- `source_relative_path_unresolved`: Resolving relative paths based on the current working directory of the executing process instead of the actual configuration source file's directory.

## Verification Method
- **Unit Tests**:
   - Test `star_toml::path` with relative paths (`./assets/img.png`), verifying they resolve correctly relative to the source configuration file's parent path.
   - Test input paths with null bytes (e.g., `config\0.toml`, `/tmp/file\0`) and ensure they return a specific validation error.
   - Test input paths with traversal patterns (`../../etc/passwd`, `some/path/../other`) and verify they are rejected.
   - Verify `PathWitness` is correctly populated with the original raw path, source file path, and resolved path, and that its `accepted` field is accurately set.
- **Integration Tests**:
   - Verify that loading a TOML file with a relative path resolves to the expected path inside the test directory, even if the current working directory of the test runner is changed.
- **Chaos & Safety Audits**:
   - Attempt to bypass path bounds by passing symlinks that resolve to forbidden paths or using nested `..` constructs, verifying that the validation framework halts admission and reports path-precise errors.
