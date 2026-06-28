//! Validate the LSP analyzer against the actual TOML fixture files in
//! `examples/config_patterns/`. This ensures that the diagnostic rules
//! fire (or stay silent) on the same inputs the star-toml core library
//! uses in its own examples.

use lsp_max::lsp_types_max::Url;
use star_toml_lsp::{
    analyzer::StarTomlDocumentAnalyzer,
    diagnostics::{INVALID_TOML, PATH_TRAVERSAL},
};
use std::path::Path;

fn load_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/config_patterns")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
}

fn analyze(name: &str) -> StarTomlDocumentAnalyzer {
    let text = load_fixture(name);
    let uri: Url = format!("file:///fixtures/{name}").parse().unwrap();
    StarTomlDocumentAnalyzer::new(uri, text)
}

fn has_code(a: &StarTomlDocumentAnalyzer, code: &str) -> bool {
    a.diagnostics().iter().any(|d| {
        matches!(&d.code,
            Some(lsp_max::lsp_types_max::NumberOrString::String(c)) if c == code)
    })
}

// ---------------------------------------------------------------------------
// Valid fixtures — analyzer must parse them cleanly (no INVALID_TOML)
// ---------------------------------------------------------------------------

#[test]
fn fixture_service_toml_parses_clean() {
    let a = analyze("service.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "service.toml is valid TOML — unexpected parse error: {:?}", a.diagnostics());
}

#[test]
fn fixture_service_dev_parses_clean() {
    let a = analyze("service.dev.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "service.dev.toml should parse clean");
}

#[test]
fn fixture_service_prod_parses_clean() {
    let a = analyze("service.prod.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "service.prod.toml should parse clean");
}

#[test]
fn fixture_database_toml_parses_clean() {
    let a = analyze("database.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "database.toml should parse clean");
}

#[test]
fn fixture_web_app_parses_clean() {
    let a = analyze("web_app.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "web_app.toml should parse clean");
}

#[test]
fn fixture_workers_parses_clean() {
    let a = analyze("workers.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "workers.toml (array-of-tables) should parse clean");
}

#[test]
fn fixture_nested_arrays_parses_clean() {
    let a = analyze("nested_arrays.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "nested_arrays.toml should parse clean");
}

#[test]
fn fixture_paths_parses_clean() {
    let a = analyze("paths.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "paths.toml should parse clean");
}

#[test]
fn fixture_feature_flags_parses_clean() {
    let a = analyze("feature_flags.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "feature_flags.toml should parse clean");
}

// ---------------------------------------------------------------------------
// Invalid fixtures — analyzer must fire the expected diagnostic
// ---------------------------------------------------------------------------

#[test]
fn fixture_invalid_path_traversal_fires_diagnostic() {
    let a = analyze("invalid_path_traversal.toml");
    assert!(
        has_code(&a, PATH_TRAVERSAL),
        "invalid_path_traversal.toml must produce path_traversal_detected diagnostic.\n\
         Got: {:?}", a.diagnostics()
    );
}

#[test]
fn fixture_invalid_unknown_field_analyzed_without_crash() {
    // The LSP cannot detect unknown fields without a schema type.
    // Verify it at least parses and does not panic.
    let a = analyze("invalid_unknown_field.toml");
    assert!(!has_code(&a, INVALID_TOML),
        "invalid_unknown_field.toml is syntactically valid TOML — parse must succeed");
    // Without a registered schema, the LSP cannot diagnose unknown fields at
    // open time. This is a known limitation: schema-aware unknown-field
    // detection requires TrustedLoader integration from the caller.
    // TODO(ST-110): integrate schema inference from workspace Cargo.toml.
}

// ---------------------------------------------------------------------------
// Document symbols — all valid fixtures must produce ≥1 symbol
// ---------------------------------------------------------------------------

#[test]
fn fixture_service_toml_has_symbols() {
    let a = analyze("service.toml");
    assert!(!a.document_symbols().is_empty(),
        "service.toml should produce at least one document symbol");
}

#[test]
fn fixture_workers_array_of_tables_has_symbols() {
    let a = analyze("workers.toml");
    // workers is an array-of-tables; the top-level key "workers" should appear.
    let syms = a.document_symbols();
    assert!(!syms.is_empty(),
        "workers.toml should produce symbols for the [[workers]] array-of-tables");
}

// ---------------------------------------------------------------------------
// Gate: all valid fixtures produce ANDON=0
// ---------------------------------------------------------------------------

#[test]
fn fixture_valid_files_produce_zero_gate() {
    let dir = tempfile::tempdir().unwrap();
    for name in &[
        "service.toml",
        "service.dev.toml",
        "service.prod.toml",
        "database.toml",
        "web_app.toml",
        "workers.toml",
        "nested_arrays.toml",
        "paths.toml",
        "feature_flags.toml",
    ] {
        let a = analyze(name);
        let state = star_toml_lsp::gate::GateState::from_diagnostics(&a.diagnostics());
        star_toml_lsp::gate::write_gate(dir.path(), &state);
        let content = std::fs::read_to_string(dir.path().join(".star-toml/lsp.gate")).unwrap();
        assert!(
            content.contains("ANDON=0"),
            "{name} should produce ANDON=0 gate, got: {content}"
        );
    }
}

#[test]
fn fixture_path_traversal_produces_andon_1() {
    let dir = tempfile::tempdir().unwrap();
    let a = analyze("invalid_path_traversal.toml");
    let state = star_toml_lsp::gate::GateState::from_diagnostics(&a.diagnostics());
    star_toml_lsp::gate::write_gate(dir.path(), &state);
    let content = std::fs::read_to_string(dir.path().join(".star-toml/lsp.gate")).unwrap();
    assert!(content.contains("ANDON=1"),
        "invalid_path_traversal.toml should produce ANDON=1");
}
