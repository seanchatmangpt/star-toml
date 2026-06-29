//! DfCM combinatorial integration test — Validator check methods.
//!
//! Covers every `Validator` check method × pass/fail path, all 4 severity levels,
//! and the full analytics surface (fitness, variant_id, repair_hint, by_section,
//! errors_above). Panics on any violation — functions as an assertive integration test.

use std::path::PathBuf;

use star_toml::{ErrorKind, PathPolicy, Severity, Validate, ValidationErrors, Validator};

fn main() {
    check_non_empty();
    check_range();
    check_one_of();
    check_predicate();
    check_consistent();
    check_semver();
    check_ip_or_domain();
    check_path();
    check_size_format();
    check_path_safe();
    check_profile();
    check_policy();
    severity_levels();
    analytics();
    println!("✓ all dfcm_validation_matrix checks passed");
}

// ---------------------------------------------------------------------------
// check_non_empty
// ---------------------------------------------------------------------------

fn check_non_empty() {
    // pass
    let mut v = Validator::new();
    v.check_non_empty("name", "hello");
    assert!(v.finish().is_ok(), "non-empty string must pass");

    // fail: empty string
    let mut v = Validator::new();
    v.check_non_empty("name", "");
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs.errors()[0].code(), "empty");
    assert_eq!(errs.errors()[0].loc.to_string(), "name");
}

// ---------------------------------------------------------------------------
// check_range
// ---------------------------------------------------------------------------

fn check_range() {
    // pass
    let mut v = Validator::new();
    v.check_range("port", 8080i64, 1024..=65535);
    assert!(v.finish().is_ok());

    // fail: below lower bound
    let mut v = Validator::new();
    v.check_range("port", 80i64, 1024..=65535);
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "out_of_range");

    // fail: above upper bound
    let mut v = Validator::new();
    v.check_range("port", 70000i64, 1024..=65535);
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "out_of_range");
}

// ---------------------------------------------------------------------------
// check_one_of
// ---------------------------------------------------------------------------

fn check_one_of() {
    // pass
    let mut v = Validator::new();
    v.check_one_of("level", "warn", &["info", "warn", "error"]);
    assert!(v.finish().is_ok());

    // fail: not in allowed set
    let mut v = Validator::new();
    v.check_one_of("level", "trace", &["info", "warn", "error"]);
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "not_one_of");
}

// ---------------------------------------------------------------------------
// check_predicate
// ---------------------------------------------------------------------------

fn check_predicate() {
    // pass
    let mut v = Validator::new();
    v.check_predicate("flag", true, "must_be_true", "must be true");
    assert!(v.finish().is_ok());

    // fail
    let mut v = Validator::new();
    v.check_predicate("flag", false, "must_be_true", "must be true");
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "must_be_true");
    assert!(matches!(errs.errors()[0].kind, ErrorKind::Predicate { .. }));
}

// ---------------------------------------------------------------------------
// check_consistent
// ---------------------------------------------------------------------------

fn check_consistent() {
    // pass: ssl_enabled=true and cert_path is set → consistent
    let mut v = Validator::new();
    v.check_consistent("cert_path", &["ssl_enabled"], true, "ssl_cert_required", "cert required");
    assert!(v.finish().is_ok());

    // fail: ssl_enabled=true but cert_path missing → inconsistent
    let mut v = Validator::new();
    v.check_consistent(
        "cert_path",
        &["ssl_enabled"],
        false,
        "ssl_cert_required",
        "cert_path required when ssl_enabled",
    );
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "ssl_cert_required");
    assert!(matches!(errs.errors()[0].kind, ErrorKind::Inconsistent { .. }));
}

// ---------------------------------------------------------------------------
// check_semver
// ---------------------------------------------------------------------------

fn check_semver() {
    // pass
    for valid in &["1.2.3", "0.0.1", "26.6.29"] {
        let mut v = Validator::new();
        v.check_semver("version", valid);
        assert!(v.finish().is_ok(), "expected valid semver: {valid}");
    }

    // fail: missing patch
    let mut v = Validator::new();
    v.check_semver("version", "1.2");
    assert!(v.finish().is_err(), "1.2 must fail semver");

    // fail: leading zero
    let mut v = Validator::new();
    v.check_semver("version", "01.2.3");
    assert!(v.finish().is_err(), "01.2.3 must fail semver");

    // fail: non-numeric component
    let mut v = Validator::new();
    v.check_semver("version", "a.b.c");
    assert!(v.finish().is_err(), "a.b.c must fail semver");
}

// ---------------------------------------------------------------------------
// check_ip_or_domain
// ---------------------------------------------------------------------------

fn check_ip_or_domain() {
    // pass
    for valid in &["127.0.0.1", "::1", "example.com", "sub.domain.org"] {
        let mut v = Validator::new();
        v.check_ip_or_domain("host", valid);
        assert!(v.finish().is_ok(), "expected valid host: {valid}");
    }

    // fail
    for invalid in &["not_a_host!", "http://example.com", ""] {
        let mut v = Validator::new();
        v.check_ip_or_domain("host", invalid);
        assert!(v.finish().is_err(), "expected invalid host: {invalid}");
    }
}

// ---------------------------------------------------------------------------
// check_path
// ---------------------------------------------------------------------------

fn check_path() {
    // pass: relative path, no policy
    let mut v = Validator::new();
    v.check_path("data_dir", "./data", None);
    assert!(v.finish().is_ok());

    // fail: null byte
    let mut v = Validator::new();
    v.check_path("data_dir", "path/with\0null", None);
    assert!(v.finish().is_err(), "null byte must fail");

    // fail: traversal
    let mut v = Validator::new();
    v.check_path("data_dir", "../escape", None);
    assert!(v.finish().is_err(), "path traversal must fail");

    // fail: absolute when relative required
    let mut v = Validator::new();
    v.check_path("data_dir", "/absolute/path", Some(false));
    assert!(v.finish().is_err(), "absolute path must fail when relative required");

    // pass: absolute when absolute required
    let mut v = Validator::new();
    v.check_path("data_dir", "/absolute/path", Some(true));
    assert!(v.finish().is_ok());
}

// ---------------------------------------------------------------------------
// check_size_format
// ---------------------------------------------------------------------------

fn check_size_format() {
    // pass
    for valid in &["512MB", "1gb", "100KB", "4TB", "256B"] {
        let mut v = Validator::new();
        v.check_size_format("max_size", valid);
        assert!(v.finish().is_ok(), "expected valid size: {valid}");
    }

    // fail: no suffix
    let mut v = Validator::new();
    v.check_size_format("max_size", "512");
    assert!(v.finish().is_err(), "no suffix must fail");

    // fail: unknown suffix
    let mut v = Validator::new();
    v.check_size_format("max_size", "512XB");
    assert!(v.finish().is_err(), "XB suffix must fail");
}

// ---------------------------------------------------------------------------
// check_path_safe
// ---------------------------------------------------------------------------

fn check_path_safe() {
    // source_path must be a FILE inside the sandbox so .parent() == sandbox root
    let sandbox_root = PathBuf::from("/tmp/dfcm_sandbox");
    let source_file = sandbox_root.join("config.toml");

    // pass: relative path within sandbox
    let mut v = Validator::new();
    v.set_source_path(source_file.clone());
    v.check_path_safe(
        "output",
        "subdir/file.json",
        &source_file,
        PathPolicy::Sandbox { root: sandbox_root.clone() },
    );
    assert!(v.finish().is_ok(), "sandbox-relative path must pass");

    // fail: traversal — caught before sandbox check
    let mut v = Validator::new();
    v.set_source_path(source_file.clone());
    v.check_path_safe(
        "output",
        "../../etc/passwd",
        &source_file,
        PathPolicy::Sandbox { root: sandbox_root.clone() },
    );
    let errs = v.finish().unwrap_err();
    let code = errs.errors()[0].code();
    assert!(
        code == "sandbox_escape" || code == "path_traversal_detected",
        "expected sandbox_escape or path_traversal_detected, got: {code}"
    );
}

// ---------------------------------------------------------------------------
// check_profile
// ---------------------------------------------------------------------------

fn check_profile() {
    // active == target: check runs and fails
    let mut v = Validator::new();
    v.check_profile(
        "debug_port",
        "dev",
        "dev",
        false,
        "dev_port_required",
        "debug port required in dev",
    );
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.len(), 1, "check must run when profile matches");

    // active != target: check is skipped entirely
    let mut v = Validator::new();
    v.check_profile(
        "debug_port",
        "prod",
        "dev",
        false,
        "dev_port_required",
        "debug port required in dev",
    );
    assert!(v.finish().is_ok(), "check must be skipped when profiles don't match");

    // active == target: check runs and passes
    let mut v = Validator::new();
    v.check_profile(
        "debug_port",
        "dev",
        "dev",
        true,
        "dev_port_required",
        "debug port required in dev",
    );
    assert!(v.finish().is_ok(), "check must pass when condition is true");
}

// ---------------------------------------------------------------------------
// check_policy
// ---------------------------------------------------------------------------

fn check_policy() {
    // pass: closure returns true
    let mut v = Validator::new();
    v.check_policy("max_retries", || true, "retry_policy", "retry must be positive");
    assert!(v.finish().is_ok());

    // fail: closure returns false
    let mut v = Validator::new();
    v.check_policy("max_retries", || false, "retry_policy", "retry must be positive");
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.errors()[0].code(), "retry_policy");
}

// ---------------------------------------------------------------------------
// Severity levels
// ---------------------------------------------------------------------------

fn severity_levels() {
    use star_toml::ErrorKind;

    // Advisory and Warning: has_fatal() must be false
    let mut v = Validator::new();
    v.with_severity(Severity::Advisory, |v| {
        v.error(ErrorKind::Empty, "advisory msg");
    });
    v.with_severity(Severity::Warning, |v| {
        v.error(ErrorKind::Empty, "warning msg");
    });
    let errs = v.finish().unwrap_err();
    assert!(!errs.has_fatal(), "Advisory and Warning must not be fatal");
    assert_eq!(errs.len(), 2);

    // Fatal: has_fatal() must be true
    let mut v = Validator::new();
    v.with_severity(Severity::Fatal, |v| {
        v.error(ErrorKind::Empty, "fatal msg");
    });
    let errs = v.finish().unwrap_err();
    assert!(errs.has_fatal(), "Fatal severity must make has_fatal() true");
}

// ---------------------------------------------------------------------------
// Analytics: fitness, variant_id, repair_hint, by_section, errors_above
// ---------------------------------------------------------------------------

fn analytics() {
    // fitness = 1.0 when no errors
    let mut v = Validator::new();
    v.check_non_empty("a", "x");
    v.check_non_empty("b", "y");
    let errs = v.finish();
    assert!(errs.is_ok());

    // fitness = 0.0 when all checks fail
    let mut v = Validator::new();
    v.check_non_empty("a", "");
    v.check_non_empty("b", "");
    let errs = v.finish().unwrap_err();
    assert_eq!(errs.fitness(), 0.0, "all-fail must have fitness 0.0");

    // fitness ~= 0.5 when half fail
    let mut v = Validator::new();
    v.check_non_empty("a", "x"); // pass
    v.check_non_empty("b", ""); // fail
    let errs = v.finish().unwrap_err();
    assert!(
        (errs.fitness() - 0.5).abs() < 0.01,
        "half-fail must have fitness ~0.5, got {}",
        errs.fitness()
    );

    // variant_id is stable across two identical-failure runs
    let make_errs = || -> ValidationErrors {
        let mut v = Validator::new();
        v.check_non_empty("x", "");
        v.check_range("y", 0i64, 1..=10);
        v.finish().unwrap_err()
    };
    assert_eq!(make_errs().variant_id(), make_errs().variant_id(), "variant_id must be stable");

    // repair_hint is non-empty for built-in error kinds
    let mut v = Validator::new();
    v.check_non_empty("a", "");
    v.check_range("b", 0i64, 1..=10);
    v.check_one_of("c", "x", &["a", "b"]);
    let errs = v.finish().unwrap_err();
    for err in errs.errors() {
        let hint = err.repair_hint();
        assert!(!hint.is_empty(), "repair_hint must be non-empty for {:?}", err.kind);
    }

    // by_section: root errors keyed "(root)", nested errors keyed by top-level field name
    let mut v = Validator::new();
    v.error(ErrorKind::Empty, "root-level error");
    v.field("database", |v| {
        v.check_non_empty("host", "");
    });
    let errs = v.finish().unwrap_err();
    let by_sec = errs.by_section();
    assert!(by_sec.contains_key("(root)"), "root errors must be in \"(root)\" section");
    assert!(by_sec.contains_key("database"), "nested errors must be in \"database\" section");

    // errors_above(Warning) excludes Advisory
    let mut v = Validator::new();
    v.with_severity(Severity::Advisory, |v| {
        v.error(ErrorKind::Empty, "advisory");
    });
    v.with_severity(Severity::Warning, |v| {
        v.error(ErrorKind::Empty, "warning");
    });
    v.with_severity(Severity::Error, |v| {
        v.error(ErrorKind::Empty, "error");
    });
    let errs = v.finish().unwrap_err();
    let above_warning: Vec<_> = errs.errors_above(Severity::Warning).collect();
    assert_eq!(above_warning.len(), 2, "errors_above(Warning) must include Warning and Error");
    let above_error: Vec<_> = errs.errors_above(Severity::Error).collect();
    assert_eq!(above_error.len(), 1, "errors_above(Error) must include only Error");
}
