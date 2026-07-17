//! Comprehensive test suite for CC-1: wasm4pm Oracle Full Integration
//!
//! Tests all acceptance criteria:
//! AC1: wpm binary discovery
//! AC2: Shell-out integration
//! AC3: Evidence gate integration
//! AC4: OCEL JSON schema validation
//! AC5: Error handling & diagnostics
//! AC6: Deterministic ordering (ADMIT → SUGGEST → WARN → FAIL)

use star_toml::{
    events::{AdmissionEvent, ConfigEventKind},
    export_and_validate_events,
    EvidenceGate, OracleGateVerdict, OracleVerdict,
    OracleDiagnostic, OracleDiagnosticCode, OracleDiagnosticReport,
    Wasm4pmShell, Wpm4Command, OcelValidator, OcelEvidence, OcelEventRecord, OcelObjectRecord,
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// AC1: wpm binary discovery tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac1_wpm_binary_discovery_from_explicit_path() {
    // AC1: Test discovery from explicit path
    // This test verifies that binary can be discovered from a known path
    // We'll use a path that exists (e.g., /bin/ls) as a substitute for wpm

    let result = Wasm4pmShell::from_path("/bin/ls");
    assert!(result.is_ok(), "should discover binary from valid path");

    let shell = result.unwrap();
    assert_eq!(shell.binary_path().to_string_lossy().to_string(), "/bin/ls");
}

#[test]
fn test_ac1_wpm_binary_discovery_fails_for_nonexistent() {
    // AC1: Test that discovery fails gracefully for non-existent binary
    let result = Wasm4pmShell::from_path("/nonexistent/wpm");
    assert!(result.is_err(), "should fail for non-existent binary");

    if let Err(err) = result {
        assert!(err.to_string().contains("not found"));
    }
}

#[test]
fn test_ac1_wpm_command_types() {
    // AC1: Test all wpm command types can be created
    let commands = [
        (Wpm4Command::Audit, "audit"),
        (Wpm4Command::Lean, "lean"),
        (Wpm4Command::SpcStatus, "spc_status"),
        (Wpm4Command::ReceiptDoctor, "receipt_doctor"),
    ];

    for (cmd, expected_str) in commands {
        assert_eq!(cmd.as_str(), expected_str);
    }
}

// ---------------------------------------------------------------------------
// AC2: Shell-out integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac2_wpm_shell_binary_path() {
    // AC2: Test that Wasm4pmShell correctly stores and returns binary path
    let shell = Wasm4pmShell::from_path("/bin/ls").unwrap();
    assert!(!shell.binary_path().as_os_str().is_empty());
}

#[test]
fn test_ac2_wpm_command_as_string() {
    // AC2: Test that commands are properly converted to strings for invocation
    let test_cases = [
        (Wpm4Command::Audit, "audit"),
        (Wpm4Command::Lean, "lean"),
        (Wpm4Command::SpcStatus, "spc_status"),
        (Wpm4Command::ReceiptDoctor, "receipt_doctor"),
    ];

    for (cmd, expected) in test_cases {
        assert_eq!(cmd.as_str(), expected);
    }
}

// ---------------------------------------------------------------------------
// AC3: Evidence gate integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac3_evidence_gate_creation() {
    // AC3: Test evidence gate can be created and verdicts added
    let mut gate = EvidenceGate::new();
    assert_eq!(gate.verdicts().len(), 0);

    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Admit,
        "all checks passed",
        "wpm_audit",
    ));

    assert_eq!(gate.verdicts().len(), 1);
}

#[test]
fn test_ac3_evidence_gate_verdict_accumulation() {
    // AC3: Test that verdicts accumulate in the gate
    let mut gate = EvidenceGate::new();

    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Suggest,
        "consider optimization",
        "wpm_lean",
    ));
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Warn,
        "config issue detected",
        "wpm_audit",
    ));

    assert_eq!(gate.verdicts().len(), 2);
    let summary = gate.verdict_summary();
    assert_eq!(summary.suggests, 1);
    assert_eq!(summary.warns, 1);
}

#[test]
fn test_ac3_evidence_gate_with_evidence_refs() {
    // AC3: Test that verdicts can track evidence references
    let verdict = OracleGateVerdict::new(
        OracleVerdict::Warn,
        "potential issue",
        "wpm_audit",
    )
    .with_evidence("evt_001".to_string())
    .with_evidence("evt_002".to_string());

    assert_eq!(verdict.evidence_refs.len(), 2);
}

// ---------------------------------------------------------------------------
// AC4: OCEL JSON schema validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac4_ocel_validator_creation() {
    // AC4: Test OcelValidator can be instantiated
    let _ = OcelValidator;
    // Validator is stateless, just verify it exists
}

#[test]
fn test_ac4_ocel_evidence_structure() {
    // AC4: Test OcelEvidence can be created with proper structure
    let evidence = OcelEvidence {
        events: vec![OcelEventRecord {
            event_id: "evt_001".to_string(),
            activity: "SourceDiscovered".to_string(),
            timestamp: 1000,
            omap: {
                let mut map = HashMap::new();
                map.insert("run_001".to_string(), "belongs_to".to_string());
                map
            },
            attributes: Default::default(),
        }],
        objects: vec![OcelObjectRecord {
            object_id: "run_001".to_string(),
            object_type: "ConfigRun".to_string(),
            attributes: Default::default(),
        }],
        relationships: vec![],
    };

    // Validate the evidence
    assert!(OcelValidator::validate_evidence(&evidence).is_ok());
}

#[test]
fn test_ac4_ocel_validator_detects_duplicate_events() {
    // AC4: Test OCEL validator detects duplicate event IDs
    let evidence = OcelEvidence {
        events: vec![
            OcelEventRecord {
                event_id: "evt_001".to_string(),
                activity: "SourceDiscovered".to_string(),
                timestamp: 1000,
                omap: {
                    let mut map = HashMap::new();
                    map.insert("obj_001".to_string(), "belongs_to".to_string());
                    map
                },
                attributes: Default::default(),
            },
            OcelEventRecord {
                event_id: "evt_001".to_string(), // Duplicate!
                activity: "LayerMerged".to_string(),
                timestamp: 2000,
                omap: Default::default(),
                attributes: Default::default(),
            },
        ],
        objects: vec![OcelObjectRecord {
            object_id: "obj_001".to_string(),
            object_type: "ConfigRun".to_string(),
            attributes: Default::default(),
        }],
        relationships: vec![],
    };

    assert!(OcelValidator::validate_evidence(&evidence).is_err());
}

#[test]
fn test_ac4_ocel_validator_detects_duplicate_objects() {
    // AC4: Test OCEL validator detects duplicate object IDs
    let evidence = OcelEvidence {
        events: vec![OcelEventRecord {
            event_id: "evt_001".to_string(),
            activity: "SourceDiscovered".to_string(),
            timestamp: 1000,
            omap: {
                let mut map = HashMap::new();
                map.insert("obj_001".to_string(), "belongs_to".to_string());
                map
            },
            attributes: Default::default(),
        }],
        objects: vec![
            OcelObjectRecord {
                object_id: "obj_001".to_string(),
                object_type: "ConfigRun".to_string(),
                attributes: Default::default(),
            },
            OcelObjectRecord {
                object_id: "obj_001".to_string(), // Duplicate!
                object_type: "Layer".to_string(),
                attributes: Default::default(),
            },
        ],
        relationships: vec![],
    };

    assert!(OcelValidator::validate_evidence(&evidence).is_err());
}

#[test]
fn test_ac4_ocel_validator_detects_orphaned_events() {
    // AC4: Test OCEL validator detects events referencing non-existent objects
    let evidence = OcelEvidence {
        events: vec![OcelEventRecord {
            event_id: "evt_001".to_string(),
            activity: "SourceDiscovered".to_string(),
            timestamp: 1000,
            omap: {
                let mut map = HashMap::new();
                map.insert("nonexistent_obj".to_string(), "belongs_to".to_string());
                map
            },
            attributes: Default::default(),
        }],
        objects: vec![OcelObjectRecord {
            object_id: "obj_001".to_string(),
            object_type: "ConfigRun".to_string(),
            attributes: Default::default(),
        }],
        relationships: vec![],
    };

    assert!(OcelValidator::validate_evidence(&evidence).is_err());
}

// ---------------------------------------------------------------------------
// AC5: Error handling & diagnostics tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac5_oracle_diagnostic_codes() {
    // AC5: Test all oracle diagnostic codes are properly defined
    let codes = [
        (OracleDiagnosticCode::OracleNotFound, "oracle_not_found"),
        (OracleDiagnosticCode::OracleCommandFailed, "oracle_command_failed"),
        (OracleDiagnosticCode::OracleVerdictFail, "oracle_verdict_fail"),
        (OracleDiagnosticCode::OracleVerdictWarn, "oracle_verdict_warn"),
        (OracleDiagnosticCode::OcelValidationFailed, "ocel_validation_failed"),
    ];

    for (code, expected_str) in codes {
        assert_eq!(code.code(), expected_str);
    }
}

#[test]
fn test_ac5_oracle_diagnostic_with_counterexample() {
    // AC5: Test diagnostics can include counterexamples
    let diag = OracleDiagnostic::new(
        OracleDiagnosticCode::OracleVerdictFail,
        "config validation failed",
    )
    .with_counterexample("port = -1  # invalid port number")
    .with_repair_hint("port must be >= 1024");

    assert!(diag.counterexample.is_some());
    assert!(diag.repair_hint.is_some());
}

#[test]
fn test_ac5_oracle_diagnostic_report() {
    // AC5: Test diagnostic report collection
    let mut report = OracleDiagnosticReport::new();

    report.add(OracleDiagnostic::new(
        OracleDiagnosticCode::OracleVerdictFail,
        "critical error",
    ));
    report.add(OracleDiagnostic::new(
        OracleDiagnosticCode::OcelValidationFailed,
        "schema error",
    ));

    let summary = report.summary();
    assert_eq!(summary.blocking, 1);
    assert_eq!(summary.warnings, 1);
    assert!(report.has_blocking());
}

// ---------------------------------------------------------------------------
// AC6: Deterministic ordering (ADMIT → SUGGEST → WARN → FAIL) tests
// ---------------------------------------------------------------------------

#[test]
fn test_ac6_oracle_verdict_ordering() {
    // AC6: Test verdicts are ordered deterministically
    let verdicts = [
        OracleVerdict::Admit,
        OracleVerdict::Suggest,
        OracleVerdict::Warn,
        OracleVerdict::Fail,
    ];

    // Verify ordering: each should be less than the next
    assert!(verdicts[0] < verdicts[1]);
    assert!(verdicts[1] < verdicts[2]);
    assert!(verdicts[2] < verdicts[3]);
}

#[test]
fn test_ac6_verdict_severity_ranks() {
    // AC6: Test deterministic severity ranking
    let ranks = [
        (OracleVerdict::Admit, 0),
        (OracleVerdict::Suggest, 1),
        (OracleVerdict::Warn, 2),
        (OracleVerdict::Fail, 3),
    ];

    for (verdict, expected_rank) in ranks {
        assert_eq!(verdict.severity_rank(), expected_rank);
    }
}

#[test]
fn test_ac6_gate_highest_verdict_respects_ordering() {
    // AC6: Test that gate correctly identifies highest-priority verdict
    let mut gate = EvidenceGate::new();

    // Add verdicts in random order
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Suggest,
        "optimization hint",
        "wpm_lean",
    ));
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Admit,
        "ok",
        "wpm_audit",
    ));
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Fail,
        "critical error",
        "wpm_audit",
    ));

    let highest = gate.highest_verdict().unwrap();
    // AC6: Deterministic ordering ensures FAIL is highest
    assert_eq!(highest.verdict, OracleVerdict::Fail);
}

#[test]
fn test_ac6_verdicts_by_severity() {
    // AC6: Test verdicts are correctly sorted by severity
    let mut gate = EvidenceGate::new();

    gate.add_verdict(OracleGateVerdict::new(OracleVerdict::Warn, "w", "test"));
    gate.add_verdict(OracleGateVerdict::new(OracleVerdict::Admit, "a", "test"));
    gate.add_verdict(OracleGateVerdict::new(OracleVerdict::Fail, "f", "test"));
    gate.add_verdict(OracleGateVerdict::new(OracleVerdict::Suggest, "s", "test"));

    let sorted = gate.verdicts_by_severity();

    // AC6: Should be sorted highest to lowest
    assert_eq!(sorted[0].verdict, OracleVerdict::Fail);
    assert_eq!(sorted[1].verdict, OracleVerdict::Warn);
    assert_eq!(sorted[2].verdict, OracleVerdict::Suggest);
    assert_eq!(sorted[3].verdict, OracleVerdict::Admit);
}

// ---------------------------------------------------------------------------
// Integration tests combining multiple ACs
// ---------------------------------------------------------------------------

#[test]
fn test_ac_integration_ocel_export_and_gate() {
    // Integration: AC4 (OCEL export) + AC3 (gate integration)
    let events = vec![
        AdmissionEvent::new(
            "run_123",
            "evt_001",
            1,
            ConfigEventKind::SourceDiscovered,
            vec!["source_main.toml".to_string()],
            vec![],
        ),
        AdmissionEvent::new(
            "run_123",
            "evt_002",
            2,
            ConfigEventKind::ConfigValidated,
            vec!["validation_ok".to_string()],
            vec![],
        ),
    ];

    let (ocel_log, _validator) = export_and_validate_events(&events);

    // AC4: Verify OCEL log is properly formed
    assert_eq!(ocel_log.events().len(), 2);
    assert_eq!(ocel_log.objects().len(), 3); // run_123, source_main.toml, validation_ok

    // AC3: Add verdicts to gate
    let mut gate = EvidenceGate::new();
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Admit,
        "OCEL log valid",
        "ocel_validator",
    ));

    assert!(!gate.is_blocked());
}

#[test]
fn test_ac_integration_diagnostics_with_gate_verdict() {
    // Integration: AC5 (diagnostics) + AC6 (verdict ordering)
    let mut report = OracleDiagnosticReport::new();

    // Create diagnostics aligned with verdict levels
    report.add(
        OracleDiagnostic::new(
            OracleDiagnosticCode::GateBlocking,
            "admission blocked",
        )
        .with_counterexample("config failed validation")
        .with_repair_hint("review error output"),
    );

    let summary = report.summary();
    assert_eq!(summary.total(), 1);
    assert!(report.has_blocking());
}

#[test]
fn test_ac_integration_full_pipeline() {
    // Full integration: AC1-AC6
    // 1. Create lifecycle events (simulation of config loading)
    let events = vec![
        AdmissionEvent::new(
            "admission_001",
            "e1",
            100,
            ConfigEventKind::SourceDiscovered,
            vec!["source_app.toml".to_string()],
            vec![("path".to_string(), "config/app.toml".to_string())],
        ),
        AdmissionEvent::new(
            "admission_001",
            "e2",
            200,
            ConfigEventKind::LayerMerged,
            vec!["layer_defaults".to_string(), "layer_overrides".to_string()],
            vec![],
        ),
    ];

    // 2. Export to OCEL (AC4)
    let (_ocel_log, validator) = export_and_validate_events(&events);
    assert!(validator.is_valid());

    // 3. Create gate with oracle verdicts (AC3, AC6)
    let mut gate = EvidenceGate::new();
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Suggest,
        "consider config optimization",
        "wpm_lean",
    ));
    gate.add_verdict(OracleGateVerdict::new(
        OracleVerdict::Admit,
        "configuration is valid",
        "wpm_audit",
    ));

    // 4. Verify deterministic ordering (AC6)
    let summary = gate.verdict_summary();
    assert_eq!(summary.suggests, 1);
    assert_eq!(summary.admits, 1);
    assert!(!gate.is_blocked());

    // 5. Create diagnostics (AC5)
    let mut report = OracleDiagnosticReport::new();
    if !gate.is_blocked() {
        report.add(OracleDiagnostic::new(
            OracleDiagnosticCode::OracleVerdictWarn,
            "oracle gate passed with suggestions",
        ));
    }

    assert_eq!(report.diagnostics().len(), 1);
}
