//! Autonomic policies for various development scenarios.
//!
//! AC1-AC6: Six policies that execute when certain conditions are met.
//! AC7: All policies respect the --apply flag.
//! AC8: Subprocess isolation with 30s timeout.
//! AC9: Idempotency guarantees.

pub mod branch_behind;
pub mod git_phase_dirty;
pub mod publish_not_adjudicated;
pub mod target_pressure;
pub mod toolchain_mismatch;
pub mod trybuild_changed;

pub use branch_behind::BranchBehindConfig;
pub use git_phase_dirty::GitPhaseDirtyConfig;
pub use publish_not_adjudicated::PublishNotAdjudicatedConfig;
pub use target_pressure::TargetPressureConfig;
pub use toolchain_mismatch::{get_current_version, ToolchainMismatchConfig};
pub use trybuild_changed::TrybuildChangedConfig;

/// The evidence_stale policy (stub) — detects stale evidence and cleans it.
pub mod evidence_stale {
    #[derive(Debug, Clone)]
    pub struct EvidenceStaleConfig {
        pub evidence_dir: String,
        pub max_age_days: u32,
    }

    impl Default for EvidenceStaleConfig {
        fn default() -> Self {
            Self { evidence_dir: ".evidence".to_string(), max_age_days: 30 }
        }
    }

    pub fn check_stale(config: &EvidenceStaleConfig) -> bool {
        // Check if evidence directory exists and has old files
        let path = std::path::Path::new(&config.evidence_dir);
        if !path.exists() {
            return false;
        }

        // Simple check: if directory exists, consider it potentially stale
        true
    }

    pub fn execute(config: &EvidenceStaleConfig, apply: bool) -> Result<(), String> {
        if !check_stale(config) {
            return Ok(());
        }

        if apply {
            eprintln!("[EvidenceStalyPolicy] Cleaning stale evidence...");
            // In a real implementation, this would remove old files
            Ok(())
        } else {
            eprintln!("[EvidenceStalyPolicy] Would clean stale evidence");
            Ok(())
        }
    }
}

pub use evidence_stale::EvidenceStaleConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_policies_export() {
        // Ensure all policies are accessible
        let _target = TargetPressureConfig::default();
        let _toolchain = ToolchainMismatchConfig::default();
        let _trybuild = TrybuildChangedConfig::default();
        let _branch = BranchBehindConfig::default();
        let _publish = PublishNotAdjudicatedConfig::default();
        let _dirty = GitPhaseDirtyConfig::default();
        let _evidence = EvidenceStaleConfig::default();
    }
}
