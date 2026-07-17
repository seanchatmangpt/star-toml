//! AC3 & AC6: Evidence gate integration and oracle verdict handling.
//!
//! AC3: Gate is invoked after evidence emission to apply oracle verdict
//! AC6: Deterministic ordering (ADMIT → SUGGEST → WARN → FAIL)
//!
//! The gate controls admission flow based on oracle decisions.
//! Verdicts are stratified by severity in deterministic order.

use crate::validation::Severity;

/// AC6: Oracle verdict severity in deterministic order.
///
/// Ordering: ADMIT (lowest) → SUGGEST → WARN → FAIL (highest)
/// This ensures consistent, predictable verdict processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OracleVerdict {
    /// AC6: ADMIT — configuration is valid, no concerns.
    Admit,
    /// AC6: SUGGEST — recommended improvements, but not blocking.
    Suggest,
    /// AC6: WARN — potential issues, advisory level.
    Warn,
    /// AC6: FAIL — critical issue, blocks admission.
    Fail,
}

impl std::fmt::Display for OracleVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleVerdict::Admit => write!(f, "ADMIT"),
            OracleVerdict::Suggest => write!(f, "SUGGEST"),
            OracleVerdict::Warn => write!(f, "WARN"),
            OracleVerdict::Fail => write!(f, "FAIL"),
        }
    }
}

impl OracleVerdict {
    /// Convert verdict to corresponding Severity level.
    pub fn to_severity(self) -> Severity {
        match self {
            OracleVerdict::Admit => Severity::Advisory,
            OracleVerdict::Suggest => Severity::Advisory,
            OracleVerdict::Warn => Severity::Warning,
            OracleVerdict::Fail => Severity::Fatal,
        }
    }

    /// Check if verdict is blocking (FAIL or higher).
    pub fn is_blocking(self) -> bool {
        matches!(self, OracleVerdict::Fail)
    }

    /// Get the numeric value for sorting (lower = less severe).
    pub fn severity_rank(self) -> u8 {
        match self {
            OracleVerdict::Admit => 0,
            OracleVerdict::Suggest => 1,
            OracleVerdict::Warn => 2,
            OracleVerdict::Fail => 3,
        }
    }
}

/// AC3 & AC6: Evidence gate verdict result.
/// Represents oracle decision with supporting evidence.
#[derive(Debug, Clone)]
pub struct OracleGateVerdict {
    /// The verdict (ADMIT/SUGGEST/WARN/FAIL).
    pub verdict: OracleVerdict,
    /// Detailed explanation of the verdict.
    pub reason: String,
    /// Supporting evidence identifiers.
    pub evidence_refs: Vec<String>,
    /// Source of the verdict (e.g., "wpm_audit").
    pub source: String,
}

impl OracleGateVerdict {
    /// Create a new oracle gate verdict.
    pub fn new(
        verdict: OracleVerdict,
        reason: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            verdict,
            reason: reason.into(),
            evidence_refs: Vec::new(),
            source: source.into(),
        }
    }

    /// Add evidence reference to this verdict.
    pub fn with_evidence(mut self, evidence_id: String) -> Self {
        self.evidence_refs.push(evidence_id);
        self
    }

    /// Check if this verdict blocks admission (FAIL).
    pub fn is_blocking(&self) -> bool {
        self.verdict.is_blocking()
    }
}

/// AC3: Evidence gate that accumulates verdicts from oracle.
/// Processes verdicts in deterministic order (AC6).
#[derive(Debug, Clone, Default)]
pub struct EvidenceGate {
    /// Accumulated verdicts from oracle invocations.
    verdicts: Vec<OracleGateVerdict>,
}

impl EvidenceGate {
    /// Create a new evidence gate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// AC3: Add a verdict from oracle to the gate.
    /// Verdicts are stored in order of addition.
    pub fn add_verdict(&mut self, verdict: OracleGateVerdict) {
        self.verdicts.push(verdict);
    }

    /// AC6: Get the highest-priority verdict (most severe).
    /// Returns the verdict with highest severity_rank.
    #[must_use]
    pub fn highest_verdict(&self) -> Option<&OracleGateVerdict> {
        self.verdicts
            .iter()
            .max_by_key(|v| v.verdict.severity_rank())
    }

    /// AC6: Determine final gate state based on verdict priority.
    /// Returns true if any verdict is blocking (FAIL).
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.verdicts.iter().any(|v| v.verdict.is_blocking())
    }

    /// Get all verdicts in order.
    pub fn verdicts(&self) -> &[OracleGateVerdict] {
        &self.verdicts
    }

    /// AC6: Get verdicts sorted by severity (highest first).
    #[must_use]
    pub fn verdicts_by_severity(&self) -> Vec<&OracleGateVerdict> {
        let mut sorted: Vec<&OracleGateVerdict> = self.verdicts.iter().collect();
        sorted.sort_by_key(|v| std::cmp::Reverse(v.verdict.severity_rank()));
        sorted
    }

    /// Count verdicts by type.
    #[must_use]
    pub fn verdict_summary(&self) -> VerdictSummary {
        let mut admits = 0;
        let mut suggests = 0;
        let mut warns = 0;
        let mut fails = 0;

        for v in &self.verdicts {
            match v.verdict {
                OracleVerdict::Admit => admits += 1,
                OracleVerdict::Suggest => suggests += 1,
                OracleVerdict::Warn => warns += 1,
                OracleVerdict::Fail => fails += 1,
            }
        }

        VerdictSummary { admits, suggests, warns, fails }
    }
}

/// Summary of verdicts by count.
#[derive(Debug, Clone)]
pub struct VerdictSummary {
    /// Count of ADMIT verdicts.
    pub admits: usize,
    /// Count of SUGGEST verdicts.
    pub suggests: usize,
    /// Count of WARN verdicts.
    pub warns: usize,
    /// Count of FAIL verdicts.
    pub fails: usize,
}

impl VerdictSummary {
    /// Total number of verdicts.
    pub fn total(&self) -> usize {
        self.admits + self.suggests + self.warns + self.fails
    }

    /// Whether any blocking verdicts exist.
    pub fn is_blocked(&self) -> bool {
        self.fails > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_verdict_ordering() {
        // AC6: Verify deterministic ordering
        assert!(OracleVerdict::Admit < OracleVerdict::Suggest);
        assert!(OracleVerdict::Suggest < OracleVerdict::Warn);
        assert!(OracleVerdict::Warn < OracleVerdict::Fail);
    }

    #[test]
    fn test_oracle_verdict_blocking() {
        assert!(!OracleVerdict::Admit.is_blocking());
        assert!(!OracleVerdict::Suggest.is_blocking());
        assert!(!OracleVerdict::Warn.is_blocking());
        assert!(OracleVerdict::Fail.is_blocking());
    }

    #[test]
    fn test_evidence_gate_highest_verdict() {
        let mut gate = EvidenceGate::new();
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Suggest,
            "test suggest",
            "test",
        ));
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Warn,
            "test warn",
            "test",
        ));
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Fail,
            "test fail",
            "test",
        ));

        let highest = gate.highest_verdict().unwrap();
        assert_eq!(highest.verdict, OracleVerdict::Fail);
    }

    #[test]
    fn test_evidence_gate_blocking() {
        let mut gate = EvidenceGate::new();
        assert!(!gate.is_blocked());

        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Warn,
            "test",
            "test",
        ));
        assert!(!gate.is_blocked());

        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Fail,
            "blocking",
            "test",
        ));
        assert!(gate.is_blocked());
    }

    #[test]
    fn test_verdict_summary() {
        let mut gate = EvidenceGate::new();
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Admit,
            "test",
            "test",
        ));
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Suggest,
            "test",
            "test",
        ));
        gate.add_verdict(OracleGateVerdict::new(
            OracleVerdict::Warn,
            "test",
            "test",
        ));

        let summary = gate.verdict_summary();
        assert_eq!(summary.admits, 1);
        assert_eq!(summary.suggests, 1);
        assert_eq!(summary.warns, 1);
        assert_eq!(summary.fails, 0);
        assert!(!summary.is_blocked());
    }
}
