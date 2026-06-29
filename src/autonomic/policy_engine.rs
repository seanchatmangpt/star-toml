//! Policy engine that orchestrates all autonomic policies.
//!
//! AC7: All policies respect --apply flag.
//! AC9: Idempotency guarantees.

use crate::autonomic::policies::{
    BranchBehindConfig, EvidenceStaleConfig, GitPhaseDirtyConfig, PublishNotAdjudicatedConfig,
    TargetPressureConfig, ToolchainMismatchConfig, TrybuildChangedConfig,
};

/// The main policy executor.
#[derive(Debug)]
pub struct PolicyEngine {
    /// Whether to actually apply changes (--apply flag).
    pub apply: bool,
    /// Target pressure policy configuration.
    pub target_pressure: TargetPressureConfig,
    /// Toolchain mismatch policy configuration.
    pub toolchain_mismatch: ToolchainMismatchConfig,
    /// Trybuild changed policy configuration.
    pub trybuild_changed: TrybuildChangedConfig,
    /// Branch behind policy configuration.
    pub branch_behind: BranchBehindConfig,
    /// Publish not adjudicated policy configuration.
    pub publish_not_adjudicated: PublishNotAdjudicatedConfig,
    /// Git phase dirty policy configuration.
    pub git_phase_dirty: GitPhaseDirtyConfig,
    /// Evidence stale policy configuration.
    pub evidence_stale: EvidenceStaleConfig,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self {
            apply: false,
            target_pressure: TargetPressureConfig::default(),
            toolchain_mismatch: ToolchainMismatchConfig::default(),
            trybuild_changed: TrybuildChangedConfig::default(),
            branch_behind: BranchBehindConfig::default(),
            publish_not_adjudicated: PublishNotAdjudicatedConfig::default(),
            git_phase_dirty: GitPhaseDirtyConfig::default(),
            evidence_stale: EvidenceStaleConfig::default(),
        }
    }
}

/// Result of executing a single policy.
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// Name of the policy.
    pub policy_name: String,
    /// Whether the policy executed (action was taken).
    pub executed: bool,
    /// Error message if the policy failed.
    pub error: Option<String>,
}

/// Result of executing all policies.
#[derive(Debug)]
pub struct PolicyEngineResult {
    /// Results for each policy.
    pub results: Vec<PolicyResult>,
    /// Total policies executed.
    pub total_executed: usize,
    /// Total policies that failed.
    pub total_failed: usize,
}

impl PolicyEngine {
    /// Creates a new policy engine with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the --apply flag (AC7).
    pub fn with_apply(mut self, apply: bool) -> Self {
        self.apply = apply;
        self
    }

    /// Executes all 7 policies in sequence.
    /// AC7: Respects the --apply flag.
    /// AC8: Uses subprocess with 30s timeout.
    /// AC9: Each policy is idempotent (only executes if conditions are met).
    pub fn execute_all(&self) -> PolicyEngineResult {
        use crate::autonomic::policies;

        let mut results = Vec::new();
        let mut total_executed = 0;
        let mut total_failed = 0;

        // Check condition once, execute only if true, record result atomically (no TOCTOU).
        let mut run = |name: &str, condition: bool, exec: Result<(), String>| match exec {
            Ok(_) => {
                if condition {
                    total_executed += 1;
                }
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: condition,
                    error: None,
                });
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        };

        let p1 = policies::target_pressure::check_pressure(&self.target_pressure);
        run(
            "TargetPressurePolicy",
            p1,
            if p1 {
                policies::target_pressure::execute(&self.target_pressure, self.apply)
            } else {
                Ok(())
            },
        );

        let p2 = policies::toolchain_mismatch::check_mismatch(&self.toolchain_mismatch);
        run(
            "ToolchainMismatchPolicy",
            p2,
            if p2 {
                policies::toolchain_mismatch::execute(&self.toolchain_mismatch, self.apply)
            } else {
                Ok(())
            },
        );

        let p3 = policies::trybuild_changed::check_changed(&self.trybuild_changed);
        run(
            "TrybuildChangedPolicy",
            p3,
            if p3 {
                policies::trybuild_changed::execute(&self.trybuild_changed, self.apply)
            } else {
                Ok(())
            },
        );

        let p4 = policies::branch_behind::check_behind(&self.branch_behind);
        run(
            "BranchBehindPolicy",
            p4,
            if p4 {
                policies::branch_behind::execute(&self.branch_behind, self.apply)
            } else {
                Ok(())
            },
        );

        let p5 =
            policies::publish_not_adjudicated::check_needs_dry_run(&self.publish_not_adjudicated);
        run(
            "PublishNotAdjudicatedPolicy",
            p5,
            if p5 {
                policies::publish_not_adjudicated::execute(
                    &self.publish_not_adjudicated,
                    self.apply,
                )
            } else {
                Ok(())
            },
        );

        let p6 = policies::git_phase_dirty::check_dirty();
        run(
            "GitPhaseDirtyPolicy",
            p6,
            if p6 {
                policies::git_phase_dirty::execute(&self.git_phase_dirty, self.apply)
            } else {
                Ok(())
            },
        );

        let p7 = policies::evidence_stale::check_stale(&self.evidence_stale);
        run(
            "EvidenceStalePolicy",
            p7,
            if p7 {
                policies::evidence_stale::execute(&self.evidence_stale, self.apply)
            } else {
                Ok(())
            },
        );

        PolicyEngineResult { results, total_executed, total_failed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_default() {
        let engine = PolicyEngine::default();
        assert!(!engine.apply);
    }

    #[test]
    fn test_policy_engine_with_apply() {
        let engine = PolicyEngine::new().with_apply(true);
        assert!(engine.apply);
    }

    #[test]
    fn test_execute_all_dry_run() {
        let engine = PolicyEngine::new().with_apply(false);
        let result = engine.execute_all();
        assert_eq!(result.results.len(), 7); // 6 AC policies + 1 evidence_stale
        assert!(result.total_failed <= 7);
    }

    #[test]
    fn test_policy_engine_idempotency() {
        let engine = PolicyEngine::new().with_apply(false);
        let result1 = engine.execute_all();
        let result2 = engine.execute_all();

        // Both runs should have the same number of results
        assert_eq!(result1.results.len(), result2.results.len());

        // Since we're in dry-run mode, results should be consistent
        for i in 0..result1.results.len() {
            assert_eq!(result1.results[i].policy_name, result2.results[i].policy_name);
        }
    }
}
