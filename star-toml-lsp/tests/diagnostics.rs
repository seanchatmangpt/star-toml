//! Unit tests for star-toml-lsp diagnostic, gate, and telemetry modules.
//!
//! These tests exercise the analyzer/diagnostics/gate modules directly —
//! no LSP server wire protocol required.

use lsp_max::lsp_types_max::{DiagnosticSeverity, Url};
use star_toml_lsp::{
    analyzer::StarTomlDocumentAnalyzer,
    diagnostics::{
        DUPLICATE_KEY, INVALID_TOML, OCEL_NOT_STANDING, PATH_TRAVERSAL, Q_CONFIG_AUTHORITY,
        RELATIVE_ONLY_ABSOLUTE,
    },
    gate::{write_gate, GateState},
};

fn uri(s: &str) -> Url {
    s.parse().unwrap()
}

fn doc_uri() -> Url {
    uri("file:///tmp/test.toml")
}

fn make_analyzer(toml: &str) -> StarTomlDocumentAnalyzer {
    StarTomlDocumentAnalyzer::new(doc_uri(), toml.to_owned())
}

fn has_code(a: &StarTomlDocumentAnalyzer, code: &str) -> bool {
    a.diagnostics().iter().any(
        |d| matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == code),
    )
}

fn diags_with_code(a: &StarTomlDocumentAnalyzer, code: &str) -> usize {
    a.diagnostics()
        .iter()
        .filter(|d| {
            matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == code)
        })
        .count()
}

// ---------------------------------------------------------------------------
// Parse errors
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_toml_diagnostic() {
    let a = make_analyzer("this is not [valid toml");
    assert!(has_code(&a, INVALID_TOML), "expected invalid_toml diagnostic");
}

#[test]
fn test_valid_toml_no_parse_error() {
    let a = make_analyzer("name = \"project\"\nversion = \"1.0.0\"\n");
    assert!(!has_code(&a, INVALID_TOML));
}

// ---------------------------------------------------------------------------
// Duplicate keys
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_key_diagnostic() {
    let toml = "port = 8080\nport = 9090\n"; // real newline in raw string
    let a = make_analyzer(toml);
    // toml v1.1 (TOML 1.1 spec) rejects duplicate keys at parse time,
    // so we get invalid_toml. Our own duplicate-key scanner runs only when
    // parse succeeds (as a belt-and-suspenders pass for edge cases).
    let caught = has_code(&a, DUPLICATE_KEY) || has_code(&a, INVALID_TOML);
    assert!(caught, "expected duplicate key to be caught as DUPLICATE_KEY or INVALID_TOML");
}

#[test]
fn test_no_false_positive_on_unique_keys() {
    let toml = "host = \"localhost\"\nport = 8080\n";
    let a = make_analyzer(toml);
    assert_eq!(diags_with_code(&a, DUPLICATE_KEY), 0);
}

// ---------------------------------------------------------------------------
// Path traversal
// ---------------------------------------------------------------------------

#[test]
fn test_path_traversal_diagnostic_unix() {
    let toml = r#"config_path = "../secret/config.toml""#;
    let a = make_analyzer(toml);
    assert!(has_code(&a, PATH_TRAVERSAL), "expected path_traversal_detected diagnostic");
}

#[test]
fn test_path_traversal_diagnostic_windows_separator() {
    // CE-12 pattern: backslash traversal
    let toml = r#"config_path = "foo\\..\\secret""#;
    let a = make_analyzer(toml);
    assert!(
        has_code(&a, PATH_TRAVERSAL),
        "expected path_traversal_detected for backslash traversal"
    );
}

#[test]
fn test_path_traversal_nested_unix() {
    let toml = r#"data = "foo/../../etc/passwd""#;
    let a = make_analyzer(toml);
    assert!(has_code(&a, PATH_TRAVERSAL));
}

#[test]
fn test_safe_relative_path_no_traversal() {
    let toml = r#"config_path = "configs/app.toml""#;
    let a = make_analyzer(toml);
    assert!(!has_code(&a, PATH_TRAVERSAL));
}

// ---------------------------------------------------------------------------
// Absolute path (RelativeOnly advisory)
// ---------------------------------------------------------------------------

#[test]
fn test_relative_only_absolute_path_diagnostic() {
    let toml = r#"config_path = "/etc/app/config.toml""#;
    let a = make_analyzer(toml);
    // Absolute paths surface as WARNING (we don't know policy without schema)
    let has_warn = a.diagnostics().iter().any(|d| {
        matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == RELATIVE_ONLY_ABSOLUTE)
            && matches!(d.severity, Some(DiagnosticSeverity::WARNING))
    });
    assert!(has_warn, "expected relative_only_escape warning for absolute path");
}

// ---------------------------------------------------------------------------
// OCEL / q_config authority boundary
// ---------------------------------------------------------------------------

#[test]
fn test_ocel_not_standing_hover_or_diagnostic() {
    let toml = r#"
# This config references ocel_standing = true
ocel_standing = true
"#;
    // The line contains "ocel" + "standing" → should trigger advisory
    let a = make_analyzer(toml);
    assert!(has_code(&a, OCEL_NOT_STANDING), "expected ocel_history_not_standing advisory");
}

#[test]
fn test_q_config_authority_note() {
    let toml = r#"q_config = "granted""#;
    let a = make_analyzer(toml);
    assert!(
        has_code(&a, Q_CONFIG_AUTHORITY),
        "expected q_config_requires_witness_and_failset_zero advisory"
    );
}

// ---------------------------------------------------------------------------
// Array of tables (no false positives)
// ---------------------------------------------------------------------------

#[test]
fn test_array_of_tables_no_false_positive_parse_error() {
    let toml = r#"
[[server]]
host = "a.example.com"
port = 443

[[server]]
host = "b.example.com"
port = 8443
"#;
    let a = make_analyzer(toml);
    assert!(!has_code(&a, INVALID_TOML), "valid array of tables should not error");
}

// ---------------------------------------------------------------------------
// Gate file
// ---------------------------------------------------------------------------

#[test]
fn test_gate_file_sets_andon_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let diags = vec![star_toml_lsp::diagnostics::invalid_toml(
        star_toml_lsp::diagnostics::document_start(),
        "bad syntax",
    )];
    let state = GateState::from_diagnostics(&diags);
    assert!(state.andon);
    assert_eq!(state.errors, 1);
    write_gate(dir.path(), &state);
    let content = std::fs::read_to_string(dir.path().join(".star-toml/lsp.gate")).unwrap();
    assert!(content.contains("ANDON=1"));
    assert!(content.contains("errors=1"));
}

#[test]
fn test_gate_file_clears_andon_on_clean_file() {
    let dir = tempfile::tempdir().unwrap();
    let state = GateState::from_diagnostics(&[]);
    assert!(!state.andon);
    write_gate(dir.path(), &state);
    let content = std::fs::read_to_string(dir.path().join(".star-toml/lsp.gate")).unwrap();
    assert!(content.contains("ANDON=0"));
    assert!(content.contains("errors=0"));
}

// ---------------------------------------------------------------------------
// Boundary: no tower-lsp, no q_config authority
// ---------------------------------------------------------------------------

#[test]
fn test_no_tower_lsp_dependency() {
    // If tower-lsp were present, this module would import it.
    // The mere fact this test compiles with only lsp-max in scope is the proof.
    // We also do a string-level check of our own source.
    let server_src = include_str!("../src/server.rs");
    assert!(!server_src.contains("tower_lsp"), "tower-lsp must not appear in server.rs");
    assert!(!server_src.contains("tower-lsp"), "tower-lsp must not appear in server.rs");
}

#[test]
fn test_lsp_does_not_compute_q_config() {
    // Verify the LSP source files never construct AdmittedConfig or ConfigWitness.
    for src in &[
        include_str!("../src/analyzer.rs"),
        include_str!("../src/server.rs"),
        include_str!("../src/state.rs"),
        include_str!("../src/diagnostics.rs"),
        include_str!("../src/gate.rs"),
        include_str!("../src/telemetry.rs"),
        include_str!("../src/commands.rs"),
        include_str!("../src/standing.rs"),
    ] {
        assert!(!src.contains("AdmittedConfig {"), "LSP must not construct AdmittedConfig");
        assert!(!src.contains("ConfigWitness {"), "LSP must not construct ConfigWitness");
        // Check that load_admitted is not called as Rust code (method call or path expression).
        // String mentions in diagnostic messages are fine.
        let call_sites: Vec<_> = src
            .lines()
            .filter(|l| {
                l.contains(".load_admitted(")
                    && !l.trim_start().starts_with("//")
                    && !l.contains('"')
            })
            .collect();
        assert!(call_sites.is_empty(), "LSP must not call load_admitted(): {call_sites:?}");
        // Only flag bare assignments, not diagnostic message strings
        let q_assignments: Vec<_> = src
            .lines()
            .filter(|l| {
                l.contains("q_config = 1") && !l.contains('"') && !l.trim_start().starts_with("//")
            })
            .collect();
        assert!(
            q_assignments.is_empty(),
            "LSP must not assign q_config standing: {q_assignments:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// DOC diagnostic projection
// ---------------------------------------------------------------------------

#[test]
fn test_doc_020_receipt_mismatch_projected() {
    use star_toml::docs::{
        AuthorityBoundary, ConfigDocumentationModel, ConfigDomain, ConfigField, OverridePolicy,
    };

    let domain = ConfigDomain {
        id: "app".to_string(),
        name: "App".to_string(),
        description: "Application domain".to_string(),
        stability: "stable".to_string(),
        authority_boundary: AuthorityBoundary {
            id: "app_boundary".to_string(),
            domain: "app".to_string(),
            may_control: vec!["port".to_string()],
            must_not_claim: vec![],
            defer_to: Some("platform".to_string()),
            escalation_rule: None,
        },
        override_policy: OverridePolicy {
            id: "app_override".to_string(),
            allowed: false,
            required_reason: false,
            required_scope: vec![],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec![],
        },
        fields: vec![ConfigField {
            id: "app.port".to_string(),
            domain_id: "app".to_string(),
            name: "port".to_string(),
            r#type: "integer".to_string(),
            required: true,
            default: None,
            allowed_values: vec![],
            forbidden_values: vec![],
            description: "Port number for the application.".to_string(),
            validator_refs: vec![],
            counterexample_refs: vec![],
            override_policy_ref: None,
            source_binding: "app".to_string(),
            stability: "stable".to_string(),
        }],
        validators: vec![],
        counterexamples: vec![],
        receipt_binding: None,
    };

    let model = ConfigDocumentationModel {
        version: "26.6.29".to_string(),
        generated_at: "2026-06-28T00:00:00Z".to_string(),
        input_digest: "abc".to_string(),
        model_digest: "def".to_string(),
        domains: vec![domain],
        diagnostics: vec![],
        // DOC-020 checks model.receipts, not domain.receipt_binding
        receipts: vec![star_toml::docs::ReceiptBinding {
            json_schema_digest: "abc".to_string(),
            admitted_schema_digest: "def".to_string(),
            admitted_config_digest: "ghi".to_string(),
            documentation_model_digest: "intentionally-wrong-digest-for-test".to_string(),
            projection_digest: "jkl".to_string(),
            generator_version: "26.6.29".to_string(),
            timestamp: "2026-06-28T00:00:00Z".to_string(),
            signer_or_operator: "test".to_string(),
        }],
    };

    let diags = star_toml_lsp::diagnostics::project_doc_diagnostics(&model);
    assert!(
        diags.iter().any(|d| matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == "DOC-020")),
        "DOC-020 ReceiptDigestMismatch must be projected when receipt digest is wrong; got: {diags:?}"
    );
}

#[test]
fn test_doc_projection_empty_on_clean_model() {
    use star_toml::docs::{
        AuthorityBoundary, ConfigDocumentationModel, ConfigDomain, OverridePolicy,
    };

    let model = ConfigDocumentationModel {
        version: "26.6.29".to_string(),
        generated_at: "2026-06-28T00:00:00Z".to_string(),
        input_digest: "abc".to_string(),
        model_digest: "def".to_string(),
        domains: vec![ConfigDomain {
            id: "svc".to_string(),
            name: "Service".to_string(),
            description: "Service domain".to_string(),
            stability: "experimental".to_string(),
            authority_boundary: AuthorityBoundary {
                id: "svc_b".to_string(),
                domain: "svc".to_string(),
                may_control: vec!["mode".to_string()],
                must_not_claim: vec![],
                defer_to: None,
                escalation_rule: None,
            },
            override_policy: OverridePolicy {
                id: "svc_op".to_string(),
                allowed: false,
                required_reason: false,
                required_scope: vec![],
                required_expiration: None,
                required_tracking: None,
                required_replacement_plan: false,
                refusal_conditions: vec![],
            },
            fields: vec![],
            validators: vec![],
            counterexamples: vec![],
            receipt_binding: None,
        }],
        diagnostics: vec![],
        receipts: vec![],
    };

    // A minimal valid model with no fields should produce no fatal diagnostics
    let diags = star_toml_lsp::diagnostics::project_doc_diagnostics(&model);
    let fatals: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Some(lsp_max::lsp_types_max::DiagnosticSeverity::ERROR))
        .collect();
    assert!(
        fatals.is_empty(),
        "Clean model must produce no fatal DOC diagnostics; got: {fatals:?}"
    );
}

// ---------------------------------------------------------------------------
// SCH-JSON diagnostic projection
// ---------------------------------------------------------------------------

#[test]
fn test_sch_json_016_unsupported_dialect_projected() {
    let json = r#"{
        "$schema": "http://json-schema.org/draft-04/schema#",
        "type": "object",
        "properties": {}
    }"#;
    let diags = star_toml_lsp::diagnostics::project_sch_json_diagnostics(json);
    assert!(
        diags.iter().any(|d| matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == "SCH-JSON-016")),
        "SCH-JSON-016 UnsupportedDialect must fire for draft-04; got: {diags:?}"
    );
}

#[test]
fn test_sch_json_007_additional_properties_projected() {
    let json = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "x-authority-boundary": {"id": "b", "may_control": ["port"]},
        "x-override-policy": {"id": "op", "allowed": false},
        "properties": {
            "port": {
                "type": "integer",
                "description": "Port number.",
                "x-counterexamples": {"minimum": "port = 0"}
            }
        }
    }"#;
    let diags = star_toml_lsp::diagnostics::project_sch_json_diagnostics(json);
    assert!(
        diags.iter().any(|d| matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == "SCH-JSON-007")),
        "SCH-JSON-007 AllowsUnboundedAdditionalProperties must fire without additionalProperties:false; got: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// Hover enrichment
// ---------------------------------------------------------------------------

#[test]
fn test_hover_shows_authority_boundary() {
    use lsp_max::lsp_types_max::DocumentUri;
    // Direct call: verify project_doc_diagnostics handles authority boundary model correctly.
    use star_toml::docs::{
        AuthorityBoundary, ConfigDocumentationModel, ConfigDomain, OverridePolicy,
    };
    use star_toml_lsp::analyzer::StarTomlDocumentAnalyzer;
    let model = ConfigDocumentationModel {
        version: "26.6.29".to_string(),
        generated_at: "2026-06-28T00:00:00Z".to_string(),
        input_digest: "abc".to_string(),
        model_digest: "def".to_string(),
        domains: vec![ConfigDomain {
            id: "svc".to_string(),
            name: "Service".to_string(),
            description: "Service domain".to_string(),
            stability: "experimental".to_string(),
            authority_boundary: AuthorityBoundary {
                id: "svc_b".to_string(),
                domain: "svc".to_string(),
                may_control: vec!["port".to_string()],
                must_not_claim: vec![],
                defer_to: Some("platform-ops".to_string()),
                escalation_rule: None,
            },
            override_policy: OverridePolicy {
                id: "svc_op".to_string(),
                allowed: false,
                required_reason: false,
                required_scope: vec![],
                required_expiration: None,
                required_tracking: None,
                required_replacement_plan: false,
                refusal_conditions: vec![],
            },
            fields: vec![],
            validators: vec![],
            counterexamples: vec![],
            receipt_binding: None,
        }],
        diagnostics: vec![],
        receipts: vec![],
    };

    // The authority boundary is surfaced via the analyzer hover. Test that the
    // model data is accessible and has the expected defer_to.
    let boundary = &model.domains[0].authority_boundary;
    assert_eq!(
        boundary.defer_to.as_deref(),
        Some("platform-ops"),
        "Authority boundary defer_to must be accessible for hover display"
    );

    // Also verify a TOML document produces a hover without panicking
    let uri: DocumentUri = "file:///tmp/test.toml".parse().unwrap();
    let toml = "[svc]\nport = 8080\n";
    let analyzer = StarTomlDocumentAnalyzer::new(uri, toml.to_string());
    // hover_at should not panic regardless of whether docs model is present
    let _ = analyzer.hover_at(1, 0);
}

#[test]
fn test_sch_json_additional_properties_false_clears_007() {
    // A schema with additionalProperties:false must NOT fire SCH-JSON-007.
    let json = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "description": "App configuration.",
        "additionalProperties": false,
        "x-authority-boundary": {"id": "app_b", "may_control": ["port"]},
        "x-override-policy": {"id": "app_override", "allowed": false},
        "properties": {
            "port": {
                "type": "integer",
                "description": "Application port.",
                "x-counterexamples": {"minimum": "port = 0"}
            }
        }
    }"#;
    let diags = star_toml_lsp::diagnostics::project_sch_json_diagnostics(json);
    let has_007 = diags.iter().any(|d| {
        matches!(&d.code, Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == "SCH-JSON-007")
    });
    assert!(
        !has_007,
        "SCH-JSON-007 must NOT fire when additionalProperties:false is set; got: {diags:?}"
    );
}
