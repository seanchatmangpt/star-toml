//! Path policy enforcement with source-relative resolution and audit witnesses.

use std::path::{Component, Path, PathBuf};

// ---------------------------------------------------------------------------
// PathPolicy
// ---------------------------------------------------------------------------

/// Policy controlling which paths are acceptable during validation.
#[derive(Debug, Clone)]
pub enum PathPolicy {
    /// Path must resolve to within `root` (sandbox).
    Sandbox { root: PathBuf },
    /// Path must not escape the directory that contains the source file.
    RelativeOnly,
    /// Path must not begin with a forbidden system prefix.
    BlockForbidden,
}

// ---------------------------------------------------------------------------
// PathWitness
// ---------------------------------------------------------------------------

/// Audit record produced by a successful or failed path validation.
#[derive(Debug, Clone)]
pub struct PathWitness {
    /// The raw string as supplied by the caller.
    pub raw_path: String,
    /// The source file that anchored relative resolution.
    pub source_path: PathBuf,
    /// The fully-resolved absolute path (if resolution succeeded).
    pub resolved_path: Option<PathBuf>,
    /// The policy that was applied.
    pub policy: String,
    /// `true` if this path was accepted by the policy.
    pub accepted: bool,
}

// ---------------------------------------------------------------------------
// resolve_and_validate
// ---------------------------------------------------------------------------

/// Resolve `raw_path` relative to `source_path.parent()` and enforce `policy`.
///
/// Returns `(resolved_path, witness)` on success, or a string error code on failure.
///
/// # Error codes
///
/// | Code | Cause |
/// |------|-------|
/// | `null_byte_detected` | `raw_path` contains a `\0` byte |
/// | `path_traversal_detected` | `raw_path` contains a `..` component |
/// | `forbidden_path` | path begins with a blocked system prefix |
/// | `sandbox_escape` | resolved path is outside the sandbox root |
/// | `relative_only_escape` | resolved path escapes the source directory |
pub fn resolve_and_validate(
    raw_path: &str,
    source_path: &Path,
    policy: &PathPolicy,
) -> Result<(PathBuf, PathWitness), String> {
    let policy_name = match policy {
        PathPolicy::Sandbox { .. } => "Sandbox",
        PathPolicy::RelativeOnly => "RelativeOnly",
        PathPolicy::BlockForbidden => "BlockForbidden",
    };

    let make_witness = |resolved: Option<PathBuf>, accepted: bool| PathWitness {
        raw_path: raw_path.to_owned(),
        source_path: source_path.to_path_buf(),
        resolved_path: resolved,
        policy: policy_name.to_owned(),
        accepted,
    };

    // Reject null bytes
    if raw_path.contains('\0') {
        return Err("null_byte_detected".to_owned());
    }

    // Reject `..` components
    let p = Path::new(raw_path);
    let has_traversal = p.components().any(|c| c == Component::ParentDir);
    if has_traversal {
        return Err("path_traversal_detected".to_owned());
    }

    // Resolve relative to source_path.parent()
    let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    };

    // Canonicalize-ish: clean without requiring existence by walking components
    let resolved = clean_path(&resolved);

    // Policy enforcement
    match policy {
        PathPolicy::BlockForbidden => {
            let forbidden = ["/etc", "/dev", "/proc", "/sys", "/var/run", ".git"];
            for prefix in &forbidden {
                let fp = Path::new(prefix);
                if resolved.starts_with(fp) || raw_path.starts_with(prefix) {
                    return Err("forbidden_path".to_owned());
                }
            }
        }
        PathPolicy::Sandbox { root } => {
            let clean_root = clean_path(root);
            if !resolved.starts_with(&clean_root) {
                let w = make_witness(Some(resolved), false);
                return Err(format!("sandbox_escape:{}", w.raw_path));
            }
        }
        PathPolicy::RelativeOnly => {
            let clean_base = clean_path(base_dir);
            if !resolved.starts_with(&clean_base) {
                let w = make_witness(Some(resolved), false);
                return Err(format!("relative_only_escape:{}", w.raw_path));
            }
        }
    }

    let witness = make_witness(Some(resolved.clone()), true);
    Ok((resolved, witness))
}

/// Lexically clean a path (resolve `.` and collapse double slashes) without
/// hitting the filesystem.
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
