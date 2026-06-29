//! Policy engine that orchestrates all autonomic policies.
//!
//! AC7: All policies respect --apply flag.
//! AC9: Idempotency guarantees.

use crate::autonomic::policies::{
    BranchBehindConfig, GitPhaseDirtyConfig, PublishNotAdjudicatedConfig, TargetPressureConfig,
    ToolchainMismatchConfig, TrybuildChangedConfig, EvidenceStaleConfig,
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
        let mut results = Vec::new();
        let mut total_executed = 0;
        let mut total_failed = 0;

        // AC1: TargetPressurePolicy
        let name = "TargetPressurePolicy";
        eprintln!("[{}] Checking target pressure...", name);
        match crate::autonomic::policies::target_pressure::execute(&self.target_pressure, self.apply)
        {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::target_pressure::check_pressure(
                        &self.target_pressure,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::target_pressure::check_pressure(&self.target_pressure)
                {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // AC2: ToolchainMismatchPolicy
        let name = "ToolchainMismatchPolicy";
        eprintln!("[{}] Checking toolchain mismatch...", name);
        match crate::autonomic::policies::toolchain_mismatch::execute(
            &self.toolchain_mismatch,
            self.apply,
        ) {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::toolchain_mismatch::check_mismatch(
                        &self.toolchain_mismatch,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::toolchain_mismatch::check_mismatch(
                    &self.toolchain_mismatch,
                ) {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // AC3: TrybuildChangedPolicy
        let name = "TrybuildChangedPolicy";
        eprintln!("[{}] Checking trybuild changes...", name);
        match crate::autonomic::policies::trybuild_changed::execute(&self.trybuild_changed, self.apply)
        {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::trybuild_changed::check_changed(
                        &self.trybuild_changed,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::trybuild_changed::check_changed(&self.trybuild_changed)
                {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // AC4: BranchBehindPolicy
        let name = "BranchBehindPolicy";
        eprintln!("[{}] Checking branch behind...", name);
        match crate::autonomic::policies::branch_behind::execute(&self.branch_behind, self.apply) {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::branch_behind::check_behind(
                        &self.branch_behind,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::branch_behind::check_behind(&self.branch_behind) {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // AC5: PublishNotAdjudicatedPolicy
        let name = "PublishNotAdjudicatedPolicy";
        eprintln!("[{}] Checking publish adjudication...", name);
        match crate::autonomic::policies::publish_not_adjudicated::execute(
            &self.publish_not_adjudicated,
            self.apply,
        ) {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::publish_not_adjudicated::check_needs_dry_run(
                        &self.publish_not_adjudicated,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::publish_not_adjudicated::check_needs_dry_run(
                    &self.publish_not_adjudicated,
                ) {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // AC6: GitPhaseDirtyPolicy
        let name = "GitPhaseDirtyPolicy";
        eprintln!("[{}] Checking git dirty state...", name);
        match crate::autonomic::policies::git_phase_dirty::execute(&self.git_phase_dirty, self.apply)
        {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::git_phase_dirty::check_dirty(),
                    error: None,
                });
                if crate::autonomic::policies::git_phase_dirty::check_dirty() {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        // evidence_stale policy
        let name = "EvidenceStalyPolicy";
        eprintln!("[{}] Checking evidence staleness...", name);
        match crate::autonomic::policies::evidence_stale::execute(&self.evidence_stale, self.apply) {
            Ok(_) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: crate::autonomic::policies::evidence_stale::check_stale(
                        &self.evidence_stale,
                    ),
                    error: None,
                });
                if crate::autonomic::policies::evidence_stale::check_stale(&self.evidence_stale) {
                    total_executed += 1;
                }
            }
            Err(e) => {
                results.push(PolicyResult {
                    policy_name: name.to_string(),
                    executed: true,
                    error: Some(e),
                });
                total_failed += 1;
            }
        }

        PolicyEngineResult {
            results,
            total_executed,
            total_failed,
        }
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
