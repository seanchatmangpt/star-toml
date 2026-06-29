//! AC6: GitPhaseDirectyPolicy (GitPhaseDirtyPolicy)
//! Executes `git add + commit` when the working tree is dirty.

use std::process::Command;

/// Configuration for the GitPhaseDirtyPolicy.
#[derive(Debug, Clone)]
pub struct GitPhaseDirtyConfig {
    /// Commit message to use when auto-committing.
    pub commit_message: String,
}

impl Default for GitPhaseDirtyConfig {
    fn default() -> Self {
        Self { commit_message: "chore: auto-commit dirty tree".to_string() }
    }
}

/// Checks if the git working tree is dirty (has uncommitted changes).
pub fn check_dirty() -> bool {
    // Run: git status --porcelain
    // If output is non-empty, there are uncommitted changes
    let output = Command::new("git").arg("status").arg("--porcelain").output();

    match output {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout);
            !status.trim().is_empty()
        }
        Err(_) => false, // If we can't run git, assume not dirty
    }
}

/// Executes the policy: runs `git add + git commit` if working tree is dirty.
pub fn execute(config: &GitPhaseDirtyConfig, apply: bool) -> Result<(), String> {
    if !check_dirty() {
        return Ok(()); // No action needed
    }

    if apply {
        // First, add all changes
        crate::autonomic::subprocess::run_with_timeout("git", &["add", "-A"], false)?;

        // Then, commit with the provided message
        crate::autonomic::subprocess::run_with_timeout(
            "git",
            &["commit", "-m", &config.commit_message],
            false,
        )
        .map(|_| ())
    } else {
        // Just report what would happen
        eprintln!(
            "[GitPhaseDirtyPolicy] Would run: git add -A && git commit -m '{}'",
            config.commit_message
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_phase_dirty_config_default() {
        let config = GitPhaseDirtyConfig::default();
        assert_eq!(config.commit_message, "chore: auto-commit dirty tree");
    }

    #[test]
    fn test_check_dirty_no_git() {
        // This test just ensures the function doesn't panic
        let _result = check_dirty();
    }

    #[test]
    fn test_execute_dry_run() {
        let config = GitPhaseDirtyConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
