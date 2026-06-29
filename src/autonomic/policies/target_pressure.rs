//! AC1: TargetPressurePolicy
//! Executes `cargo clean` when target/ directory exceeds a size limit.

use std::fs;
use std::path::Path;

/// Configuration for the TargetPressurePolicy.
#[derive(Debug, Clone)]
pub struct TargetPressureConfig {
    /// Maximum size of target/ directory in bytes (default: 1 GB).
    pub limit_bytes: u64,
    /// Path to the project root containing target/ directory.
    pub project_root: String,
}

impl Default for TargetPressureConfig {
    fn default() -> Self {
        Self {
            limit_bytes: 1_000_000_000, // 1 GB
            project_root: String::from("."),
        }
    }
}

/// Calculates the total size of a directory recursively.
fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size += dir_size(&entry.path())?;
        } else {
            size += metadata.len();
        }
    }
    Ok(size)
}

/// Checks if target/ exceeds the limit and needs cleaning.
pub fn check_pressure(config: &TargetPressureConfig) -> bool {
    let target_path = Path::new(&config.project_root).join("target");
    if !target_path.exists() {
        return false;
    }

    match dir_size(&target_path) {
        Ok(size) => size > config.limit_bytes,
        Err(_) => false,
    }
}

/// Executes the policy: runs `cargo clean` if pressure exceeds limit.
pub fn execute(config: &TargetPressureConfig, apply: bool) -> Result<(), String> {
    if !check_pressure(config) {
        return Ok(()); // No action needed
    }

    if apply {
        // Actually run cargo clean
        crate::autonomic::subprocess::run_with_timeout("cargo", &["clean"], false)
            .map(|_| ())
    } else {
        // Just report what would happen
        eprintln!("[TargetPressurePolicy] Would run: cargo clean");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_pressure_config_default() {
        let config = TargetPressureConfig::default();
        assert_eq!(config.limit_bytes, 1_000_000_000);
        assert_eq!(config.project_root, ".");
    }

    #[test]
    fn test_check_pressure_nonexistent() {
        let config = TargetPressureConfig {
            limit_bytes: 1000,
            project_root: "/nonexistent/path".to_string(),
        };
        assert!(!check_pressure(&config));
    }

    #[test]
    fn test_execute_dry_run() {
        let config = TargetPressureConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
