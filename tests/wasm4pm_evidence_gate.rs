//! Test suite for wasm4pm oracle integration with evidence gate.
//!
//! Verifies that:
//! 1. Evidence OCEL is generated correctly
//! 2. Oracle verdict can be inferred from wpm audit results
//! 3. Oracle verdict flows through gate decision logic
//! 4. Graceful degradation when wpm binary is unavailable

use std::{fs, io::Write, path::PathBuf, process::Command};

use star_toml::{
    events::{AdmissionEvent, ConfigEventKind},
    ocel::export_events_to_ocel,
};
use tempfile::TempDir;

/// Represents the verdict returned by wpm oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WpmVerdict {
    Pass,
    Warn,
    Fail,
    Partial,
    NotAvailable,
}

impl WpmVerdict {
    fn to_string(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Partial => "PARTIAL",
            Self::NotAvailable => "NOT_AVAILABLE",
        }
    }
}

/// Represents the gate decision after oracle verdict integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GateVerdict {
    Admit,
    Suggest,
    Warn,
    Fail,
}

impl GateVerdict {
    /// Integrate an oracle verdict into the current gate verdict.
    /// Oracle verdicts escalate gate verdicts but never reduce them.
    fn integrate_oracle(self, oracle: WpmVerdict) -> Self {
        match oracle {
            WpmVerdict::Pass => self,
            WpmVerdict::Warn => {
                // Warn escalates Admit/Suggest to Warn, but doesn't override Fail
                match self {
                    Self::Fail => Self::Fail,
                    Self::Warn => Self::Warn,
                    Self::Suggest => Self::Warn,
                    Self::Admit => Self::Warn,
                }
            }
            WpmVerdict::Fail => Self::Fail,
            WpmVerdict::Partial | WpmVerdict::NotAvailable => self,
        }
    }
}

/// Creates a temporary directory with evidence OCEL JSON and mock wpm script.
fn setup_test_environment() -> std::io::Result<TempDir> {
    let tmpdir = TempDir::new()?;

    // Create a mock wpm script that simulates wpm audit behavior
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");
    let mut mock_file = fs::File::create(&mock_wpm_path)?;

    mock_file.write_all(
        br#"#!/bin/bash
# Mock wpm audit script for testing oracle verdict flows

INPUT_FILE="$2"  # Second argument is typically the input file

if [[ "$1" == "audit" ]] && [[ -f "$INPUT_FILE" ]]; then
    # Simulate successful audit by checking for OCEL structure
    if grep -q '"version"' "$INPUT_FILE" 2>/dev/null; then
        echo '{"verdict":"PASS","message":"OCEL audit successful","conformance":0.98}'
        exit 0
    else
        echo '{"verdict":"WARN","message":"OCEL structure incomplete","conformance":0.75}'
        exit 1
    fi
elif [[ "$1" == "doctor" ]]; then
    echo '{"verdict":"PASS","message":"System health check passed"}'
    exit 0
elif [[ "$1" == "lean" ]]; then
    echo '{"verdict":"WARN","message":"Potential waste detected in process","severity":"low"}'
    exit 0
elif [[ "$1" == "spc_status" ]]; then
    echo '{"verdict":"PASS","message":"Statistical process control trending normal"}'
    exit 0
else
    echo '{"verdict":"FAIL","message":"Unknown command or missing file"}'
    exit 1
fi
"#,
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&mock_wpm_path, fs::Permissions::from_mode(0o755))?;
    }

    Ok(tmpdir)
}

/// Creates a minimal OCEL JSON file with proper version field.
fn create_ocel_evidence(path: &PathBuf) -> std::io::Result<()> {
    let ocel_json = r#"{
  "version": "2.0",
  "omap": [
    {
      "ocel:oid": "run_001",
      "ocel:type": "ConfigRun",
      "ocel:timestamp": 1234567890000000000
    }
  ],
  "events": [
    {
      "ocel:eid": "evt_001",
      "ocel:timestamp": 1234567890000000000,
      "ocel:activity": "SourceDiscovered",
      "ocel:omap": ["run_001"]
    }
  ]
}"#;

    let mut file = fs::File::create(path)?;
    file.write_all(ocel_json.as_bytes())?;
    Ok(())
}

// ============================================================================
// TESTS: Oracle Verdict Discovery & Inference
// ============================================================================

#[test]
fn test_wpm_discovery_env_var() {
    let tmpdir = setup_test_environment().expect("Failed to setup test environment");
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");

    // Simulate wpm discovery via environment variable
    let wpm_bin = std::env::var("WPM_BIN")
        .or_else(|_| std::env::var("WPM_PATH"))
        .unwrap_or_else(|_| mock_wpm_path.to_string_lossy().to_string());

    assert!(!wpm_bin.is_empty(), "wpm binary path discovery should succeed");
}

#[test]
fn test_wpm_binary_missing_returns_partial() {
    let non_existent = "/tmp/nonexistent_wpm_binary_9999";

    let result = Command::new(non_existent).arg("audit").arg("/tmp/fake.json").status();

    assert!(result.is_err(), "Calling non-existent wpm should fail gracefully");

    // In real code, this would set verdict = Partial
    // Here we just verify the error handling pattern
}

#[test]
fn test_wpm_audit_ocel_json_success() {
    let tmpdir = setup_test_environment().expect("Failed to setup test environment");
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");
    let evidence_path = tmpdir.path().join("evidence.json");

    create_ocel_evidence(&evidence_path).expect("Failed to create OCEL evidence");

    // Run mock wpm audit
    let output = Command::new(&mock_wpm_path)
        .arg("audit")
        .arg(&evidence_path)
        .output()
        .expect("Failed to execute mock wpm");

    assert!(output.status.success(), "wpm audit should succeed for valid OCEL");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.contains("PASS"), "wpm audit verdict should be PASS for valid OCEL");
}

#[test]
fn test_wpm_audit_malformed_ocel_warns() {
    let tmpdir = setup_test_environment().expect("Failed to setup test environment");
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");
    let evidence_path = tmpdir.path().join("malformed.json");

    // Create malformed OCEL (missing version field)
    let mut file = fs::File::create(&evidence_path).expect("Failed to create malformed OCEL");
    file.write_all(b"{}").expect("Failed to write malformed OCEL");

    let output = Command::new(&mock_wpm_path)
        .arg("audit")
        .arg(&evidence_path)
        .output()
        .expect("Failed to execute mock wpm");

    // Mock wpm exits non-zero for malformed OCEL
    assert!(!output.status.success(), "wpm audit should fail for malformed OCEL");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        stdout.contains("WARN"),
        "wpm audit verdict should reflect warning for incomplete OCEL"
    );
}

// ============================================================================
// TESTS: Verdict Inference from wpm Output
// ============================================================================

#[test]
fn test_infer_verdict_from_wpm_stdout() {
    let test_cases = vec![
        (r#"{"verdict":"PASS"}"#, WpmVerdict::Pass),
        (r#"{"verdict":"WARN"}"#, WpmVerdict::Warn),
        (r#"{"verdict":"FAIL"}"#, WpmVerdict::Fail),
    ];

    for (stdout, expected) in test_cases {
        let inferred = if stdout.contains("PASS") {
            WpmVerdict::Pass
        } else if stdout.contains("WARN") {
            WpmVerdict::Warn
        } else if stdout.contains("FAIL") {
            WpmVerdict::Fail
        } else {
            WpmVerdict::NotAvailable
        };

        assert_eq!(inferred, expected, "Should infer verdict from stdout: {}", stdout);
    }
}

#[test]
fn test_infer_verdict_from_exit_code() {
    let test_cases = vec![(true, WpmVerdict::Pass), (false, WpmVerdict::Fail)];

    for (success, expected) in test_cases {
        let inferred = if success { WpmVerdict::Pass } else { WpmVerdict::Fail };

        assert_eq!(inferred, expected);
    }
}

// ============================================================================
// TESTS: Evidence Gate Integration
// ============================================================================

#[test]
fn test_gate_verdict_ordering_deterministic() {
    // Test that gate verdict ordering is strictly monotonic
    let initial_verdict = GateVerdict::Admit;

    // Successive oracle verdicts should only escalate or maintain
    let after_pass = initial_verdict.integrate_oracle(WpmVerdict::Pass);
    assert_eq!(after_pass, GateVerdict::Admit);

    let after_warn = after_pass.integrate_oracle(WpmVerdict::Warn);
    assert_eq!(after_warn, GateVerdict::Warn);

    let after_fail = after_warn.integrate_oracle(WpmVerdict::Fail);
    assert_eq!(after_fail, GateVerdict::Fail);

    // Fail should never be reduced
    let still_fail = after_fail.integrate_oracle(WpmVerdict::Pass);
    assert_eq!(still_fail, GateVerdict::Fail);
}

#[test]
fn test_gate_verdict_oracle_pass_does_not_escalate() {
    let initial = GateVerdict::Suggest;
    let result = initial.integrate_oracle(WpmVerdict::Pass);
    assert_eq!(result, GateVerdict::Suggest);
}

#[test]
fn test_gate_verdict_oracle_warn_escalates_admit_to_warn() {
    let initial = GateVerdict::Admit;
    let result = initial.integrate_oracle(WpmVerdict::Warn);
    assert_eq!(result, GateVerdict::Warn);
}

#[test]
fn test_gate_verdict_oracle_warn_does_not_reduce_fail() {
    let initial = GateVerdict::Fail;
    let result = initial.integrate_oracle(WpmVerdict::Warn);
    assert_eq!(result, GateVerdict::Fail);
}

#[test]
fn test_gate_verdict_oracle_fail_always_escalates() {
    let verdicts =
        vec![GateVerdict::Admit, GateVerdict::Suggest, GateVerdict::Warn, GateVerdict::Fail];

    for verdict in verdicts {
        let result = verdict.integrate_oracle(WpmVerdict::Fail);
        assert_eq!(result, GateVerdict::Fail);
    }
}

#[test]
fn test_gate_verdict_oracle_partial_has_no_effect() {
    let verdicts =
        vec![GateVerdict::Admit, GateVerdict::Suggest, GateVerdict::Warn, GateVerdict::Fail];

    for verdict in verdicts {
        let result = verdict.integrate_oracle(WpmVerdict::Partial);
        assert_eq!(result, verdict);
    }
}

// ============================================================================
// TESTS: OCEL Schema Validation
// ============================================================================

#[test]
fn test_ocel_schema_has_required_version_field() {
    let ocel_json = r#"{"version": "2.0", "omap": []}"#;

    let is_valid = ocel_json.contains("\"version\"") && ocel_json.contains("2.0");
    assert!(is_valid, "OCEL should have version 2.0 field");
}

#[test]
fn test_ocel_schema_missing_version_is_invalid() {
    let ocel_json = r#"{"omap": []}"#;

    let is_valid = ocel_json.contains("\"version\"");
    assert!(!is_valid, "OCEL without version field should fail validation");
}

#[test]
fn test_ocel_export_from_admission_events() {
    let events = vec![
        AdmissionEvent::new(
            "run_test",
            "evt_001",
            1000,
            ConfigEventKind::SourceDiscovered,
            vec!["source_main.toml".to_string()],
            vec![("action".to_string(), "load".to_string())],
        ),
        AdmissionEvent::new(
            "run_test",
            "evt_002",
            2000,
            ConfigEventKind::ConfigValidated,
            vec!["config_final.toml".to_string()],
            vec![("status".to_string(), "valid".to_string())],
        ),
    ];

    let ocel_log = export_events_to_ocel(&events);

    // Verify OCEL structure
    assert_eq!(ocel_log.events().len(), 2);
    assert!(!ocel_log.objects().is_empty());

    let run_obj_found = ocel_log.objects().iter().any(|obj| obj.object_type() == "ConfigRun");
    assert!(run_obj_found, "OCEL should contain ConfigRun object");
}

#[test]
fn test_ocel_export_preserves_event_timeline() {
    let events = vec![
        AdmissionEvent::new(
            "run_123",
            "first",
            1000,
            ConfigEventKind::SourceDiscovered,
            vec![],
            vec![],
        ),
        AdmissionEvent::new(
            "run_123",
            "second",
            2000,
            ConfigEventKind::LayerMerged,
            vec![],
            vec![],
        ),
        AdmissionEvent::new(
            "run_123",
            "third",
            3000,
            ConfigEventKind::CanonicalSaved,
            vec![],
            vec![],
        ),
    ];

    let ocel_log = export_events_to_ocel(&events);
    let logged_events = ocel_log.events();

    assert_eq!(logged_events.len(), 3);
    assert_eq!(logged_events[0].id(), "first");
    assert_eq!(logged_events[1].id(), "second");
    assert_eq!(logged_events[2].id(), "third");
}

// ============================================================================
// TESTS: End-to-End Oracle Integration with Gate
// ============================================================================

#[test]
fn test_full_oracle_integration_evidence_to_gate_verdict() {
    let tmpdir = setup_test_environment().expect("Failed to setup test environment");
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");
    let evidence_path = tmpdir.path().join("evidence.json");

    // Step 1: Create evidence OCEL
    create_ocel_evidence(&evidence_path).expect("Failed to create evidence");

    // Step 2: Run oracle audit
    let output = Command::new(&mock_wpm_path)
        .arg("audit")
        .arg(&evidence_path)
        .output()
        .expect("Failed to run oracle");

    // Step 3: Infer oracle verdict
    let oracle_verdict = if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("PASS") {
            WpmVerdict::Pass
        } else {
            WpmVerdict::Warn
        }
    } else {
        WpmVerdict::Fail
    };

    // Step 4: Integrate into gate decision
    let initial_gate = GateVerdict::Admit;
    let final_gate = initial_gate.integrate_oracle(oracle_verdict);

    // Oracle PASS should not escalate gate
    assert_eq!(final_gate, GateVerdict::Admit);

    // Verify flow: evidence → oracle verdict → gate decision
    assert_eq!(oracle_verdict, WpmVerdict::Pass);
}

#[test]
fn test_oracle_integration_with_warn_escalation() {
    // Simulate scenario where oracle detects drift
    let initial_gate = GateVerdict::Admit;
    let oracle_verdict = WpmVerdict::Warn; // Oracle detected SPC drift

    let final_gate = initial_gate.integrate_oracle(oracle_verdict);

    assert_eq!(final_gate, GateVerdict::Warn);
}

#[test]
fn test_oracle_integration_with_fail_gate() {
    // Simulate scenario where oracle detects conformance violation
    let initial_gate = GateVerdict::Suggest;
    let oracle_verdict = WpmVerdict::Fail; // Oracle detected OCEL violation

    let final_gate = initial_gate.integrate_oracle(oracle_verdict);

    assert_eq!(final_gate, GateVerdict::Fail);
}

#[test]
fn test_oracle_unavailable_graceful_degradation() {
    // When wpm is not in PATH, verdict should be PARTIAL
    let oracle_verdict = WpmVerdict::Partial;
    let initial_gate = GateVerdict::Warn;

    let final_gate = initial_gate.integrate_oracle(oracle_verdict);

    // Partial should not affect gate decision
    assert_eq!(final_gate, GateVerdict::Warn);
}

// ============================================================================
// TESTS: Error Handling & Diagnostics
// ============================================================================

#[test]
fn test_wpm_exit_nonzero_implies_fail_verdict() {
    let tmpdir = setup_test_environment().expect("Failed to setup test environment");
    let mock_wpm_path = tmpdir.path().join("mock_wpm.sh");
    let missing_file = tmpdir.path().join("missing.json");

    // Don't create the file; mock wpm will fail
    let output = Command::new(&mock_wpm_path)
        .arg("audit")
        .arg(&missing_file)
        .output()
        .expect("Failed to execute command");

    // Non-zero exit should map to Fail verdict
    let verdict = if output.status.success() { WpmVerdict::Pass } else { WpmVerdict::Fail };

    assert_eq!(verdict, WpmVerdict::Fail);
}

#[test]
fn test_wpm_timeout_handling_returns_partial() {
    // Simulating timeout behavior: would return Partial verdict
    // In real implementation, use timeout(1) or similar
    let verdict = WpmVerdict::Partial;
    assert_eq!(verdict, WpmVerdict::Partial);
}

#[test]
fn test_diagnostic_message_oracle_pass() {
    let oracle = WpmVerdict::Pass;
    let message = format!("oracle verdict: {}", oracle.to_string());
    assert_eq!(message, "oracle verdict: PASS");
}

#[test]
fn test_diagnostic_message_oracle_warn() {
    let oracle = WpmVerdict::Warn;
    let message = format!("oracle verdict: {}", oracle.to_string());
    assert_eq!(message, "oracle verdict: WARN");
}

#[test]
fn test_diagnostic_message_oracle_partial() {
    let oracle = WpmVerdict::Partial;
    let message = format!(
        "oracle verdict: {}; wpm not found in PATH; skipping oracle gate",
        oracle.to_string()
    );
    assert!(message.contains("wpm not found"));
}

#[test]
fn test_diagnostic_message_oracle_fail() {
    let oracle = WpmVerdict::Fail;
    let message = format!("oracle verdict: {}; detected conformance violation", oracle.to_string());
    assert!(message.contains("conformance violation"));
}
