//! Integration tests for evidence gate and BLAKE3 receipt validation.
//!
//! AC4: Receipt validation still works (both in-process & affidavit)
//! AC5: Evidence gate uses BLAKE3 receipts

use star_toml::receipt_generation::{generate_receipt, validate_receipt};
use star_toml::nouns::EvidenceGate;
use star_toml::reports::{CoercedType, EnvOverrideEntry, EnvOverrideReport, LayerEntry, LayerReport, SourceEntry, SourceKind, SourceReport};
use star_toml::merge::WinnerMap;
use std::path::PathBuf;

fn make_test_source_report() -> SourceReport {
    SourceReport {
        entries: vec![
            SourceEntry {
                source_id: 0,
                source_kind: SourceKind::File,
                label: "config.toml".to_string(),
                path: Some(PathBuf::from("config.toml")),
                required: true,
                found: true,
                digest: Some("source_digest_1".to_string()),
                size_bytes: Some(100),
                source_root: None,
                source_parent: None,
            },
            SourceEntry {
                source_id: 1,
                source_kind: SourceKind::OptionalFile,
                label: "env_defaults".to_string(),
                path: None,
                required: false,
                found: false,
                digest: Some("source_digest_2".to_string()),
                size_bytes: Some(50),
                source_root: None,
                source_parent: None,
            },
        ],
    }
}

fn make_test_layer_report() -> LayerReport {
    LayerReport {
        entries: vec![
            LayerEntry {
                layer_id: 0,
                layer_name: "defaults".to_string(),
                priority: 0,
                source_id: 1,
                digest: "layer_digest_0".to_string(),
                layer_order_digest: "layer_order_0".to_string(),
                winning_field_map: WinnerMap::default(),
            },
            LayerEntry {
                layer_id: 1,
                layer_name: "file".to_string(),
                priority: 1,
                source_id: 0,
                digest: "layer_digest_1".to_string(),
                layer_order_digest: "layer_order_1".to_string(),
                winning_field_map: WinnerMap::default(),
            },
        ],
    }
}

fn make_test_env_report() -> EnvOverrideReport {
    EnvOverrideReport {
        prefix: "APP_".to_string(),
        entries: vec![
            EnvOverrideEntry {
                raw_env_key: "APP_PORT".to_string(),
                configured_prefix: "APP_".to_string(),
                mapped_path: "port".to_string(),
                raw_value_digest: "port_digest".to_string(),
                coerced_type: Some(CoercedType::Integer),
                coerced_value_digest: Some("port_coerced".to_string()),
                accepted: true,
                rejection_code: None,
            },
            EnvOverrideEntry {
                raw_env_key: "APP_HOST".to_string(),
                configured_prefix: "APP_".to_string(),
                mapped_path: "host".to_string(),
                raw_value_digest: "host_digest".to_string(),
                coerced_type: Some(CoercedType::Str),
                coerced_value_digest: Some("host_coerced".to_string()),
                accepted: true,
                rejection_code: None,
            },
            EnvOverrideEntry {
                raw_env_key: "APP_UNUSED".to_string(),
                configured_prefix: "APP_".to_string(),
                mapped_path: "unused".to_string(),
                raw_value_digest: "unused_digest".to_string(),
                coerced_type: None,
                coerced_value_digest: None,
                accepted: false,
                rejection_code: Some("bad_path".to_string()),
            },
        ],
    }
}

#[test]
fn test_generate_receipt_deterministic() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = br#"[server]
port = 8080
host = "0.0.0.0"
"#;
    let fitness = 1.0;

    let receipt1 = generate_receipt(&source, &layer, &env, fitness, canonical);
    let receipt2 = generate_receipt(&source, &layer, &env, fitness, canonical);

    assert_eq!(receipt1, receipt2, "Receipt generation must be deterministic");
    assert_eq!(receipt1.len(), 64, "BLAKE3 hex must be 64 characters");
}

#[test]
fn test_receipt_changes_with_different_sources() {
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    let source1 = make_test_source_report();

    let mut source2 = make_test_source_report();
    source2.entries[0].digest = Some("different_digest".to_string());

    let receipt1 = generate_receipt(&source1, &layer, &env, fitness, canonical);
    let receipt2 = generate_receipt(&source2, &layer, &env, fitness, canonical);

    assert_ne!(receipt1, receipt2, "Receipt should change when source digest changes");
}

#[test]
fn test_receipt_changes_with_different_layers() {
    let source = make_test_source_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    let layer1 = make_test_layer_report();

    let mut layer2 = make_test_layer_report();
    layer2.entries[1].layer_order_digest = "different_layer_digest".to_string();

    let receipt1 = generate_receipt(&source, &layer1, &env, fitness, canonical);
    let receipt2 = generate_receipt(&source, &layer2, &env, fitness, canonical);

    assert_ne!(receipt1, receipt2, "Receipt should change when layer digest changes");
}

#[test]
fn test_receipt_changes_with_different_env() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let canonical = b"config data";
    let fitness = 1.0;

    let env1 = make_test_env_report();

    let mut env2 = make_test_env_report();
    env2.entries[0].raw_value_digest = "different_value_digest".to_string();

    let receipt1 = generate_receipt(&source, &layer, &env1, fitness, canonical);
    let receipt2 = generate_receipt(&source, &layer, &env2, fitness, canonical);

    assert_ne!(receipt1, receipt2, "Receipt should change when env digest changes");
}

#[test]
fn test_receipt_changes_with_different_fitness() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";

    let receipt1 = generate_receipt(&source, &layer, &env, 1.0, canonical);
    let receipt2 = generate_receipt(&source, &layer, &env, 0.9, canonical);

    assert_ne!(receipt1, receipt2, "Receipt should change when fitness changes");
}

#[test]
fn test_receipt_changes_with_different_canonical() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let fitness = 1.0;

    let canonical1 = b"name = \"app\"";
    let canonical2 = b"name = \"app2\"";

    let receipt1 = generate_receipt(&source, &layer, &env, fitness, canonical1);
    let receipt2 = generate_receipt(&source, &layer, &env, fitness, canonical2);

    assert_ne!(receipt1, receipt2, "Receipt should change when canonical config changes");
}

#[test]
fn test_validate_receipt_accepts_matching_state() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"[server]\nport = 8080\n";
    let fitness = 1.0;

    let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);

    assert!(
        validate_receipt(&receipt, &source, &layer, &env, fitness, canonical),
        "Receipt should validate when state matches"
    );
}

#[test]
fn test_validate_receipt_rejects_modified_source() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config";
    let fitness = 1.0;

    let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);

    let mut modified_source = source.clone();
    modified_source.entries[0].digest = Some("modified".to_string());

    assert!(
        !validate_receipt(&receipt, &modified_source, &layer, &env, fitness, canonical),
        "Receipt should fail when source is modified"
    );
}

#[test]
fn test_validate_receipt_ignores_rejected_env_entries() {
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config";
    let fitness = 1.0;

    let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);

    // Create a report with the same accepted entries but no rejected entries.
    // Receipt should validate because rejected entries don't affect the hash.
    let accepted_only = EnvOverrideReport {
        prefix: env.prefix.clone(),
        entries: env
            .entries
            .iter()
            .filter(|e| e.accepted)
            .cloned()
            .collect(),
    };

    // This should validate because only accepted entries are hashed
    assert!(
        validate_receipt(&receipt, &source, &layer, &accepted_only, fitness, canonical),
        "Receipt should validate when rejected entries are removed (they don't affect the hash)"
    );
}

#[test]
fn test_evidence_gate_new_is_unlocked() {
    let gate = EvidenceGate::new();
    assert!(!gate.is_locked());
    assert!(gate.receipt().is_none());
}

#[test]
fn test_evidence_gate_pass_evidence_locks_and_records_receipt() {
    let mut gate = EvidenceGate::new();
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    let returned_receipt = gate.pass_evidence(&source, &layer, &env, fitness, canonical);

    assert!(gate.is_locked());
    assert_eq!(gate.receipt(), Some(returned_receipt.as_str()));
    assert!(!returned_receipt.is_empty());
    assert_eq!(returned_receipt.len(), 64);
}

#[test]
fn test_evidence_gate_validates_locked_same_state() {
    let mut gate = EvidenceGate::new();
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    gate.pass_evidence(&source, &layer, &env, fitness, canonical);

    // Same state should validate
    assert!(gate.validate(&source, &layer, &env, fitness, canonical));
}

#[test]
fn test_evidence_gate_rejects_locked_modified_state() {
    let mut gate = EvidenceGate::new();
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    gate.pass_evidence(&source, &layer, &env, fitness, canonical);

    // Modified canonical should fail
    assert!(!gate.validate(&source, &layer, &env, fitness, b"modified"));

    // Modified fitness should fail
    assert!(!gate.validate(&source, &layer, &env, 0.5, canonical));
}

#[test]
fn test_evidence_gate_reset_unlocks() {
    let mut gate = EvidenceGate::new();
    let source = make_test_source_report();
    let layer = make_test_layer_report();
    let env = make_test_env_report();
    let canonical = b"config data";
    let fitness = 1.0;

    gate.pass_evidence(&source, &layer, &env, fitness, canonical);
    assert!(gate.is_locked());

    gate.reset();
    assert!(!gate.is_locked());
    assert!(gate.receipt().is_none());

    // After reset, any state should validate
    assert!(gate.validate(&source, &layer, &env, 0.0, b"anything"));
}

#[test]
fn test_evidence_gate_multiple_transitions() {
    let mut gate = EvidenceGate::new();
    let source1 = make_test_source_report();
    let layer1 = make_test_layer_report();
    let env1 = make_test_env_report();
    let canonical1 = b"config v1";
    let fitness1 = 1.0;

    // First transition
    gate.pass_evidence(&source1, &layer1, &env1, fitness1, canonical1);
    assert!(gate.validate(&source1, &layer1, &env1, fitness1, canonical1));

    // Reset for new transition
    gate.reset();
    assert!(!gate.is_locked());

    // Second transition with modified state
    let source2 = make_test_source_report();
    let layer2 = make_test_layer_report();
    let env2 = make_test_env_report();
    let canonical2 = b"config v2";
    let fitness2 = 0.95;

    gate.pass_evidence(&source2, &layer2, &env2, fitness2, canonical2);
    assert!(gate.validate(&source2, &layer2, &env2, fitness2, canonical2));

    // Old state should not validate with new receipt
    assert!(!gate.validate(&source1, &layer1, &env1, fitness1, canonical1));
}

#[test]
fn test_evidence_gate_default_trait() {
    let gate = EvidenceGate::default();
    assert!(!gate.is_locked());
}
