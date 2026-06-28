# ST-108: Path Bounds
# ST-108: Path Bounds (Lexical Sandboxing)

## Description
Enforces lexical containment constraints on all path-bearing fields in configuration files. This ticket implements the path bounds sub-system (`star_toml::path`), ensuring that path references do not lexically lead to directory traversals, security escapes (such as null bytes), or prefix matches on forbidden system paths. It also guarantees that relative paths are resolved with respect to the source configuration file's parent directory rather than the current working directory, and generates a structured `PathWitness` to audit path evaluation. Note: Validation is purely lexical and does not access the filesystem or resolve filesystem-level symlinks; thus, it does not protect against symlink-based sandbox escapes.

### Key Requirements
1. **Lexical Path Traversal Guard**:
   - Paths must be validated lexically to prevent directory traversal patterns.
   - Any path containing parent directory markers (`..`) or resolving lexically to a location outside the allowed config directory scope must be rejected with a `path_traversal_detected` error.
2. **Null Byte Prevention**:
   - Paths must be checked for C-style null byte terminations or characters (`\0` or `0x00`).
   - If a null byte is detected anywhere in the raw path string, the system must immediately reject the path with a validation error to prevent null-byte injection attacks.
3. **Forbidden Paths Restriction**:
   - The engine must enforce a lexical blocklist of forbidden paths and system directories (e.g., `/etc/`, `/dev/`, `/proc/`, `/sys/`, `/var/run/`, `.git/`, etc.).
   - Access to paths containing these components or pointing directly to them (lexically) must be blocked. Note that `BlockForbidden` prefix matching uses `starts_with` which checks from index 0, allowing nested forbidden components (e.g., `/home/user/project/.git/config`) to pass validation.
4. **Source-Relative Resolution**:
   - All relative paths specified within a configuration file must be resolved relative to the directory containing the source configuration file (i.e. `resolved_path = source_file.parent() / raw_path`).
   - Resolving relative paths relative to the process's ambient current working directory (CWD) is strictly forbidden. Note that if the source configuration file path is itself relative, the resolved path remains relative, making it sensitive to CWD shifts.
5. **PathWitness Generation**:
   - The validation process must output a structured `PathWitness` representation for every path evaluated.
   - The `PathWitness` struct must contain:
     - `raw_path`: The original raw string from the config file.
     - `source_path`: The path of the configuration file defining the path.
     - `resolved_path`: The resolved path as `Option<PathBuf>` (None if validation failed).
     - `policy`: The specific safety policy applied represented as `String` (e.g. `"Sandbox"`, `"RelativeOnly"`, `"BlockForbidden"`).
     - `accepted`: A boolean indicating whether the path passed validation.
     - `rejection_code`: Rejection error code as `Option<String>`.
     - `sandbox_root`: The sandbox root enforced as `Option<PathBuf>`.

## Acceptance Criteria
- [x] Any configuration value representing a path that contains directory traversal segments (`..`) is rejected during the path validation phase.
- [x] Any configuration path value containing a null byte (`\0`) is immediately rejected.
- [x] Attempting to resolve paths pointing lexically to forbidden system directories (e.g. `/etc/passwd`) fails validation.
- [x] Relative paths are resolved using the parent directory of the configuration file as the base, not the process's current working directory (CWD).
- [x] Path validation successfully generates a `PathWitness` structure containing the correct `raw_path`, `source_path`, `resolved_path`, `policy`, and `accepted` status.
- [x] The validation fails with path-precise error diagnostic mappings including stable variant fingerprints (e.g., `path_traversal_detected`).

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
   - Attempt to bypass path bounds by passing lexical traversal constructs or forbidden prefixes, verifying that the validation framework halts admission and reports path-precise errors. Note and document the physical filesystem symlink bypass limitation.
