//! AC5: PublishNotAdjudicatedPolicy
//! Executes `cargo publish --dry-run` before actually publishing to crates.io.

use std::process::Command;

/// Configuration for the PublishNotAdjudicatedPolicy.
#[derive(Debug, Clone)]
pub struct PublishNotAdjudicatedConfig {
    /// Whether to check if the package has been published before.
    pub check_published: bool,
}

impl Default for PublishNotAdjudicatedConfig {
    fn default() -> Self {
        Self {
            check_published: true,
        }
    }
}

/// Checks if this package is unpublished (never published before).
fn check_unpublished() -> bool {
    // Run: cargo search <package-name> --limit 1
    // If it doesn't exist in registry, it's unpublished
    let output = Command::new("cargo")
        .arg("search")
        .arg("star-toml")
        .arg("--limit")
        .arg("1")
        .output();

    match output {
        Ok(out) => !String::from_utf8_lossy(&out.stdout).contains("star-toml"),
        Err(_) => true, // If we can't check, assume unpublished (safe)
    }
}

/// Checks if we should perform a dry-run before publishing.
pub fn check_needs_dry_run(config: &PublishNotAdjudicatedConfig) -> bool {
    if !config.check_published {
        return false;
    }

    // Only require dry-run for unpublished packages
    check_unpublished()
}

/// Executes the policy: runs `cargo publish --dry-run` before actual publish.
pub fn execute(config: &PublishNotAdjudicatedConfig, apply: bool) -> Result<(), String> {
    if !check_needs_dry_run(config) {
        return Ok(()); // No action needed
    }

    if apply {
        // First, run cargo publish --dry-run
        eprintln!("[PublishNotAdjudicatedPolicy] Running dry-run first...");
        crate::autonomic::subprocess::run_with_timeout(
            "cargo",
            &["publish", "--dry-run"],
            false,
        )?;

        // If dry-run succeeds, run actual publish
        eprintln!("[PublishNotAdjudicatedPolicy] Dry-run passed, proceeding with publish...");
        crate::autonomic::subprocess::run_with_timeout("cargo", &["publish"], false)
            .map(|_| ())
    } else {
        // Just report what would happen
        eprintln!("[PublishNotAdjudicatedPolicy] Would run: cargo publish --dry-run && cargo publish");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_not_adjudicated_config_default() {
        let config = PublishNotAdjudicatedConfig::default();
        assert!(config.check_published);
    }

    #[test]
    fn test_check_unpublished() {
        // This test just ensures the function doesn't panic
        let _result = check_unpublished();
    }

    #[test]
    fn test_check_needs_dry_run_disabled() {
        let config = PublishNotAdjudicatedConfig {
            check_published: false,
        };
        assert!(!check_needs_dry_run(&config));
    }

    #[test]
    fn test_execute_dry_run() {
        let config = PublishNotAdjudicatedConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
