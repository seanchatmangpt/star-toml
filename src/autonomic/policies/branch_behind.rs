//! AC4: BranchBehindPolicy
//! Safely executes `git merge origin/main` when the current branch is behind origin/main.

use std::process::Command;

/// Configuration for the BranchBehindPolicy.
#[derive(Debug, Clone)]
pub struct BranchBehindConfig {
    /// Remote name (default: "origin").
    pub remote: String,
    /// Target branch (default: "main").
    pub target_branch: String,
}

impl Default for BranchBehindConfig {
    fn default() -> Self {
        Self {
            remote: "origin".to_string(),
            target_branch: "main".to_string(),
        }
    }
}

/// Checks if the current branch is behind the remote branch.
pub fn check_behind(config: &BranchBehindConfig) -> bool {
    let remote_branch = format!("{}/{}", config.remote, config.target_branch);

    // Run: git fetch (to ensure we have latest remote info)
    let fetch_output = Command::new("git")
        .arg("fetch")
        .arg(&config.remote)
        .output();

    if fetch_output.is_err() {
        return false; // Can't fetch, assume not behind
    }

    // Run: git rev-list --count remote_branch..HEAD
    // If this returns > 0, we're behind
    let output = Command::new("git")
        .arg("rev-list")
        .arg("--count")
        .arg(format!("{}..HEAD", remote_branch))
        .output();

    match output {
        Ok(out) => {
            if let Ok(count_str) = String::from_utf8(out.stdout) {
                count_str.trim().parse::<u32>().unwrap_or(0) == 0
                    && check_commits_behind(&remote_branch)
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Checks if there are commits ahead on the remote that we don't have.
fn check_commits_behind(remote_branch: &str) -> bool {
    let output = Command::new("git")
        .arg("rev-list")
        .arg("--count")
        .arg(format!("HEAD..{}", remote_branch))
        .output();

    match output {
        Ok(out) => {
            if let Ok(count_str) = String::from_utf8(out.stdout) {
                count_str.trim().parse::<u32>().unwrap_or(0) > 0
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Safely executes the policy: runs `git merge origin/main` if behind.
pub fn execute(config: &BranchBehindConfig, apply: bool) -> Result<(), String> {
    if !check_behind(config) {
        return Ok(()); // No action needed
    }

    let merge_ref = format!("{}/{}", config.remote, config.target_branch);

    if apply {
        // Actually run git merge with --no-ff to create a merge commit
        crate::autonomic::subprocess::run_with_timeout(
            "git",
            &["merge", "--no-ff", "-m", &format!("merge {}", merge_ref), &merge_ref],
            false,
        )
        .map(|_| ())
    } else {
        // Just report what would happen
        eprintln!(
            "[BranchBehindPolicy] Would run: git merge --no-ff -m 'merge {}' {}",
            merge_ref, merge_ref
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_behind_config_default() {
        let config = BranchBehindConfig::default();
        assert_eq!(config.remote, "origin");
        assert_eq!(config.target_branch, "main");
    }

    #[test]
    fn test_check_behind_no_git() {
        // This test assumes we're in a git repo; if not, check_behind returns false
        let config = BranchBehindConfig::default();
        let result = check_behind(&config);
        // We don't assert anything specific since behavior depends on git state
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_execute_dry_run() {
        let config = BranchBehindConfig::default();
        let result = execute(&config, false);
        assert!(result.is_ok());
    }
}
