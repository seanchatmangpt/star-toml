//! AC5: Error handling and oracle counterexamples for diagnostics.
//!
//! Provides comprehensive error codes and diagnostics for oracle integration,
//! including counterexamples, repair hints, and error context.

use std::fmt;

/// AC5: Oracle diagnostic error code.
/// Machine-matchable codes for oracle verdicts and wasm4pm integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OracleDiagnosticCode {
    /// Oracle binary not found.
    OracleNotFound,
    /// Oracle command execution failed.
    OracleCommandFailed,
    /// Oracle verdict indicates failure.
    OracleVerdictFail,
    /// Oracle verdict indicates warning.
    OracleVerdictWarn,
    /// OCEL validation failed.
    OcelValidationFailed,
    /// Event missing required field.
    OcelEventMissingField,
    /// Object missing required field.
    OcelObjectMissingField,
    /// Orphaned event (references non-existent object).
    OcelOrphanedEvent,
    /// Duplicate event ID.
    OcelDuplicateEvent,
    /// Duplicate object ID.
    OcelDuplicateObject,
    /// Evidence serialization failed.
    EvidenceSerializationFailed,
    /// Gate verdict is blocking admission.
    GateBlocking,
    /// Unknown oracle error.
    Unknown,
}

impl OracleDiagnosticCode {
    /// Convert code to machine-matchable string.
    pub fn code(&self) -> &'static str {
        match self {
            OracleDiagnosticCode::OracleNotFound => "oracle_not_found",
            OracleDiagnosticCode::OracleCommandFailed => "oracle_command_failed",
            OracleDiagnosticCode::OracleVerdictFail => "oracle_verdict_fail",
            OracleDiagnosticCode::OracleVerdictWarn => "oracle_verdict_warn",
            OracleDiagnosticCode::OcelValidationFailed => "ocel_validation_failed",
            OracleDiagnosticCode::OcelEventMissingField => "ocel_event_missing_field",
            OracleDiagnosticCode::OcelObjectMissingField => "ocel_object_missing_field",
            OracleDiagnosticCode::OcelOrphanedEvent => "ocel_orphaned_event",
            OracleDiagnosticCode::OcelDuplicateEvent => "ocel_duplicate_event",
            OracleDiagnosticCode::OcelDuplicateObject => "ocel_duplicate_object",
            OracleDiagnosticCode::EvidenceSerializationFailed => "evidence_serialization_failed",
            OracleDiagnosticCode::GateBlocking => "gate_blocking",
            OracleDiagnosticCode::Unknown => "oracle_unknown_error",
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            OracleDiagnosticCode::OracleNotFound => "wpm oracle binary not found",
            OracleDiagnosticCode::OracleCommandFailed => "oracle command execution failed",
            OracleDiagnosticCode::OracleVerdictFail => "oracle verdict indicates failure",
            OracleDiagnosticCode::OracleVerdictWarn => "oracle verdict indicates warning",
            OracleDiagnosticCode::OcelValidationFailed => "OCEL log validation failed",
            OracleDiagnosticCode::OcelEventMissingField => "OCEL event missing required field",
            OracleDiagnosticCode::OcelObjectMissingField => "OCEL object missing required field",
            OracleDiagnosticCode::OcelOrphanedEvent => "OCEL event references non-existent object",
            OracleDiagnosticCode::OcelDuplicateEvent => "OCEL duplicate event ID",
            OracleDiagnosticCode::OcelDuplicateObject => "OCEL duplicate object ID",
            OracleDiagnosticCode::EvidenceSerializationFailed => "evidence serialization failed",
            OracleDiagnosticCode::GateBlocking => "oracle gate verdict is blocking",
            OracleDiagnosticCode::Unknown => "unknown oracle error",
        }
    }
}

/// AC5: Oracle diagnostic with counterexamples and repair hints.
#[derive(Debug, Clone)]
pub struct OracleDiagnostic {
    /// Diagnostic error code (machine-matchable).
    pub code: OracleDiagnosticCode,
    /// Human-readable message.
    pub message: String,
    /// Path to the problematic config element.
    pub location: Option<String>,
    /// Counterexample showing the problem.
    pub counterexample: Option<String>,
    /// Suggested repair/fix.
    pub repair_hint: Option<String>,
}

impl OracleDiagnostic {
    /// Create a new oracle diagnostic.
    pub fn new(code: OracleDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            counterexample: None,
            repair_hint: None,
        }
    }

    /// Add location information.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// AC5: Add a counterexample showing the problem.
    pub fn with_counterexample(mut self, example: impl Into<String>) -> Self {
        self.counterexample = Some(example.into());
        self
    }

    /// AC5: Add a repair hint.
    pub fn with_repair_hint(mut self, hint: impl Into<String>) -> Self {
        self.repair_hint = Some(hint.into());
        self
    }
}

impl fmt::Display for OracleDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}",
            self.code.code(),
            self.code.description()
        )?;

        if let Some(location) = &self.location {
            write!(f, " (at {})", location)?;
        }

        write!(f, ": {}", self.message)?;

        if let Some(counterexample) = &self.counterexample {
            write!(f, "\n  counterexample: {}", counterexample)?;
        }

        if let Some(hint) = &self.repair_hint {
            write!(f, "\n  fix: {}", hint)?;
        }

        Ok(())
    }
}

impl std::error::Error for OracleDiagnostic {}

/// AC5: Oracle diagnostic report collecting multiple diagnostics.
#[derive(Debug, Clone, Default)]
pub struct OracleDiagnosticReport {
    diagnostics: Vec<OracleDiagnostic>,
}

impl OracleDiagnosticReport {
    /// Create a new diagnostic report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic to the report.
    pub fn add(&mut self, diagnostic: OracleDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Get all diagnostics.
    pub fn diagnostics(&self) -> &[OracleDiagnostic] {
        &self.diagnostics
    }

    /// Count diagnostics by severity level.
    #[must_use]
    pub fn summary(&self) -> DiagnosticSummary {
        let mut blocking = 0;
        let mut warnings = 0;
        let mut info = 0;

        for diag in &self.diagnostics {
            match diag.code {
                OracleDiagnosticCode::OracleVerdictFail
                | OracleDiagnosticCode::GateBlocking => {
                    blocking += 1;
                }
                OracleDiagnosticCode::OracleVerdictWarn
                | OracleDiagnosticCode::OcelValidationFailed
                | OracleDiagnosticCode::OcelOrphanedEvent
                | OracleDiagnosticCode::OcelDuplicateEvent
                | OracleDiagnosticCode::OcelDuplicateObject => {
                    warnings += 1;
                }
                _ => info += 1,
            }
        }

        DiagnosticSummary { blocking, warnings, info }
    }

    /// Check if any diagnostics are blocking.
    #[must_use]
    pub fn has_blocking(&self) -> bool {
        self.summary().blocking > 0
    }
}

/// Summary of diagnostics by severity.
#[derive(Debug, Clone)]
pub struct DiagnosticSummary {
    /// Count of blocking diagnostics.
    pub blocking: usize,
    /// Count of warning diagnostics.
    pub warnings: usize,
    /// Count of info diagnostics.
    pub info: usize,
}

impl DiagnosticSummary {
    /// Total number of diagnostics.
    pub fn total(&self) -> usize {
        self.blocking + self.warnings + self.info
    }
}

/// AC5: Common oracle diagnostic patterns and factories.
pub struct OracleDiagnostics;

impl OracleDiagnostics {
    /// Create diagnostic for oracle binary not found.
    pub fn oracle_not_found(searched_paths: &[&str]) -> OracleDiagnostic {
        let hint = format!(
            "ensure wpm is in PATH or set WPM_BIN environment variable (searched: {})",
            searched_paths.join(", ")
        );
        OracleDiagnostic::new(
            OracleDiagnosticCode::OracleNotFound,
            "wpm binary discovery failed",
        )
        .with_repair_hint(hint)
    }

    /// Create diagnostic for oracle command failure.
    pub fn oracle_command_failed(cmd: &str, stderr: &str) -> OracleDiagnostic {
        OracleDiagnostic::new(
            OracleDiagnosticCode::OracleCommandFailed,
            format!("wpm {} command failed", cmd),
        )
        .with_counterexample(stderr.to_string())
    }

    /// Create diagnostic for blocking verdict with counterexample.
    pub fn gate_blocking(verdict: &str, reason: &str) -> OracleDiagnostic {
        OracleDiagnostic::new(
            OracleDiagnosticCode::GateBlocking,
            format!("oracle verdict: {}", verdict),
        )
        .with_counterexample(reason.to_string())
        .with_repair_hint("review configuration against oracle output")
    }

    /// Create diagnostic for OCEL validation failure with counterexample.
    pub fn ocel_invalid(field: &str, detail: &str) -> OracleDiagnostic {
        OracleDiagnostic::new(
            OracleDiagnosticCode::OcelValidationFailed,
            format!("OCEL validation failed: {}", field),
        )
        .with_counterexample(detail.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_diagnostic_code_strings() {
        assert_eq!(
            OracleDiagnosticCode::OracleNotFound.code(),
            "oracle_not_found"
        );
        assert_eq!(
            OracleDiagnosticCode::GateBlocking.code(),
            "gate_blocking"
        );
    }

    #[test]
    fn test_oracle_diagnostic_with_hints() {
        let diag = OracleDiagnostic::new(OracleDiagnosticCode::OracleVerdictFail, "failed")
            .with_counterexample("example config")
            .with_repair_hint("fix this way");

        assert_eq!(diag.counterexample, Some("example config".to_string()));
        assert_eq!(diag.repair_hint, Some("fix this way".to_string()));
    }

    #[test]
    fn test_diagnostic_report_summary() {
        let mut report = OracleDiagnosticReport::new();
        report.add(OracleDiagnostic::new(
            OracleDiagnosticCode::GateBlocking,
            "test",
        ));
        report.add(OracleDiagnostic::new(
            OracleDiagnosticCode::OcelValidationFailed,
            "test",
        ));
        report.add(OracleDiagnostic::new(
            OracleDiagnosticCode::OracleNotFound,
            "test",
        ));

        let summary = report.summary();
        assert_eq!(summary.blocking, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.info, 1);
        assert!(report.has_blocking());
    }
}
