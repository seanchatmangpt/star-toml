//! Runtime config conformance test helpers for star-toml (ST-113 / v26.6.29).
//!
//! Provides admission assertion helpers, counterexample runners, and a
//! DfCM matrix runner for verifying that star-toml's standing law holds
//! at runtime across all declared variant axes.
//!
//! This crate is **test infrastructure only**. It does not grant standing.
//! `AdmittedConfig<T>` is still the only authority.

use std::fmt;

use star_toml::docs::{ConfigDocumentationModel, ReceiptBinding};

// ---------------------------------------------------------------------------
// TestFailure
// ---------------------------------------------------------------------------

/// Reason a conformance assertion failed.
#[derive(Debug)]
pub enum TestFailure {
    AdmissionFailed(String),
    RefusalExpected { expected_code: String, got: String },
    DiagnosticMismatch { expected: String, found: Vec<String> },
    DocsFailed(String),
    ReceiptFailed(String),
    SchemaFailed(String),
    Unexpected(String),
}

impl fmt::Display for TestFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdmissionFailed(m) => write!(f, "AdmissionFailed: {m}"),
            Self::RefusalExpected { expected_code, got } => {
                write!(f, "Expected refusal with {expected_code}, got: {got}")
            }
            Self::DiagnosticMismatch { expected, found } => {
                write!(f, "Expected diagnostic {expected}, found: {found:?}")
            }
            Self::DocsFailed(m) => write!(f, "DocsFailed: {m}"),
            Self::ReceiptFailed(m) => write!(f, "ReceiptFailed: {m}"),
            Self::SchemaFailed(m) => write!(f, "SchemaFailed: {m}"),
            Self::Unexpected(m) => write!(f, "Unexpected: {m}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Schema helpers
// ---------------------------------------------------------------------------

/// Assert that a JSON Schema string is admitted by the bridge.
pub fn assert_schema_admitted(schema_json: &str) -> Result<(), TestFailure> {
    let result = star_toml_json_schema::import_json_schema(schema_json, true);
    match result {
        Ok(_) => Ok(()),
        Err(diags) => {
            let codes: Vec<String> = diags.iter().map(|d| d.code.to_string()).collect();
            Err(TestFailure::SchemaFailed(format!(
                "Schema should have been admitted but got diagnostics: {codes:?}"
            )))
        }
    }
}

/// Assert that a JSON Schema string is **refused** with `expected_code` among the diagnostics.
pub fn assert_schema_refused(schema_json: &str, expected_code: &str) -> Result<(), TestFailure> {
    let result = star_toml_json_schema::import_json_schema(schema_json, true);
    match result {
        Ok(_) => Err(TestFailure::RefusalExpected {
            expected_code: expected_code.to_string(),
            got: "schema was admitted (expected refusal)".to_string(),
        }),
        Err(diags) => {
            let codes: Vec<String> = diags.iter().map(|d| d.code.to_string()).collect();
            if codes.iter().any(|c| c == expected_code) {
                Ok(())
            } else {
                Err(TestFailure::DiagnosticMismatch {
                    expected: expected_code.to_string(),
                    found: codes,
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Docs helpers
// ---------------------------------------------------------------------------

/// Assert that all DOC diagnostics pass for the given model.
pub fn assert_docs_fresh(model: &ConfigDocumentationModel) -> Result<(), TestFailure> {
    let diags = star_toml::docs::validate_docs_integrity(model);
    let fatal: Vec<String> =
        diags.iter().filter(|d| d.severity == "Fatal").map(|d| d.code.to_string()).collect();
    if fatal.is_empty() {
        Ok(())
    } else {
        Err(TestFailure::DocsFailed(format!("Fatal DOC diagnostics: {fatal:?}")))
    }
}

/// Assert that `model` has at least one valid, non-empty receipt binding.
pub fn assert_receipt_bound(model: &ConfigDocumentationModel) -> Result<(), TestFailure> {
    let has_receipt = model
        .receipts
        .iter()
        .any(|r| !r.admitted_config_digest.is_empty() && !r.documentation_model_digest.is_empty());
    if has_receipt {
        Ok(())
    } else {
        Err(TestFailure::ReceiptFailed("Model has no valid receipt binding".to_string()))
    }
}

/// Assert that the model receipt is NOT bound (i.e., digests are empty or missing).
pub fn assert_receipt_missing(model: &ConfigDocumentationModel) -> Result<(), TestFailure> {
    let all_empty = model.receipts.iter().all(|r| r.admitted_config_digest.is_empty());
    if all_empty {
        Ok(())
    } else {
        Err(TestFailure::ReceiptFailed(
            "Model has unexpected receipt binding (expected none)".to_string(),
        ))
    }
}

/// Assert that a receipt digest mismatch is detected (DOC-020).
pub fn assert_receipt_mismatch_detected(
    model: &ConfigDocumentationModel,
    tampered_receipt: &ReceiptBinding,
) -> Result<(), TestFailure> {
    let mut tampered = model.clone();
    tampered.receipts.clear();
    tampered.receipts.push(tampered_receipt.clone());
    let diags = star_toml::docs::validate_docs_integrity(&tampered);
    if diags.iter().any(|d| d.code == "DOC-020") {
        Ok(())
    } else {
        Err(TestFailure::DiagnosticMismatch {
            expected: "DOC-020".to_string(),
            found: diags.iter().map(|d| d.code.to_string()).collect(),
        })
    }
}

// ---------------------------------------------------------------------------
// General diagnostic assertion
// ---------------------------------------------------------------------------

/// Assert that `validate_docs_integrity` produces a diagnostic with `expected_code`.
pub fn assert_doc_diagnostic(
    model: &ConfigDocumentationModel,
    expected_code: &str,
) -> Result<(), TestFailure> {
    let diags = star_toml::docs::validate_docs_integrity(model);
    if diags.iter().any(|d| d.code == expected_code) {
        Ok(())
    } else {
        let found: Vec<String> = diags.iter().map(|d| d.code.to_string()).collect();
        Err(TestFailure::DiagnosticMismatch { expected: expected_code.to_string(), found })
    }
}

/// Assert that `validate_docs_integrity` produces **no** diagnostics.
pub fn assert_no_doc_diagnostics(model: &ConfigDocumentationModel) -> Result<(), TestFailure> {
    let diags = star_toml::docs::validate_docs_integrity(model);
    if diags.is_empty() {
        Ok(())
    } else {
        let codes: Vec<String> = diags.iter().map(|d| d.code.to_string()).collect();
        Err(TestFailure::DocsFailed(format!("Unexpected diagnostics: {codes:?}")))
    }
}

// ---------------------------------------------------------------------------
// Counterexample fixture runner
// ---------------------------------------------------------------------------

/// A single counterexample fixture: TOML input + expected diagnostic code.
pub struct CounterexampleFixture {
    pub label: &'static str,
    pub toml: &'static str,
    pub expected_code: &'static str,
}

/// Run a list of counterexample fixtures.
/// Returns a Vec of (label, result) pairs.
pub fn run_counterexample_fixtures(
    fixtures: &[CounterexampleFixture],
    validate: impl Fn(&str) -> Vec<String>,
) -> Vec<(&'static str, Result<(), TestFailure>)> {
    fixtures
        .iter()
        .map(|f| {
            let codes = validate(f.toml);
            let result = if codes.iter().any(|c| c == f.expected_code) {
                Ok(())
            } else {
                Err(TestFailure::DiagnosticMismatch {
                    expected: f.expected_code.to_string(),
                    found: codes,
                })
            };
            (f.label, result)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// DfCM matrix runner
// ---------------------------------------------------------------------------

/// One axis in the DfCM test matrix.
#[derive(Debug, Clone)]
pub struct DfCMAxis {
    pub name: &'static str,
    pub variants: Vec<(&'static str, bool)>, // (label, should_admit)
}

/// Run all declared axis variants through the provided admission function.
pub fn run_dfcm_matrix(
    axes: &[DfCMAxis],
    admit: impl Fn(&str) -> bool,
) -> Vec<(&'static str, &'static str, bool, bool)> {
    let mut results = Vec::new();
    for axis in axes {
        for &(label, should_admit) in &axis.variants {
            let admitted = admit(label);
            results.push((axis.name, label, should_admit, admitted));
        }
    }
    results
}

/// Assert that all DfCM matrix variants produce the expected admission result.
pub fn assert_dfcm_matrix_passes(
    axes: &[DfCMAxis],
    admit: impl Fn(&str) -> bool,
) -> Result<(), TestFailure> {
    let results = run_dfcm_matrix(axes, admit);
    let failures: Vec<String> = results
        .iter()
        .filter(|(_, _, expected, got)| expected != got)
        .map(|(axis, label, expected, got)| {
            format!("{axis}/{label}: expected admitted={expected}, got {got}")
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(TestFailure::Unexpected(format!("DfCM matrix failures: {failures:?}")))
    }
}
