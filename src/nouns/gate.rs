//! Evidence gate and oracle verdict types for star-toml.

use crate::integrations::WpmVerdict;

/// The verdict produced by the oracle gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OracleVerdict {
    /// Fully admitted; no issues.
    Admit = 0,
    /// Suggestion only; not blocking.
    Suggest = 1,
    /// Warning present; may be escalated.
    Warn = 2,
    /// Failure; admission blocked.
    Fail = 3,
}

/// A single verdict from the oracle gate, with supporting context.
#[derive(Debug, Clone)]
pub struct OracleGateVerdict {
    /// The verdict level.
    pub verdict: OracleVerdict,
    /// Human-readable reason.
    pub reason: String,
    /// IDs of evidence records referenced.
    pub evidence_refs: Vec<String>,
    /// Source identifier (e.g. rule name or oracle name).
    pub source: String,
}

/// Summary of all verdicts in an evidence gate evaluation.
#[derive(Debug, Clone, Default)]
pub struct VerdictSummary {
    /// Total number of verdicts.
    pub total: usize,
    /// Number of Admit verdicts.
    pub admits: usize,
    /// Number of Suggest verdicts.
    pub suggests: usize,
    /// Number of Warn verdicts.
    pub warns: usize,
    /// Number of Fail verdicts.
    pub fails: usize,
}

/// Accumulates oracle gate verdicts and produces a final report.
#[derive(Debug, Clone, Default)]
pub struct EvidenceGate {
    verdicts: Vec<OracleGateVerdict>,
}

/// Final report produced by an `EvidenceGate` evaluation.
#[derive(Debug, Clone)]
pub struct GateReport {
    /// The highest (worst) `OracleVerdict` across all gate verdicts.
    pub highest_verdict: Option<OracleVerdict>,
    /// The verdict from the external wpm oracle, if available.
    pub oracle_verdict: Option<WpmVerdict>,
    /// True if the highest verdict is `Fail`.
    pub is_blocked: bool,
    /// Counts by verdict level.
    pub summary: VerdictSummary,
}

impl EvidenceGate {
    /// Create a new, empty evidence gate.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a verdict to the gate.
    pub fn push(&mut self, verdict: OracleGateVerdict) {
        self.verdicts.push(verdict);
    }

    /// Return the accumulated verdicts.
    pub fn verdicts(&self) -> &[OracleGateVerdict] {
        &self.verdicts
    }

    /// Compute the highest (worst) verdict across all accumulated verdicts.
    pub fn highest_verdict(&self) -> Option<OracleVerdict> {
        self.verdicts.iter().map(|v| v.verdict).max()
    }

    /// Produce a `GateReport`, optionally incorporating a wpm oracle verdict.
    ///
    /// The `oracle_verdict` is stored verbatim. If it maps to `Warn` or `Fail`,
    /// it escalates the `highest_verdict` accordingly.
    pub fn to_report(&self, oracle_verdict: Option<WpmVerdict>) -> GateReport {
        let mut summary = VerdictSummary {
            total: self.verdicts.len(),
            ..Default::default()
        };

        for v in &self.verdicts {
            match v.verdict {
                OracleVerdict::Admit => summary.admits += 1,
                OracleVerdict::Suggest => summary.suggests += 1,
                OracleVerdict::Warn => summary.warns += 1,
                OracleVerdict::Fail => summary.fails += 1,
            }
        }

        let mut highest = self.highest_verdict();

        // Escalate from oracle verdict
        if let Some(wpm) = oracle_verdict {
            let escalated = match wpm {
                WpmVerdict::Fail => Some(OracleVerdict::Fail),
                WpmVerdict::Warn => Some(OracleVerdict::Warn),
                WpmVerdict::Pass => None,
                WpmVerdict::Partial | WpmVerdict::NotAvailable => None,
            };
            if let Some(esc) = escalated {
                highest = Some(match highest {
                    None => esc,
                    Some(h) => h.max(esc),
                });
            }
        }

        let is_blocked = highest == Some(OracleVerdict::Fail);

        GateReport {
            highest_verdict: highest,
            oracle_verdict,
            is_blocked,
            summary,
        }
    }
}
