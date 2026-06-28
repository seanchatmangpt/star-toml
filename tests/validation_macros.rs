//! Validation macro and helper method unit tests.

#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::unnecessary_wraps,
    clippy::items_after_statements,
    unused_imports,
    unused_variables,
    dead_code,
    missing_docs
)]

use star_toml::{schema, Validate, Validator};

// ---------------------------------------------------------------------------
// 1. Structural validation via #[derive(Validate)]
// ---------------------------------------------------------------------------

struct Host {
    name: String,
}

impl Validate for Host {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
    }
}

struct Port {
    value: u16,
}

impl Validate for Port {
    fn validate(&self, v: &mut Validator) {
        v.check_range("value", self.value, 1..=65535);
    }
}

#[derive(Validate)]
struct Database {
    #[validate]
    host: Host,
    #[validate]
    port: Option<Port>,
}

#[derive(Validate)]
struct Config {
    #[validate]
    db: Database,
    #[validate]
    servers: Vec<Database>,
}

#[test]
fn test_derive_validate_complex_option_vec() {
    let cfg = Config {
        db: Database { host: Host { name: "".into() }, port: Some(Port { value: 0 }) },
        servers: vec![
            Database { host: Host { name: "localhost".into() }, port: None },
            Database { host: Host { name: "".into() }, port: Some(Port { value: 8080 }) },
        ],
    };

    let errs = cfg.check().unwrap_err();
    assert_eq!(errs.len(), 3);

    let locs: Vec<String> = errs.errors().iter().map(|e| e.loc.to_string()).collect();
    assert!(locs.contains(&"db.host.name".to_string()));
    assert!(locs.contains(&"db.port.value".to_string()));
    assert!(locs.contains(&"servers[1].host.name".to_string()));

    for err in errs.errors() {
        assert_eq!(
            err.code(),
            if err.loc.to_string().contains("value") { "out_of_range" } else { "empty" }
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Declarative schema! macro tests
// ---------------------------------------------------------------------------

#[test]
fn test_schema_macro_flat_and_nested() {
    // A nested schema with flat fields, lists of constraints, and nested section
    // using both identifier and string literal keys.
    let s = schema! {
        name: non_empty,
        "content-type": one_of("json", "toml"),
        port: [required, range(1024, 65535)],
        ratio: range_f64(0.0, 1.0),
        server: {
            host: non_empty,
            "port": [required, range(1, 65535)],
        }
    };

    // Valid case
    let toml_valid = r#"
name = "my-app"
content-type = "toml"
port = 8080
ratio = 0.5
[server]
host = "127.0.0.1"
port = 443
"#;
    assert!(s.validate_str(toml_valid).is_ok());

    // Invalid case
    let toml_invalid = r#"
name = ""
content-type = "xml"
port = 80
ratio = 1.5
[server]
host = ""
port = 0
"#;
    let errs = s.validate_str(toml_invalid).unwrap_err();
    assert_eq!(errs.len(), 6);

    let locs: Vec<String> = errs.errors().iter().map(|e| e.loc.to_string()).collect();
    assert!(locs.contains(&"name".to_string()));
    assert!(locs.contains(&"content-type".to_string()));
    assert!(locs.contains(&"port".to_string()));
    assert!(locs.contains(&"ratio".to_string()));
    assert!(locs.contains(&"server.host".to_string()));
    assert!(locs.contains(&"server.port".to_string()));
}

// ---------------------------------------------------------------------------
// 3. Profile and policy validator tests
// ---------------------------------------------------------------------------

struct ProfilePolicyConfig {
    profile: String,
    debug_mode: bool,
    prod_url: String,
}

impl Validate for ProfilePolicyConfig {
    fn validate(&self, v: &mut Validator) {
        // profile verification
        v.check_profile(
            "prod_url",
            &self.profile,
            "production",
            !self.prod_url.is_empty(),
            "missing_prod_url",
            "Production URL must be configured when in production profile",
        );

        // policy verification
        v.check_policy(
            "debug_mode",
            || !self.debug_mode || self.profile != "production",
            "unsafe_debug_in_prod",
            "Debug mode is not allowed in production profile",
        );
    }
}

#[test]
fn test_profile_validator() {
    // 1. Target profile matches and condition is true -> passes
    let cfg = ProfilePolicyConfig {
        profile: "production".into(),
        debug_mode: false,
        prod_url: "https://example.com".into(),
    };
    assert!(cfg.check().is_ok());

    // 2. Target profile matches and condition is false -> fails
    let cfg_fail = ProfilePolicyConfig {
        profile: "production".into(),
        debug_mode: false,
        prod_url: "".into(),
    };
    let errs = cfg_fail.check().unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs.errors()[0].loc.to_string(), "prod_url");
    assert_eq!(errs.errors()[0].code(), "missing_prod_url");

    // 3. Target profile does not match and condition is false -> passes (check skipped)
    let cfg_other_profile = ProfilePolicyConfig {
        profile: "development".into(),
        debug_mode: false,
        prod_url: "".into(),
    };
    assert!(cfg_other_profile.check().is_ok());
}

#[test]
fn test_policy_validator() {
    // 1. Policy returns true -> passes
    let cfg = ProfilePolicyConfig {
        profile: "development".into(),
        debug_mode: true,
        prod_url: "".into(),
    };
    assert!(cfg.check().is_ok());

    // 2. Policy returns false -> fails
    let cfg_fail = ProfilePolicyConfig {
        profile: "production".into(),
        debug_mode: true,
        prod_url: "https://example.com".into(),
    };
    let errs = cfg_fail.check().unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs.errors()[0].loc.to_string(), "debug_mode");
    assert_eq!(errs.errors()[0].code(), "unsafe_debug_in_prod");
}
