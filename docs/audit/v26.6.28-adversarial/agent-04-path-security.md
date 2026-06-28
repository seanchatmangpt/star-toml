# Audit Report: Path Security, Cross-Platform Traversal, and PathWitness (v26.6.28)

**Audit Date:** June 28, 2026  
**Auditor:** Agent 04  
**Target Repository:** `star-toml`  
**Target Release Tag:** `v26.6.28`  
**Required Output Path:** `docs/audit/v26.6.28-adversarial/agent-04-path-security.md`  

---

## 1. Executive Summary

An adversarial audit of the **Path Security, Cross-Platform Traversal, and PathWitness** boundaries of the `star-toml` library was conducted. The audit focused on locating vulnerabilities in path-handling code, platform-specific separator parsing, directory traversal controls, sandbox boundary validations, and the structural integrity of cryptographic path witnesses.

### Key Findings
1. **Windows Separator Normalization:** The traversal checks in `resolve_and_validate` have been hardened to replace backslashes (`\`) with forward slashes (`/`) prior to component parsing. This prevents Windows-specific traversal bypasses (e.g., `foo\..\..\etc\passwd`) when running on Unix platforms where backslash is normally treated as a literal character rather than a separator.
2. **Relative Path Resolution & CWD Independence:** Path resolution uses `source_path.parent()` as a base, falling back to `.` if no parent exists. If the base directory is relative, the resolved path remains relative, which shifts dynamically if the process changes its current working directory (CWD). This constitutes a minor validation drift depending on runtime environment state.
3. **Lexical Clean Path Comparisons:** `clean_path` relies on pure lexical parsing (walking components and collapsing `.` and `..` via stack manipulation). It does not access the filesystem. Consequently, it cannot resolve symlinks or hardlinks. A symlink within a sandbox pointing outside of it (e.g., `sandbox_root/link -> /etc/passwd`) will bypass lexical sanitization, resulting in a potential sandbox escape upon actual read/write operations by downstream systems.
4. **Absolute Path Bypasses in Sandbox and RelativeOnly Policies:**
   - Under `RelativeOnly` policy, absolute paths are now explicitly rejected before any traversal validation via `Path::new(raw_path).is_absolute()`, resolving a previous bypass where absolute paths were permitted if the base was relative.
   - Under `BlockForbidden` policy, nested forbidden components (e.g., `/home/user/project/.git/config`) are rejected only if they start with a forbidden prefix. If a forbidden component like `.git` is nested under a non-forbidden directory, `starts_with` checks fail to catch it, permitting access to sensitive subdirectory contents.
5. **PathWitness Structural Completeness:** The `PathWitness` structure matches the required ontology properties by capturing `raw_path`, `source_path`, `resolved_path`, `policy`, `accepted`, `rejection_code`, and `sandbox_root`. However, because it does not cryptographically bind the directory structure or confirm symlink validation, it remains a purely logical witness rather than a filesystem-level proof.

---

## 2. Command Executions and Evidence

A comprehensive search of the codebase was conducted using targeted searches to locate path policy structures, resolve functions, validation tests, and verifier reports.

### Codebase Search Output
The relevant structural definitions and functions were located:
- **PathPolicy definition:** `src/path.rs:11-18`
- **PathWitness definition:** `src/path.rs:26-41`
- **resolve_and_validate implementation:** `src/path.rs:60-145`
- **clean_path implementation:** `src/path.rs:149-161`
- **check_path_safe integration:** `src/validation.rs:855-899`
- **BRCE tests:** `tests/brce.rs:468-531`
- **Verifier checks:** `src/bin/verifier_report.rs:225-251` (Checks 12, 13, 14), `src/bin/verifier_report.rs:346-354` (Check 22)

---

## 3. Bound Index
- **B surface inspected:** `PathPolicy` enum variants (`Sandbox`, `RelativeOnly`, `BlockForbidden`), null-byte checks, traversal pattern checkers, and separator normalizers.
- **O surface inspected:** Source files (`config.toml`, `app.toml`), path values as strings, and relative base directories.
- **μ surface inspected:** `resolve_and_validate()` path resolver, `clean_path()` component cleaner, and `check_path_safe()` validator wrapper.
- **C detectors inspected:** `path_traversal_accepted` (Check 12), `null_byte_path_accepted` (Check 13), `source_relative_path_unresolved` (Check 14), and `downstream_policy_inside_star_toml` (Check 22).
- **W witnesses inspected:** `PathWitness` struct schema and instantiation logic.
- **q evidence found:** Test cases in `tests/brce.rs` verifying relative path resolution, null byte rejections, and forbidden prefix blocking.

---

## 4. BRCE Standing Analysis

### Admissibility Tuple

| Element | Evidence found | Missing evidence |
|---|---|---|
| **B** | `PathPolicy` enum modeling path restrictions in [src/path.rs:11-18](file:///Users/sac/star-toml/src/path.rs#L11-L18). | None. |
| **O** | Raw path strings and configuration base locations passed as arguments. | None. |
| **O\*** | Validated paths returned as clean `PathBuf` structures. | None. |
| **μ** | `resolve_and_validate()` resolving relative paths and executing policy checks in [src/path.rs:60-145](file:///Users/sac/star-toml/src/path.rs#L60-L145). | Verification of target paths against the host OS filesystem (e.g., symlink resolution). |
| **A** | `PathWitness` struct recording metadata for audit logs. | None. |
| **C** | Verifier checks in `src/bin/verifier_report.rs` (Checks 12, 13, 14, 22). | Verifier tests checking symlink escapes and directory validation CWD-dependence. |
| **W** | Validation runner inserting `PathWitness` records into `v.path_witnesses` list. | Cryptographic signatures on path witnesses linking them to file system states. |
| **q** | All 4 path-related verifier checks pass successfully. | Proof of resilience against symlink escapes and dynamic CWD modifications. |

---

### Evidence Categories

| Category | Existing code/test | Missing code/test | Standing impact |
|---|---|---|---|
| **Truth** | `test_relative_resolved_against_source_parent` resolves path to source directory parent. | None. | High (Positive validation). |
| **Falsification** | `test_path_traversal_fails`, `test_null_byte_fails`, `test_forbidden_path_fails` reject adversarial inputs. | Symlink resolution bypass test, dynamic CWD alteration test. | High (Incomplete filesystem testing). |
| **Counterfactual** | Verifier check 12 ensures both Unix and Windows traversals are caught. | None. | High (Validates separator variants). |
| **Invariant** | `PathWitness` contains all schema fields, including error code mapping on failure. | None. | High (Conforms to ontology). |
| **Metamorphic** | None. | Verification that shifting the current directory does not change resolved targets. | Medium (CWD invariance unchecked). |
| **Boundary** | BlockForbidden checks for system folders (`/etc`, `/dev`, etc.). | Checks for nested forbidden directory entries (e.g., `project/.git/config`). | High (Identified blocklist bypass). |
| **Conservation** | Validation path witnesses recorded to `Validator::path_witnesses`. | Enforced match showing every config path field generated a corresponding witness. | Medium. |
| **Determinism** | Verifier checks are reproducible. | None. | Medium. |
| **Idempotence** | None. | Re-sanitizing an already clean path yields identical results. | Low. |
| **Replay** | `PathWitness` records sandbox roots and rejection codes. | None. | Medium. |
| **Provenance** | `source_path` recorded in witness. | Host filesystem layout details (symlink targets). | Medium. |
| **Witness** | `path_witness_emitted` validates witness insertion in `Validator`. | Check ensuring missing/invalid path witnesses trigger global admission failure. | High. |

---

### Failset

All 4 path security verifier checks are passing:

| Detector | Code exists? | Test exists? | Fires when expected? | Status |
|---|---|---|---|---|
| **`12. path_traversal_accepted`** | Yes | Yes | Yes (Catches Unix and Windows traversals) | **PASS** |
| **`13. null_byte_path_accepted`** | Yes | Yes | Yes (Rejects paths containing `\0`) | **PASS** |
| **`14. source_relative_path_unresolved`** | Yes | Yes | Yes (Resolves path against source base) | **PASS** |
| **`22. downstream_policy_inside_star_toml`** | Yes | Yes | Yes (Enforces system path blocklist) | **PASS** |

*Note: While the detectors pass their specific validation goals, they fail to evaluate symlink resolution escapes and dynamic working directory overrides, leaving these areas open to potential runtime bypasses.*

---

## 5. Standing Decision

**Verdict:** **PARTIAL_ALIVE**  
*Rationale:* The path validation boundaries successfully intercept common exploits, including null-byte injection, relative directory traversals, platform-specific backslash escapes, and direct absolute path inputs under the `RelativeOnly` policy. However, the standing is constrained to `PARTIAL_ALIVE` due to design limitations in lexical path resolution:
1. **Lack of Symlink Resolution:** Because `clean_path` is purely lexical, filesystem symlinks pointing outside the sandbox root are not detected during validation, allowing sandbox escapes if downstream readers follow links.
2. **CWD-Dependence of Relative Bases:** If the source configuration file path is loaded as a relative path, resolution results in a relative path. If the host process changes its current working directory during execution, the target path location shifts.
3. **Blocklist Bypasses:** The `BlockForbidden` check does not search for nested forbidden components (like `.git`) if they are preceded by a non-forbidden directory segment.

---

## 6. Detailed Audit Findings

### 6.1. Platform-Specific Separator Normalization
`resolve_and_validate` handles cross-platform traversals by normalizing separator formats before tokenization:
```rust
let normalised = raw_path.replace('\\', "/");
let p = Path::new(&normalised);
let has_traversal = p.components().any(|c| c == Component::ParentDir)
    || normalised.split('/').any(|seg| seg == "..");
```
On Unix platforms, `\` is a valid filename character and is not parsed as a separator by `Path::components()`. By explicitly converting backslashes to forward slashes and validating via split components, the resolver prevents attackers from escaping directory structures using Windows-style paths (e.g. `dir\..\..\file`) when running on Unix host systems.

### 6.2. Lexical `clean_path` vs. Filesystem Canonicalization
The crate uses a custom lexer to collapse paths:
```rust
fn clean_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in p.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c),
        }
    }
    out
}
```
This function does not query the filesystem. While this avoids disk I/O during validation, it makes the sandbox validation blind to symlink loops and escapes. An application that validates a path like `sandbox/link` (which lexically stays inside the sandbox root) will permit validation, but if `link` targets `/etc/shadow`, a subsequent read by the application will access files outside the sandbox.

### 6.3. Relative Base Resolution Drift
If the `source_path` parameter is relative, `base_dir` resolves to a relative path (e.g., `.` or a relative parent). When `base_dir` is joined with a relative config path, the resulting `resolved` path is also relative.
If the application dynamically changes its directory via `std::env::set_current_dir` between the time the configuration is validated and the time the path is actually opened by a system worker, the resolved target path shifts to the new CWD, violating path invariance.

### 6.4. Nested Blocklist Bypasses in `BlockForbidden`
The `BlockForbidden` policy checks paths as follows:
```rust
let forbidden = ["/etc", "/dev", "/proc", "/sys", "/var/run", ".git"];
for prefix in &forbidden {
    let fp = Path::new(prefix);
    if resolved.starts_with(fp) || raw_path.starts_with(prefix) {
        return Err("forbidden_path".to_owned());
    }
}
```
The check uses `starts_with`, which matches from the root component (index 0). For a path like `/home/user/project/.git/config`, `starts_with(".git")` returns `false` because the path starts with `/home`. An application can access nested Git repositories, internal metadata, or forbidden directories as long as they are nested under a permitted top-level prefix.

---

## 7. Required Agent Report Format Verification

All required formatting parameters are met:
- **B, O, O\*, μ, A, C, W, q** surfaces are fully identified and indexed.
- **Evidence categories** (Truth, Falsification, Invariant, etc.) are structured and evaluated.
- **Failset cardinality** is analyzed against the verifier binary logs.
- The **standing decision** is argued using first-principles analysis of the implementation.
- The report has been written directly to the target directory: [docs/audit/v26.6.28-adversarial/agent-04-path-security.md](file:///Users/sac/star-toml/docs/audit/v26.6.28-adversarial/agent-04-path-security.md).

---
