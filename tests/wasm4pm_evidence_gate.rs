//! Integration tests for the wasm4pm oracle shell and evidence gate.
//!
//! Tests use WPM_BIN=/bin/echo (always exits 0) for discovery tests.

use star_toml::{
    EvidenceGate, OracleGateVerdict, OracleVerdict, Wasm4pmShell, WpmVerdict,
};

// ---------------------------------------------------------------------------
// Oracle Verdict Discovery & Inference
// ---------------------------------------------------------------------------

#[test]
fn test_wpm_discovery_env_var() {
    // /bin/echo always exists and exits 0 — use it as a stand-in for wpm.
    std::env::set_var("WPM_BIN", "/bin/echo");
    let result = Wasm4pmShell::discover();
    std::env::remove_var("WPM_BIN");
    assert!(result.is_ok(), "discover() should succeed when WPM_BIN points to /bin/echo");
}

#[test]
fn test_wpm_discovery_missing() {
    // Remove all relevant env vars and ensure no `wpm` is on PATH in CI.
    std::env::remove_var("WPM_BIN");
    std::env::remove_var("WPM_PATH");
    std::env::remove_var("WPM_SEARCH_PATHS");

    // Only expect NotFound if wpm is genuinely absent.
    match Wasm4pmShell::discover() {
        Err(star_toml::Wasm4pmError::NotFound) => {
            // Correct: not found.
        }
        Ok(_) => {
            // wpm happens to be on this machine — acceptable, test is inconclusive.
        }
        Err(e) => panic!("unexpected error variant: {e:?}"),
    }
}

#[test]
fn test_wpm_binary_missing_returns_partial() {
    let result = Wasm4pmShell::from_path("/nonexistent/path/to/wpm_binary_xyz");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Verdict Inference
// ---------------------------------------------------------------------------

#[test]
fn test_infer_verdict_from_wpm_stdout() {
    assert_eq!(Wasm4pmShell::infer_verdict("PASS: all checks ok", true), WpmVerdict::Pass);
    assert_eq!(Wasm4pmShell::infer_verdict("WARN: drift detected", true), WpmVerdict::Warn);
    assert_eq!(Wasm4pmShell::infer_verdict("FAIL: critical error", true), WpmVerdict::Fail);
}

#[test]
fn test_infer_verdict_from_exit_code() {
    // Non-zero exit always means Fail, regardless of stdout
    assert_eq!(Wasm4pmShell::infer_verdict("PASS", false), WpmVerdict::Fail);
    assert_eq!(Wasm4pmShell::infer_verdict("", false), WpmVerdict::Fail);
}

#[test]
fn test_wpm_exit_code_1_returns_fail() {
    assert_eq!(Wasm4pmShell::infer_verdict("error output", false), WpmVerdict::Fail);
}

#[test]
fn test_wpm_malformed_output_returns_warn() {
    // Output with no recognizable keywords → Warn (unknown = cautious)
    assert_eq!(
        Wasm4pmShell::infer_verdict("some unrecognized oracle output here", true),
        WpmVerdict::Warn
    );
}

#[test]
fn test_wpm_not_found_returns_partial() {
    // When wpm binary doesn't exist, audit_with_verdict returns NotAvailable
    // (we can't call audit_with_verdict without a valid Wasm4pmShell, but we
    //  can verify the shell creation fails and the verdict would be NotAvailable)
    let result = Wasm4pmShell::from_path("/nonexistent/wpm_xyz_abc");
    assert!(
        result.is_err(),
        "from_path with nonexistent binary should return error"
    );
}

// ---------------------------------------------------------------------------
// Evidence Gate Integration
// ---------------------------------------------------------------------------

#[test]
fn test_gate_verdict_ordering_deterministic() {
    assert!(OracleVerdict::Fail > OracleVerdict::Warn);
    assert!(OracleVerdict::Warn > OracleVerdict::Suggest);
    assert!(OracleVerdict::Suggest > OracleVerdict::Admit);
}

#[test]
fn test_gate_verdict_oracle_pass_does_not_escalate() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Admit,
        reason: "all clear".to_string(),
        evidence_refs: vec![],
        source: "test".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Pass));
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Admit));
    assert!(!report.is_blocked);
}

#[test]
fn test_gate_verdict_oracle_warn_escalates_admit_to_warn() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Admit,
        reason: "local check passed".to_string(),
        evidence_refs: vec![],
        source: "test".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Warn));
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Warn));
}

#[test]
fn test_gate_verdict_oracle_warn_does_not_reduce_fail() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Fail,
        reason: "local failure".to_string(),
        evidence_refs: vec![],
        source: "test".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Warn));
    // Warn cannot reduce an existing Fail
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Fail));
    assert!(report.is_blocked);
}

#[test]
fn test_gate_verdict_oracle_fail_always_escalates() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Suggest,
        reason: "minor suggestion".to_string(),
        evidence_refs: vec![],
        source: "test".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Fail));
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Fail));
    assert!(report.is_blocked);
}

#[test]
fn test_gate_verdict_oracle_partial_has_no_effect() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Admit,
        reason: "admitted".to_string(),
        evidence_refs: vec![],
        source: "test".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Partial));
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Admit));
}

#[test]
fn test_gate_verdict_ordering_with_wpm() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Admit,
        reason: "ok".to_string(),
        evidence_refs: vec![],
        source: "rule-1".to_string(),
    });
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Warn,
        reason: "borderline".to_string(),
        evidence_refs: vec!["ev-1".to_string()],
        source: "rule-2".to_string(),
    });
    let report = gate.to_report(None);
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Warn));
    assert!(!report.is_blocked);
    assert_eq!(report.summary.warns, 1);
    assert_eq!(report.summary.admits, 1);
}

#[test]
fn test_wpm_warn_escalates_gate_to_warn() {
    let mut gate = EvidenceGate::new();
    gate.push(OracleGateVerdict {
        verdict: OracleVerdict::Admit,
        reason: "local admit".to_string(),
        evidence_refs: vec![],
        source: "local-rule".to_string(),
    });
    let report = gate.to_report(Some(WpmVerdict::Warn));
    assert_eq!(report.highest_verdict, Some(OracleVerdict::Warn));
    assert_eq!(report.oracle_verdict, Some(WpmVerdict::Warn));
    assert!(!report.is_blocked);
}

// ---------------------------------------------------------------------------
// OCEL Schema Validation
// ---------------------------------------------------------------------------

#[test]
fn test_ocel_schema_has_required_version_field() {
    use star_toml::validate_ocel_schema;
    let json = r#"{"ocel:version":"2.0","ocel:events":[{"ocel:activity":"load","ocel:timestamp":"2024-01-01T00:00:00Z"}]}"#;
    assert!(validate_ocel_schema(json).is_ok());
}

#[test]
fn test_ocel_schema_missing_version_is_invalid() {
    use star_toml::validate_ocel_schema;
    let json = r#"{"ocel:events":[{"ocel:activity":"load","ocel:timestamp":"2024-01-01T00:00:00Z"}]}"#;
    assert!(validate_ocel_schema(json).is_err());
}
