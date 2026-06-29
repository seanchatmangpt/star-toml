//! AC3: TrybuildChangedPolicy
//! Executes `cargo test --test changed` when trybuild files change.

use std::fs;
use std::path::Path;

/// Configuration for the TrybuildChangedPolicy.
#[derive(Debug, Clone)]
pub struct TrybuildChangedConfig {
    /// Directory containing trybuild tests.
    pub trybuild_dir: String,
    /// Last known hash of trybuild files (for detecting changes).
    pub last_hash: Option<String>,
}

impl Default for TrybuildChangedConfig {
    fn default() -> Self {
        Self {
            trybuild_dir: "tests/trybuild".to_string(),
            last_hash: None,
        }
    }
}

/// Computes a simple hash of all files in the trybuild directory.
fn compute_dir_hash(path: &Path) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();

    if !path.exists() {
        return Ok(String::new());
    }

    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

    // Sort for deterministic ordering
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let metadata = entry.metadata().map_err(|e| format!("Metadata error: {}", e))?;

        if metadata.is_file() {
            let file_path = entry.path();
            let file_content = fs::read(&file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            hasher.update(&file_content);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Checks if trybuild files have changed since the last known hash.
pub fn check_changed(config: &TrybuildChangedConfig) -> bool {
    let trybuild_path = Path::new(&config.trybuild_dir);

    match compute_dir_hash(trybuild_path) {
        Ok(current_hash) => {
            if let Some(last_hash) = &config.last_hash {
                current_hash != *last_hash && !current_hash.is_empty()
            } else {
                // No previous hash stored, so we consider it "changed" (first run)
                !current_hash.is_empty()
            }
        }
        Err(_) => false, // If we can't compute hash, no change detected
    }
}

/// Executes the policy: runs `cargo test --test changed` if trybuild files changed.
pub fn execute(config: &TrybuildChangedConfig, apply: bool) -> Result<(), String> {
    if !check_changed(config) {
        return Ok(()); // No action needed
    }

    if apply {
        // Actually run cargo test
        crate::autonomic::subprocess::run_with_timeout("cargo", &["test", "--test", "changed"], false)
            .map(|_| ())
    } else {
        // Just report what would happen
        eprintln!("[TrybuildChangedPolicy] Would run: cargo test --test changed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trybuild_changed_config_default() {
        let config = TrybuildChangedConfig::default();
        assert_eq!(config.trybuild_dir, "tests/trybuild");
        assert!(config.last_hash.is_none());
    }

    #[test]
    fn test_compute_dir_hash_missing() {
        let hash = compute_dir_hash(Path::new("/nonexistent/path"));
        assert!(hash.is_ok());
        assert_eq!(hash.unwrap(), "");
    }

    #[test]
    fn test_check_changed_missing_dir() {
        let config = TrybuildChangedConfig {
            trybuild_dir: "/nonexistent/trybuild".to_string(),
            last_hash: None,
        };
        assert!(!check_changed(&config));
    }

    #[test]
    fn test_check_changed_with_previous_hash() {
        let config = TrybuildChangedConfig {
            trybuild_dir: "/nonexistent/trybuild".to_string(),
            last_hash: Some("previous_hash".to_string()),
        };
        assert!(!check_changed(&config));
    }

    #[test]
    fn test_execute_dry_run() {
        let config = TrybuildChangedConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
