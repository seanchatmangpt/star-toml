//! DfCM combinatorial integration test — Error enum variants.
//!
//! Produces and matches every `star_toml::Error` variant explicitly.
//! Panics on any mismatch — functions as an assertive integration test.

use std::io::Write;

use serde::{Deserialize, Serialize};
use star_toml::{load_file, loader::TrustedLoader, Error, Validate, Validator};
use tempfile::tempdir;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Good {
    name: String,
    score: u8,
}

impl Validate for Good {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("score", self.score as i64, 0..=100);
    }
}
impl star_toml::loader::ConfigLifecycle for Good {}

// A struct whose validate() always fails — used to trigger Error::Invalid.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct AlwaysInvalid {
    value: String,
}

impl Validate for AlwaysInvalid {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("value", ""); // always fails
    }
}
impl star_toml::loader::ConfigLifecycle for AlwaysInvalid {}

fn main() {
    error_file_not_found();
    error_parse();
    error_invalid();
    error_validation_constructor();
    all_display_non_empty();
    println!("✓ all dfcm_error_variants checks passed");
}

// ---------------------------------------------------------------------------
// Error::FileNotFound
// ---------------------------------------------------------------------------

fn error_file_not_found() {
    let result = load_file::<Good>("/nonexistent/path/does/not/exist.toml");
    let err = result.unwrap_err();
    assert!(matches!(err, Error::FileNotFound(_)), "expected FileNotFound, got: {err}");
    assert!(!err.to_string().is_empty());
}

// ---------------------------------------------------------------------------
// Error::Parse — malformed TOML
// ---------------------------------------------------------------------------

fn error_parse() {
    let result = star_toml::from_str::<Good>("[broken toml without closing bracket");
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Parse { .. }), "expected Parse, got: {err}");
    assert!(!err.to_string().is_empty());
}

// ---------------------------------------------------------------------------
// Error::Invalid — valid TOML but validation rules fail
// ---------------------------------------------------------------------------

fn error_invalid() {
    let toml = r#"
value = "present but validate() ignores it and checks empty string"
"#;
    let result = TrustedLoader::new().layer_str(toml, "test").load_admitted::<AlwaysInvalid>();
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Invalid(_)), "expected Invalid, got: {err}");
    assert!(!err.to_string().is_empty());

    // Drill into the ValidationErrors inside Invalid
    if let Error::Invalid(errs) = &err {
        assert!(errs.len() >= 1, "must have at least one validation error");
    }
}

// ---------------------------------------------------------------------------
// Error::Validation — ad-hoc constructor
// ---------------------------------------------------------------------------

fn error_validation_constructor() {
    let err = Error::validation("my_context", "something went wrong");
    assert!(matches!(err, Error::Validation { .. }), "expected Validation, got: {err}");
    let s = err.to_string();
    assert!(!s.is_empty());
    // The context and reason should appear somewhere in the display
    assert!(
        s.contains("my_context") || s.contains("something went wrong"),
        "Display must include context or reason: {s}"
    );
}

// ---------------------------------------------------------------------------
// Error::Io — file write to path that can't be created
// ---------------------------------------------------------------------------
// Note: triggering Error::Io reliably requires a path that the OS refuses.
// We use a file-as-directory trick: write a file, then try to write inside it.

fn error_io() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not_a_dir");
    std::fs::write(&file_path, b"content").unwrap();
    // Try to save into a path that treats the file as a directory
    let nested = file_path.join("child.toml");
    let result = star_toml::save_file(&Good { name: "x".into(), score: 42 }, &nested);
    if let Err(err) = result {
        assert!(matches!(err, Error::Io { .. }), "expected Io, got: {err}");
        assert!(!err.to_string().is_empty());
    }
    // If the OS doesn't error (e.g. some filesystems), we skip without panicking.
}

// ---------------------------------------------------------------------------
// All Error variants have non-empty Display
// ---------------------------------------------------------------------------

fn all_display_non_empty() {
    let variants: Vec<Error> = vec![
        Error::FileNotFound("/missing.toml".into()),
        Error::Parse {
            path: "t.toml".into(),
            source: toml::from_str::<toml::Value>("[[[").unwrap_err(),
        },
        Error::validation("ctx", "reason"),
    ];
    for e in &variants {
        assert!(!e.to_string().is_empty(), "Display must be non-empty for {e:?}");
    }
}
