# ST-108: Path Bounds Implementation Instructions

This document provides formal implementation instructions for implementing the Path Bounds security subsystem (`star_toml::path`) within the `star-toml` crate. It adheres strictly to the **Bounded Receipted Chatman Equation (BRCE)** standing framework.

---

## 1. The Admissibility Tuple

The filesystem path validation boundary is defined by the following components of the Admissibility Tuple:

$$\text{AdmissibilityTuple} = \langle B, O, O^*, \mu, A, C, W, q \rangle$$

### $B$ (Boundary Constraints)
- **Path Traversal Guard**: Lexical parent directory markers (`..`) are strictly forbidden. Resolved path canonicalization must verify that the path does not escape the allowed directory scope.
- **Null Byte Blocker**: Strings containing C-style null byte characters (`\0` or `0x00`) are immediately rejected.
- **Forbidden Paths Blocklist**: Rejects paths matching or nested within `/etc/`, `/dev/`, `/proc/`, `/sys/`, `/var/run/`, `.git/`, or specific files like `/etc/passwd`.
- **Absolute/Relative Constraints**: Standardizes absolute paths, or enforces relative paths when specified by the policy.

### $O$ (Observations)
- Raw input path strings parsed from the TOML configuration files.
- The `source_path` representing the absolute path of the configuration file itself.

### $O^*$ (Admitted Config)
- Configuration values that have been validated, resolved, and recorded under a valid `PathWitness`.

### $\mu$ (Lawful Transformations)
- `resolve_and_validate(raw_path: &str, source_path: &Path, policy: &PathPolicy) -> Result<PathBuf, ValidationError>`
  1. Scan `raw_path` for null bytes (`\0`). If found, reject immediately with `null_byte_detected`.
  2. Scan `raw_path` components lexically. If any component is `..`, reject immediately with `path_traversal_detected`.
  3. Resolve relative path: `resolved_path = source_path.parent() / raw_path`. Do **not** use the process's current working directory (`CWD`).
  4. Perform logical path normalization (resolving `.` and redundant separators without disk access).
  5. If the path exists on disk, canonicalize it (`std::fs::canonicalize`) to resolve symlinks and ensure the real target does not escape the policy scope.
  6. Enforce Policy Scope:
     - `RelativeOnly`: Ensure the path remains relative to the source configuration file's parent directory and doesn't point outside it.
     - `Sandbox { root }`: Ensure the canonicalized path starts with `root`.
     - `BlockForbidden`: Ensure the canonicalized path does not start with any forbidden prefix.

### $A$ (Artifacts)
- [PathWitness](file:///Users/sac/star-toml/src/path.rs#L30) (struct schema details below).
- [ValidationError](file:///Users/sac/star-toml/src/validation.rs#L302) containing precise `loc`, `kind`, and `severity`.

### $C$ (Counterexamples / Detectors)
- `path_traversal_accepted`: Path escapes the allowed base boundaries using lexical/logical traversal.
- `null_byte_path_accepted`: System fails to reject null byte injections.
- `source_relative_path_unresolved`: Relative path resolves against process CWD instead of config source parent.
- `forbidden_path_accepted`: System permits resolving paths within blocked directories.

### $W$ (Witnesses)
- A collection of `PathWitness` records accumulated during validation.

### $q$ (Standing Bit)
- $q_{path} = 1 \iff \text{BoundSatisfied}(B, O) \wedge \text{TransformLawful}(\mu) \wedge \text{WitnessComplete}(W) \wedge \text{CounterexamplesAbsent}(C)$

---

## 2. PathWitness Data Schema

Define the following data structures in the new file [src/path.rs](file:///Users/sac/star-toml/src/path.rs):

```rust
use std::path::PathBuf;

/// The security policy to apply to path validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum PathPolicy {
    /// Sandboxed: the resolved path must lie strictly within the designated root.
    Sandbox { root: PathBuf },
    /// Relative only: the path must be lexically relative and must not traverse outside the source parent.
    RelativeOnly,
    /// Block forbidden: checks null bytes, directory traversal, and forbidden system components.
    BlockForbidden,
}

/// A structured receipt audit record for all configuration path checks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PathWitness {
    /// The original raw path string from the configuration file.
    pub raw_path: String,
    /// The absolute path of the configuration file defining the path.
    pub source_path: PathBuf,
    /// The canonicalized, absolute resolved path on the filesystem.
    pub resolved_path: PathBuf,
    /// The specific safety policy applied.
    pub policy: PathPolicy,
    /// Whether the path successfully passed validation.
    pub accepted: bool,
}
```

---

## 3. Target Files & Symbols to Create/Change

### File 1: [src/path.rs](file:///Users/sac/star-toml/src/path.rs) (New File)
- **Symbols**:
  - `pub enum PathPolicy`
  - `pub struct PathWitness`
  - `pub fn resolve_and_validate(raw_path: &str, source_path: &Path, policy: &PathPolicy) -> Result<PathBuf, String>`
- **Logic**:
  - Implement blocklist checks (`/etc`, `/dev`, `/proc`, `/sys`, `/var/run`, `.git`).
  - Scan for `\0` and lexical `..`.
  - Canonicalize and match prefixes depending on `PathPolicy`.

### File 2: [src/lib.rs](file:///Users/sac/star-toml/src/lib.rs) (Modify)
- **Symbols**:
  - `pub mod path;`
  - Re-export `PathPolicy` and `PathWitness` in the crate root.

### File 3: [src/validation.rs](file:///Users/sac/star-toml/src/validation.rs) (Modify)
- **Symbols**:
  - Update `Validator` to include:
    - `source_path: Option<PathBuf>`
    - `pub path_witnesses: Vec<PathWitness>`
  - Update [Validator::check_path](file:///Users/sac/star-toml/src/validation.rs#L773) or create `pub fn check_path_safe(&mut self, field: &str, value: &str, policy: PathPolicy)`:
    - Perform validation using `star_toml::path::resolve_and_validate`.
    - If valid, record a successful `PathWitness`.
    - If invalid, record an error with stable codes like `"path_traversal_detected"`, `"null_byte_detected"`, or `"forbidden_path"`, and store an unsuccessful `PathWitness`.

### File 4: [src/loader.rs](file:///Users/sac/star-toml/src/loader.rs) (Modify)
- **Symbols**:
  - Update the validator execution inside `Config<Deserialized<T>>::validate` to set the source path:
    ```rust
    let mut v = crate::validation::Validator::new();
    v.set_source_path(self.path.clone());
    ```
  - Pass the resulting `v.path_witnesses` into the `ValidationReport` or the final `AdmittedConfig` structure.

---

## 4. Tests to Write

Implement these integration tests inside a new test suite: [tests/path_bounds.rs](file:///Users/sac/star-toml/tests/path_bounds.rs) or update [tests/adversarial.rs](file:///Users/sac/star-toml/tests/adversarial.rs).

1. `test_path_traversal_fails`: Assert that paths like `../../etc/passwd` or `dir/../escaping` are rejected and fail with code `path_traversal_detected`.
2. `test_null_byte_fails`: Assert that paths with `\0` (e.g., `config\0.toml`) are rejected immediately with code `null_byte_detected`.
3. `test_forbidden_absolute_path_fails`: Assert that absolute paths to system dirs (like `/etc/passwd` or `/dev/null`) are blocked under `BlockForbidden` and `Sandbox` policies.
4. `test_source_escape_fails`: Verify that symlinks resolving to paths outside the sandbox or configuration source folder fail validation.
5. `test_relative_resolved_against_source_parent`: Assert that a relative path `./db/sqlite.db` defined in `/opt/app/config.toml` resolves precisely to `/opt/app/db/sqlite.db`, independent of the process CWD.
6. `test_path_witness_emitted_for_checked_path`: Verify that checking a path successfully generates a `PathWitness` with `accepted: true`, correct resolved paths, and the exact policy applied.

---

## 5. Counterexamples Killed

- **`path_traversal_accepted`**: Blocked by scanning path components for lexical parent markers `..` and verifying resolved canonical path starts with the allowed root directory.
- **`null_byte_path_accepted`**: Blocked by screening for the exact character `\0` prior to any filesystem/std::path operations.
- **`source_relative_path_unresolved`**: Blocked by making relative paths resolve explicitly with respect to `source_path.parent()` and forcing `source_path` to be provided.

---

## 6. Acceptance Criteria

- [ ] Path validation fails if the path contains lexical `..` components.
- [ ] Path validation fails if the path contains `\0` null bytes.
- [ ] Attempting to resolve paths pointing to forbidden system directories fails validation with descriptive messages.
- [ ] Relative paths resolve relative to the source configuration file's parent directory, completely ignoring process CWD.
- [ ] Successful validation generates and records a `PathWitness` struct with accurate metadata.
- [ ] Path check failures map to stable error codes: `"path_traversal_detected"`, `"null_byte_detected"`, or `"forbidden_path"`.

---

## 7. Falsifiers & Metamorphic Stability

- **CWD Shift Falsifier**: Ensure that spawning a test, changing the working directory using `std::env::set_current_dir`, and loading a config with relative paths resolves to the correct path relative to the config file (i.e. resolved path is invariant under CWD shift).
- **Symlink Escape Falsifier**: Create a symlink in the sandbox pointing to `/etc/passwd`. Attempting to load/validate this path must fail validation when it resolves to the forbidden path.

---

## 8. Dependencies
- Standard library features (`std::path`, `std::fs`).
- Serialization support via `serde`.

---

## 9. Stop Condition
- Subsystem compiles successfully, integration tests pass, and zero failures remain in the counterexample detector set.
- `cargo test` runs and verifies all target behaviors.
