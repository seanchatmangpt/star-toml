//! AC2: ToolchainMismatchPolicy
//! Executes `rustup update` when Rust toolchain version mismatches.

use std::{fs, process::Command};

/// Configuration for the ToolchainMismatchPolicy.
#[derive(Debug, Clone)]
pub struct ToolchainMismatchConfig {
    /// Path to rust-toolchain.toml file.
    pub toolchain_file: String,
}

impl Default for ToolchainMismatchConfig {
    fn default() -> Self {
        Self { toolchain_file: "rust-toolchain.toml".to_string() }
    }
}

/// Reads the toolchain version from rust-toolchain.toml.
fn read_toolchain_version(path: &str) -> Result<String, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read toolchain file: {}", e))?;

    // Simple parsing: look for "channel = " line
    for line in content.lines() {
        if let Some(pos) = line.find("channel") {
            if let Some(eq_pos) = line[pos..].find('=') {
                let version_str = &line[pos + eq_pos + 1..];
                return Ok(version_str.trim().trim_matches('"').to_string());
            }
        }
    }

    Err("Could not parse toolchain version from file".to_string())
}

/// Gets the current Rust version installed.
pub fn get_current_version() -> Result<String, String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to get rustc version: {}", e))?;

    if !output.status.success() {
        return Err("rustc --version failed".to_string());
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    Ok(version_str.to_string())
}

/// Checks if the toolchain versions match.
pub fn check_mismatch(config: &ToolchainMismatchConfig) -> bool {
    match (read_toolchain_version(&config.toolchain_file), get_current_version()) {
        (Ok(expected), Ok(current)) => {
            // Simple check: if expected string is not in current output, it's a mismatch
            !current.contains(&expected)
        }
        _ => false, // If we can't determine, assume no mismatch
    }
}

/// Executes the policy: runs `rustup update` if versions mismatch.
pub fn execute(config: &ToolchainMismatchConfig, apply: bool) -> Result<(), String> {
    if !check_mismatch(config) {
        return Ok(()); // No action needed
    }

    if apply {
        // Actually run rustup update
        crate::autonomic::subprocess::run_with_timeout("rustup", &["update"], false).map(|_| ())
    } else {
        // Just report what would happen
        eprintln!("[ToolchainMismatchPolicy] Would run: rustup update");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_mismatch_config_default() {
        let config = ToolchainMismatchConfig::default();
        assert_eq!(config.toolchain_file, "rust-toolchain.toml");
    }

    #[test]
    fn test_read_toolchain_version_missing() {
        let result = read_toolchain_version("/nonexistent/file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_current_version() {
        let result = get_current_version();
        // rustc --version should work in any Rust environment
        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(version.contains("rustc"));
    }

    #[test]
    fn test_check_mismatch_missing_file() {
        let config =
            ToolchainMismatchConfig { toolchain_file: "/nonexistent/toolchain.toml".to_string() };
        assert!(!check_mismatch(&config));
    }

    #[test]
    fn test_execute_dry_run() {
        let config = ToolchainMismatchConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
