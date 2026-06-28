# Path Policy and PathWitness Audit Report

## Bound Index
- B surface inspected: `PathPolicy` (Sandbox, RelativeOnly, BlockForbidden), Null byte check, traversal limits, forbidden prefixes.
- O surface inspected: Config paths, source paths, filesystem state.
- μ surface inspected: `resolve_and_validate()`, `clean_path()`, `check_path_safe()`.
- C detectors inspected: `path_traversal_accepted`, `null_byte_path_accepted`, `source_relative_path_unresolved`, `forbidden_path_accepted`.
- W witnesses inspected: `PathWitness` struct schema and generation.
- q evidence found: `tests/brce.rs::test_path_traversal_fails`, `tests/brce.rs::test_null_byte_fails`, `tests/brce.rs::test_forbidden_path_fails`, `tests/brce.rs::test_relative_resolved_against_source_parent`, `tests/brce.rs::test_path_witness_emitted`, `tests/adversarial.rs::test_path_adversarial`.

---

## BRCE Standing Analysis

### Admissibility Tuple
| Element | Evidence found | Missing evidence |
|---|---|---|
| B | Code in [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L11-L18) and [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L75-L85) checking null bytes, `..` traversals, and forbidden prefixes. | Lack of Windows backslash parsing on Unix; lack of symlink resolution (`std::fs::canonicalize` bypass); lack of nested components checks; lack of absolute path rejection under `RelativeOnly` when base is relative. |
| O | Path string config values, `source_path` in `Validator` ([src/validation.rs](file:///Users/sac/star-toml/src/validation.rs#L859)). | Guarantee of absolute/canonical source paths in loader layer configuration. |
| O* | Validated config values when `check_path_safe` succeeds. | Enforced type validation that path fields *must* go through `check_path_safe` instead of standard strings. |
| μ | `resolve_and_validate()` and `clean_path()` in [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L56). | File system verification and symlink target validation. |
| A | `PathWitness` struct in [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L26-L37). | Ontology-required `rejection_code` field and Sandbox `root` parameter. |
| C | Verifier check 12, 13, 14, 22 in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs). | Tests/detectors for Windows backslash on Unix, symlink sandbox escapes, and relative `source_path` base-directory bypasses. |
| W | `v.path_witnesses` list inside `Validator` ([src/validation.rs](file:///Users/sac/star-toml/src/validation.rs#L554)). | Complete witnesses recording sandbox roots and failure reason codes. |
| q | Partial standing proof for basic paths (defaults passing). | Full standing proof ($q = 0$ since multiple falsifiers are active). |

### Evidence Categories
| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| Truth | [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L498-L515) tests basic relative resolution and witness generation. | None. | High (Positive validation). |
| Falsification | [tests/brce.rs](file:///Users/sac/star-toml/tests/brce.rs#L468-L496) tests standard traversal, null bytes, and forbidden paths. | No tests for symlink escapes, Windows separator traversals on Unix, absolute paths under `RelativeOnly` with relative sources, and nested forbidden paths. | Critical (Vulnerabilities untested). |
| Counterfactual | Verifier checks 12, 13, 14, 22 in [src/bin/verifier_report.rs](file:///Users/sac/star-toml/src/bin/verifier_report.rs). | Verification that changing CWD dynamically alters path resolution when `source_path` is relative. | High (Resolution CWD-dependence). |
| Invariant | `PathWitness` struct matches fields. | Path witness does not include `rejection_code` (violating `star-toml.core.ttl`). | Critical (Ontology violation). |
| Metamorphic | None. | Verification that CWD shift does not alter resolved paths. | High (CWD invariance fails). |
| Boundary | Blocklist check in `BlockForbidden`. | Test checking nested forbidden paths like `src/.git` or `/path/.git`. | High (Blocklist bypasses). |
| Conservation | None. | Verification that every configuration path field maps to a generated witness. | Medium (Missing conservation). |
| Determinism | `verifier_report` check 15/21. | Path witness determinism tests. | Medium (Determinism unchecked). |
| Idempotence | None. | Re-resolving already resolved paths is idempotent. | Low. |
| Replay | None. | Replaying path safety checks from `PathWitness` alone is impossible (missing Sandbox root parameter). | High (Replay impossible). |
| Provenance | `source_path` in `PathWitness`. | Incomplete path source location bounds. | Medium. |
| Witness | Witness generation code in [src/validation.rs](file:///Users/sac/star-toml/src/validation.rs#L863). | Enforcing that missing path witnesses prevent terminal admission. | High. |

### Failset
| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| **`path_traversal_accepted`** | Yes | Yes | **NO**. Fails to detect Windows backslash separator traversals on Unix platforms (e.g., `foo\\..\\bar`), and fails to detect symlink escapes pointing outside sandboxes because no filesystem canonicalization is run. | **ACTIVE** (Vulnerable) |
| **`null_byte_path_accepted`** | Yes | Yes | Yes. Screened immediately in [src/path.rs:76-78](file:///Users/sac/star-toml/src/path.rs#L76-L78). | **INACTIVE** (Blocked) |
| **`source_relative_path_unresolved`** | Yes | Yes | **NO**. If the configuration source file is loaded via a relative path (e.g., `config.toml`), the resolution base is relative, causing the resolved path to depend on the process CWD when accessed or run, violating CWD-invariance. | **ACTIVE** (Vulnerable) |
| **`forbidden_path_accepted`** (absolute path accepted under forbidden policy) | Yes | Yes | **NO**. <br>1. Absolute paths like `/etc/passwd` are accepted under `RelativeOnly` policy if `source_path` is relative, because `resolved.starts_with("")` returns `true`. <br>2. Nested forbidden components like `/home/user/project/.git/config` bypass `BlockForbidden` because `starts_with` only checks components starting at index 0. | **ACTIVE** (Vulnerable) |
| **`PathWitness missing evidence fields`** | No | No | **NO**. `PathWitness` struct in [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L26-L37) lacks the required `rejection_code` (or error code) and does not record the sandbox `root` parameters, violating the ontology. | **ACTIVE** (Vulnerable) |

### Standing Decision
- **PARTIAL_ALIVE**: Some q inputs exist, but witness/lifecycle/detector coverage is incomplete, and multiple falsifiers are active.

---

## Detailed Audit Findings & Active Falsifiers

### 1. Falsifier: Path Traversal Accepted
#### Vulnerability 1.1: Platform-Specific Separator Bypass on Unix
- **Mechanism**: The traversal check in `resolve_and_validate` ([src/path.rs:81-85](file:///Users/sac/star-toml/src/path.rs#L81-L85)) is:
  ```rust
  let p = Path::new(raw_path);
  let has_traversal = p.components().any(|c| c == Component::ParentDir);
  ```
  On Unix/macOS, backslash (`\`) is not treated as a directory separator by the standard library. Thus, a path like `foo\\..\\bar` is parsed as a single component name `"foo\..\bar"` rather than containing `Component::ParentDir`. This bypasses the check. However, if this path is later resolved on Windows or processed by an application normalizing backslashes, it will perform a directory traversal escape.
- **Evidence Reference**: [src/path.rs:81-85](file:///Users/sac/star-toml/src/path.rs#L81-L85).

#### Vulnerability 1.2: Symlink Sandbox Escape
- **Mechanism**: The policy checks (Sandbox, RelativeOnly) in `resolve_and_validate` ([src/path.rs:109-122](file:///Users/sac/star-toml/src/path.rs#L109-L122)) are performed on a path processed by `clean_path` ([src/path.rs:131-143](file:///Users/sac/star-toml/src/path.rs#L131-L143)).
  `clean_path` is a purely lexical path cleaner that does not resolve symlinks or check the filesystem. If a symlink exists inside the sandbox pointing to `/etc/passwd` (e.g. `sandbox/symlink -> /etc/passwd`), `clean_path` yields `sandbox/symlink`, which begins with the sandbox root and is accepted. Upon actual access, the OS resolves the symlink, resulting in an audited sandbox escape.
- **Evidence Reference**: [src/path.rs:96](file:///Users/sac/star-toml/src/path.rs#L96), [src/path.rs:131-143](file:///Users/sac/star-toml/src/path.rs#L131-L143).

### 2. Falsifier: Relative Path Resolved against CWD instead of Source Parent
- **Mechanism**: When a configuration file is loaded via a relative path (e.g. `config.toml`), the `source_path` passed to `resolve_and_validate` is relative.
  - `source_path.parent()` yields `Some("")`.
  - `base_dir` resolves to `""`.
  - `resolved = base_dir.join(raw_path)` yields a relative path.
  - No absolute path conversion or canonicalization is performed in `resolve_and_validate`.
  - If the application dynamically changes its current working directory (CWD) via `std::env::set_current_dir` during execution, the resolved path shifts relative to the new CWD when evaluated or accessed, violating CWD-invariance.
- **Evidence Reference**: [src/path.rs:88-93](file:///Users/sac/star-toml/src/path.rs#L88-L93).

### 3. Falsifier: Absolute Path Accepted under Forbidden Policy
#### Vulnerability 3.1: Absolute Path Accepted under `RelativeOnly` Policy
- **Mechanism**: If `source_path` is relative (e.g., `config.toml`), `base_dir` becomes `""`.
  Under `RelativeOnly` policy ([src/path.rs:116-122](file:///Users/sac/star-toml/src/path.rs#L116-L122)), the check is:
  ```rust
  let clean_base = clean_path(base_dir); // clean_base = ""
  if !resolved.starts_with(&clean_base) { ... }
  ```
  If an absolute path like `/etc/passwd` is passed, `resolved` is `/etc/passwd`. In Rust, `"/etc/passwd".starts_with("")` returns `true` because any path starts with the empty path. Thus, `/etc/passwd` is accepted under `RelativeOnly` policy, completely bypassing the safety policy.
- **Evidence Reference**: [src/path.rs:117-122](file:///Users/sac/star-toml/src/path.rs#L117-L122).

#### Vulnerability 3.2: Nested Forbidden Components Bypass `BlockForbidden`
- **Mechanism**: Under `BlockForbidden` policy ([src/path.rs:100-108](file:///Users/sac/star-toml/src/path.rs#L100-L108)), the check is:
  ```rust
  if resolved.starts_with(fp) || raw_path.starts_with(prefix)
  ```
  `Path::starts_with` only checks if the path begins with the specified components. For a forbidden component like `".git"`, a path like `/home/user/project/.git/config` does not start with `.git` (it starts with `/home`), and is therefore accepted. This allows access to forbidden system folders or git configuration files as long as they are nested under some prefix.
- **Evidence Reference**: [src/path.rs:104-106](file:///Users/sac/star-toml/src/path.rs#L104-L106).

### 4. Falsifier: PathWitness Missing Evidence Fields
- **Mechanism**:
  1. The `PathWitness` struct in [src/path.rs](file:///Users/sac/star-toml/src/path.rs#L26-L37) does not record the `rejection_code` (or error/validation code) upon failure, preventing auditing of the failure cause (violating the `star-toml.core.ttl` ontology).
  2. The `policy` field in the witness is stored as a `String` representing only the name (e.g. `"Sandbox"`). The specific sandbox `root` parameter in `PathPolicy::Sandbox { root }` is not saved. Therefore, independent verifiers cannot determine or replay which sandbox root boundaries the path was validated against.
  3. `resolved_path` is represented as an `Option<PathBuf>` rather than the required absolute `PathBuf` specified in the design constraints.
- **Evidence Reference**: [src/path.rs:26-37](file:///Users/sac/star-toml/src/path.rs#L26-L37), [src/path.rs:67-73](file:///Users/sac/star-toml/src/path.rs#L67-L73).
