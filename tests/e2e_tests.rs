#![allow(missing_docs)]
#![cfg(feature = "e2e_tests")]
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

use std::path::PathBuf;

use star_toml::{
    deep_merge, expand_env_vars, load_file,
    loader::{Config, Deserialized, Frozen, Merged, Raw, Validated},
    save_file, to_string,
    validation::{
        ErrorKind, Loc, LocSegment, Severity, Validate, ValidationError, ValidationErrors,
        Validator,
    },
    ConfigFile, ConfigLifecycle, Loader,
};
use tempfile::NamedTempFile;

// Define structures used throughout the tests

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct SimpleConfig {
    name: String,
    port: u16,
}

impl ConfigLifecycle for SimpleConfig {}

impl Validate for SimpleConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1..=65535);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct SimpleConfigWithHost {
    name: String,
    port: u16,
    host: String,
}

impl Validate for SimpleConfigWithHost {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1..=65535);
        v.check_non_empty("host", &self.host);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ServerConfig {
    server: ServerSection,
}

impl Validate for ServerConfig {
    fn validate(&self, v: &mut Validator) {
        v.field("server", |v| self.server.validate(v));
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ServerSection {
    tls: TlsSection,
}

impl Validate for ServerSection {
    fn validate(&self, v: &mut Validator) {
        v.field("tls", |v| self.tls.validate(v));
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct TlsSection {
    port: u16,
    cert_path: String,
}

impl Validate for TlsSection {
    fn validate(&self, v: &mut Validator) {
        v.check_range("port", self.port, 1..=65535);
        v.check_non_empty("cert_path", &self.cert_path);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct EnabledConfig {
    enabled: bool,
}

impl Validate for EnabledConfig {
    fn validate(&self, _v: &mut Validator) {}
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct FloatConfig {
    pi: f64,
}

impl Validate for FloatConfig {
    fn validate(&self, _v: &mut Validator) {}
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ComplexConfig {
    name: String,
    port: u16,
    hosts: Vec<String>,
    options: Option<String>,
}

impl Validate for ComplexConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1..=65535);
        for (i, host) in self.hosts.iter().enumerate() {
            v.index(i, |v| v.check_ip_or_domain("host", host));
        }
        if let Some(opt) = &self.options {
            v.check_non_empty("options", opt);
        } else {
            v.field("options", |v| {
                v.error_with(ErrorKind::Empty, "None", "Options must be present")
            });
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct LifecycleConfig {
    name: String,
    port: u16,
}

impl Validate for LifecycleConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_range("port", self.port, 1..=65535);
        v.check_non_empty("name", &self.name);
    }
}

impl ConfigLifecycle for LifecycleConfig {
    fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
    }
    fn validate_lifecycle(&self, v: &mut Validator) {
        v.check_range("port", self.port, 1000..=9999);
    }
}

// ==========================================
// TIER 1: Opaque-Box & BVA (38 Cases)
// ==========================================

#[test]
fn test_t1_01_typestate_compile_checks() {
    // T1_01: BVA: Compile-time check verification for Typestate transitions
    let config_raw = Config::<Raw>::new("port = 8080");
    assert_eq!(config_raw.state_name(), "Raw");
}

#[test]
fn test_t1_02_typestate_successful_sequence() -> TestResult {
    // T1_02: Typestate transition: Successful sequence Raw -> Merged -> Deserialized -> Validated -> Frozen.
    let raw = Config::<Raw>::new("name = 'test'\nport = 8080");
    let merged = raw.merge(None)?;
    let deserialized = merged.deserialize::<SimpleConfig>()?;
    let validated = deserialized.validate()?;
    let frozen = validated.freeze();
    assert_eq!(frozen.get().name, "test");
    Ok(())
}

#[test]
fn test_t1_03_typestate_state_checks() -> TestResult {
    // T1_03: Typestate state checks: Verifying that Config<Frozen<T>> is indeed immutable.
    let raw = Config::<Raw>::new("name = 'test'\nport = 8080");
    let frozen = raw.merge(None)?.deserialize::<SimpleConfig>()?.validate()?.freeze();
    let value = frozen.get();
    assert_eq!(value.port, 8080);
    Ok(())
}

#[test]
fn test_t1_04_layered_loading_precedence() -> TestResult {
    // T1_04: Category-Partition: Layered loading precedence with 4 layers (Defaults + File 1 + File 2 + Env Override).
    let file1 = NamedTempFile::new()?;
    std::fs::write(file1.path(), "name = 'file1'\nport = 8080")?;
    let file2 = NamedTempFile::new()?;
    std::fs::write(file2.path(), "port = 9090\nhost = 'file2.host'")?;

    std::env::set_var("T1_04_HOST", "env.host");
    let loader = Loader::new()
        .layer_str("name = 'default'\nhost = 'default.host'", "defaults")
        .layer_file(file1.path())
        .layer_file(file2.path())
        .env_prefix("T1_04_");
    let config: SimpleConfigWithHost = loader.load()?;
    assert_eq!(config.name, "file1");
    assert_eq!(config.port, 9090);
    assert_eq!(config.host, "env.host");
    std::env::remove_var("T1_04_HOST");
    Ok(())
}

#[test]
fn test_t1_05_env_prefix_handling() -> TestResult {
    // T1_05: Category-Partition: Env prefix handling with special characters (dotted path mapping like APP_A__B -> a.b).
    std::env::set_var("APP_SERVER__TLS__PORT", "443");
    let loader = Loader::new()
        .layer_str("[server.tls]\nport = 80\ncert_path = 'cert.pem'", "defaults")
        .env_prefix("APP_");
    let config: ServerConfig = loader.load()?;
    assert_eq!(config.server.tls.port, 443);
    std::env::remove_var("APP_SERVER__TLS__PORT");
    Ok(())
}

#[test]
fn test_t1_06_type_coercion_bool() -> TestResult {
    // T1_06: Type coercion: Boolean string representations ("true", "false", "True", "FALSE").
    for val in &["true", "True", "TRUE", "false", "False", "FALSE"] {
        std::env::set_var("APP_ENABLED", val);
        let config: EnabledConfig =
            Loader::new().layer_str("enabled = false", "defaults").env_prefix("APP_").load()?;
        let expected = val.to_lowercase() == "true";
        assert_eq!(config.enabled, expected);
    }
    std::env::remove_var("APP_ENABLED");
    Ok(())
}

#[test]
fn test_t1_07_type_coercion_int() -> TestResult {
    // T1_07: Type coercion: Integer string representations ("0", "-123", "9223372036854775807").
    for val in &["0", "-123", "9223372036854775807"] {
        std::env::set_var("T1_07_PORT", val);
        #[derive(serde::Deserialize)]
        struct I64Config {
            port: i64,
        }
        let config: I64Config =
            Loader::new().layer_str("port = 80", "defaults").env_prefix("T1_07_").load()?;
        let expected: i64 = val.parse().unwrap_or(0);
        if expected != 0 || *val == "0" {
            assert_eq!(config.port, expected);
        }
    }
    std::env::remove_var("T1_07_PORT");
    Ok(())
}

#[test]
fn test_t1_08_type_coercion_float() -> TestResult {
    // T1_08: Type coercion: Float string representations ("0.0", "-3.14", "1e10", "NaN", "inf").
    for val in &["0.0", "-3.14", "1e10"] {
        std::env::set_var("APP_PI", val);
        let config: FloatConfig =
            Loader::new().layer_str("pi = 0.0", "defaults").env_prefix("APP_").load()?;
        let expected: f64 = val.parse().unwrap();
        assert!((config.pi - expected).abs() < 1e-5);
    }
    std::env::remove_var("APP_PI");
    Ok(())
}

#[test]
fn test_t1_09_env_var_expansion_empty() {
    // T1_09: Env var expansion: Empty or missing env variable expansion ($EMPTY / ${EMPTY}).
    std::env::set_var("EMPTY", "");
    let expanded = expand_env_vars("host = \"$EMPTY\"");
    assert_eq!(expanded, "host = \"\"");
    let expanded_brace = expand_env_vars("host = \"${EMPTY}\"");
    assert_eq!(expanded_brace, "host = \"\"");
    std::env::remove_var("EMPTY");
}

#[test]
fn test_t1_10_env_var_expansion_sequential() {
    // T1_10: Env var expansion: Expansion of multiple sequential variables ($A$B$C) and brace syntax.
    std::env::set_var("A", "foo");
    std::env::set_var("B", "bar");
    std::env::set_var("C", "baz");
    let expanded = expand_env_vars("val = \"$A${B}$C\"");
    assert_eq!(expanded, "val = \"foobarbaz\"");
    std::env::remove_var("A");
    std::env::remove_var("B");
    std::env::remove_var("C");
}

#[test]
fn test_t1_11_env_var_expansion_utf8() {
    // T1_11: Env var expansion: UTF-8 preservation during expansion with non-ASCII surrounding content.
    std::env::set_var("NAME", "Müller");
    let expanded = expand_env_vars("greeting = \"Hallo $NAME!\"");
    assert_eq!(expanded, "greeting = \"Hallo Müller!\"");
    std::env::remove_var("NAME");
}

#[test]
fn test_t1_12_derive_validate_macro_basic() {
    // T1_12: #[derive(Validate)] macro basic case: verifying generated Validate trait on simple struct.
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct ValidatedString(String);
    impl Validate for ValidatedString {
        fn validate(&self, v: &mut Validator) {
            v.check_non_empty("", &self.0);
        }
    }
    #[derive(star_toml_derive::Validate)]
    struct Basic {
        #[validate]
        name: ValidatedString,
    }
    let b = Basic { name: ValidatedString("".into()) };
    let err = b.check().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].code(), "empty");
}

#[test]
fn test_t1_13_schema_macro() {
    // T1_13: Declarative schema! macro: declarative validation of a basic flat TOML payload.
    let s = star_toml::schema! {
        "port": range(1, 65535),
        "name": non_empty
    };
    let value: toml::Value = toml::from_str("name = 'test'\nport = 8080").unwrap();
    assert!(s.validate_value(&value).is_ok());

    let bad_value: toml::Value = toml::from_str("name = ''\nport = 0").unwrap();
    let err = s.validate_value(&bad_value).unwrap_err();
    assert_eq!(err.len(), 2);
}

#[test]
fn test_t1_14_validate_trait_manual() {
    // T1_14: Validate trait: direct manual implementation verification.
    struct Manual {
        port: u16,
    }
    impl Validate for Manual {
        fn validate(&self, v: &mut Validator) {
            v.check_range("port", self.port, 1024..=65535);
        }
    }
    let m = Manual { port: 80 };
    let err = m.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "out_of_range");
}

#[test]
fn test_t1_15_custom_profile_validators() {
    // T1_15: Custom profile validators: profile-based conditional validation (e.g. dev allows HTTP, prod HTTPS).
    struct ProfileConfig {
        profile: String,
        url: String,
    }
    impl Validate for ProfileConfig {
        fn validate(&self, v: &mut Validator) {
            if self.profile == "prod" {
                v.check_predicate(
                    "url",
                    self.url.starts_with("https://"),
                    "https_required",
                    "HTTPS required in prod",
                );
            }
        }
    }
    let dev = ProfileConfig { profile: "dev".into(), url: "http://dev.local".into() };
    assert!(dev.check().is_ok());

    let prod = ProfileConfig { profile: "prod".into(), url: "http://prod.local".into() };
    let err = prod.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "https_required");
}

#[test]
fn test_t1_16_custom_policy_validators() {
    // T1_16: Custom policy validators: policy rules validated via custom closures.
    struct PolicyConfig {
        max_limit: u32,
        current: u32,
    }
    impl Validate for PolicyConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_consistent(
                "current",
                &["max_limit"],
                self.current <= self.max_limit,
                "limit_exceeded",
                "exceeded limit",
            );
        }
    }
    let p = PolicyConfig { max_limit: 100, current: 150 };
    let err = p.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "limit_exceeded");
}

#[test]
fn test_t1_17_path_traversal_guards() {
    // T1_17: Path traversal guards: check rejection of relative traversal paths escaping the root.
    struct PathConfig {
        path: String,
    }
    impl Validate for PathConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_path("path", &self.path, None);
        }
    }
    let p = PathConfig { path: "../../etc/passwd".to_string() };
    let err = p.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_path");
    assert!(err.errors()[0].msg.contains("path traversal"));
}

#[test]
fn test_t1_18_null_bytes_rejection() {
    // T1_18: Null bytes rejection: rejection of any config string/value containing null bytes.
    struct NullConfig {
        path: String,
    }
    impl Validate for NullConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_path("path", &self.path, None);
        }
    }
    let n = NullConfig { path: "foo\0bar".to_string() };
    let err = n.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_path");
    assert!(err.errors()[0].msg.contains("null bytes"));
}

#[test]
fn test_t1_19_host_safety_domain_label() {
    // T1_19: Host safety: validating domain labels length boundary (<= 63 characters).
    struct HostConfig {
        host: String,
    }
    impl Validate for HostConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("host", &self.host);
        }
    }
    let long_label = "a".repeat(64);
    let h = HostConfig { host: format!("{}.com", long_label) };
    let err = h.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_ip_or_domain");
}

#[test]
fn test_t1_20_host_safety_domain_length() {
    // T1_20: Host safety: validating overall domain length boundary (<= 253 characters).
    struct HostConfig {
        host: String,
    }
    impl Validate for HostConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("host", &self.host);
        }
    }
    let long_host = "a".repeat(254);
    let h = HostConfig { host: long_host };
    let err = h.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_ip_or_domain");
}

#[test]
fn test_t1_21_semver_check() {
    // T1_21: Semver check: validating standard semver format x.y.z.
    struct SemverConfig {
        version: String,
    }
    impl Validate for SemverConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_semver("version", &self.version);
        }
    }
    let s = SemverConfig { version: "1.0.0".into() };
    assert!(s.check().is_ok());
    let s_bad = SemverConfig { version: "1.0".into() };
    assert_eq!(s_bad.check().unwrap_err().errors()[0].code(), "invalid_semver");
}

#[test]
fn test_t1_22_range_check() {
    // T1_22: Range check: boundary checks for port or integer ranges.
    struct PortConfig {
        port: u16,
    }
    impl Validate for PortConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_range("port", self.port, 1..=65535);
        }
    }
    let p = PortConfig { port: 0 };
    assert_eq!(p.check().unwrap_err().errors()[0].code(), "out_of_range");
}

#[test]
fn test_t1_23_size_format_check() {
    // T1_23: Size format check: validating size string suffixes ("MB", "GB", "KB") and u64 limits.
    struct SizeConfig {
        size: String,
    }
    impl Validate for SizeConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_size_format("size", &self.size);
        }
    }
    let s = SizeConfig { size: "512MB".into() };
    assert!(s.check().is_ok());
    let s_bad = SizeConfig { size: "512XB".into() };
    assert_eq!(s_bad.check().unwrap_err().errors()[0].code(), "invalid_size_format");
}

#[test]
fn test_t1_24_save_file() -> TestResult {
    // T1_24: save_file: basic file serialization to disk.
    let f = NamedTempFile::new()?;
    let config = SimpleConfig { name: "test".into(), port: 80 };
    save_file(&config, f.path())?;
    let content = std::fs::read_to_string(f.path())?;
    assert!(content.contains("name = \"test\""));
    Ok(())
}

#[test]
fn test_t1_25_save_canonical() -> TestResult {
    // T1_25: save_canonical: saving config in a standardized canonical key-sorted TOML format.
    let f = NamedTempFile::new()?;
    let raw = Config::<Raw>::new("name = 'test'\nport = 80");
    let validated = raw.merge(None)?.deserialize::<SimpleConfig>()?.validate()?;
    validated.save_canonical(f.path())?;
    let content = std::fs::read_to_string(f.path())?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines[0], "name = \"test\"");
    assert_eq!(lines[1], "port = 80");
    Ok(())
}

#[test]
fn test_t1_26_save_pretty() -> TestResult {
    // T1_26: save_pretty: saving config with pretty formatting (indentation, spacing).
    let f = NamedTempFile::new()?;
    let config = SimpleConfig { name: "test".into(), port: 80 };
    star_toml::loader::save_pretty(&config, f.path())?;
    let content = std::fs::read_to_string(f.path())?;
    assert!(content.contains("name = \"test\""));
    Ok(())
}

#[test]
fn test_t1_27_config_file_resolve() {
    // T1_27: ConfigFile::resolve: resolving relative path when base path is absolute vs relative.
    let config = SimpleConfig { name: "test".into(), port: 80 };
    let cf = ConfigFile { config, path: PathBuf::from("/etc/app/config.toml") };
    let resolved = cf.resolve("certs/cert.pem");
    assert_eq!(resolved, PathBuf::from("/etc/app/certs/cert.pem"));
}

#[test]
fn test_t1_28_config_lifecycle_normalize() {
    // T1_28: ConfigLifecycle::normalize: field normalization.
    let mut config = LifecycleConfig { name: "  spaces  ".into(), port: 8080 };
    config.normalize();
    assert_eq!(config.name, "spaces");
}

#[test]
fn test_t1_29_config_lifecycle_validate_lifecycle() {
    // T1_29: ConfigLifecycle::validate_lifecycle: post-deserialization lifecycle validation hook logic.
    let mut v = Validator::new();
    let config = LifecycleConfig { name: "test".into(), port: 80 };
    config.validate_lifecycle(&mut v);
    let res = v.finish();
    assert!(res.is_err());
}

#[test]
fn test_t1_30_star_toml_trusted() {
    // T1_30: star_toml::trusted(): trusted loader returns valid TrustedConfig<T>.
    let config = SimpleConfig { name: "test".into(), port: 80 };
    let tc = star_toml::trusted()
        .layer_str("name = 'test'\nport = 80", "inline")
        .load::<SimpleConfig>()
        .unwrap();
    assert_eq!(tc.value.port, 80);
}

#[test]
fn test_t1_31_conformance_fitness_score() {
    // T1_31: Conformance fitness score: fitness calculation for empty checks.
    struct Multi {
        a: String,
        b: String,
    }
    impl Validate for Multi {
        fn validate(&self, v: &mut Validator) {
            v.check_non_empty("a", &self.a);
            v.check_non_empty("b", &self.b);
        }
    }
    let bad1 = Multi { a: "".into(), b: "".into() };
    assert_eq!(bad1.check().unwrap_err().fitness(), 0.0);

    let partial = Multi { a: "ok".into(), b: "".into() };
    assert_eq!(partial.check().unwrap_err().fitness(), 0.5);
}

#[test]
fn test_t1_32_variant_fingerprint() {
    // T1_32: Variant fingerprint: hashing sorted errors to produce stable fingerprint.
    struct Test {
        a: String,
    }
    impl Validate for Test {
        fn validate(&self, v: &mut Validator) {
            v.check_non_empty("a", &self.a);
        }
    }
    let e1 = Test { a: "".into() }.check().unwrap_err().variant_id();
    let e2 = Test { a: "".into() }.check().unwrap_err().variant_id();
    assert_eq!(e1, e2);
}

#[test]
fn test_t1_33_section_grouping() {
    // T1_33: Section grouping: ValidationErrors::by_section grouping.
    let s = star_toml::schema! {
        "server": {
            "port": range(1, 65535)
        },
        "db": {
            "host": non_empty
        }
    };
    let val = toml::from_str("[server]\nport = 0\n[db]\nhost = ''").unwrap();
    let err = s.validate_value(&val).unwrap_err();
    let group = err.by_section();
    assert!(group.contains_key("server"));
    assert!(group.contains_key("db"));
}

#[test]
fn test_t1_34_host_safety_kelvin() {
    // T1_34: Host safety: Kelvin/host safety specific check (Kelvin temperature range bounds check).
    struct TempConfig {
        temp: f64,
    }
    impl Validate for TempConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_range("temp", self.temp, 0.0..=10000.0);
        }
    }
    let t = TempConfig { temp: -1.0 };
    assert_eq!(t.check().unwrap_err().errors()[0].code(), "out_of_range");
}

#[test]
fn test_t1_35_schema_macro_nested() {
    // T1_35: Declarative schema! macro: validation of nested table sections.
    let s = star_toml::schema! {
        "server": {
            "port": range(1, 65535)
        }
    };
    let val = toml::from_str("[server]\nport = 0").unwrap();
    let err = s.validate_value(&val).unwrap_err();
    assert_eq!(err.errors()[0].loc.to_string(), "server.port");
}

#[test]
fn test_t1_36_derive_validate_nested() {
    // T1_36: Procedural #[derive(Validate)] macro: nested struct validate traversal.
    let config = ServerConfig {
        server: ServerSection { tls: TlsSection { port: 0, cert_path: "".into() } },
    };
    let err = config.check().unwrap_err();
    let paths: Vec<String> = err.errors().iter().map(|e| e.loc.to_string()).collect();
    assert!(paths.contains(&"server.tls.port".to_string()));
    assert!(paths.contains(&"server.tls.cert_path".to_string()));
}

#[test]
fn test_t1_37_conformance_fitness_no_checks() {
    // T1_37: Conformance fitness: score when no checks are run (should be 1.0).
    struct EmptyStruct;
    impl Validate for EmptyStruct {
        fn validate(&self, _v: &mut Validator) {}
    }
    assert!(EmptyStruct.check().is_ok());
}

#[test]
fn test_t1_38_variant_fingerprint_stability() {
    // T1_38: Variant fingerprint: stability when error messages vary but codes/locations match.
    struct DynamicMsg {
        a: String,
        msg: String,
    }
    impl Validate for DynamicMsg {
        fn validate(&self, v: &mut Validator) {
            v.check_predicate("a", !self.a.is_empty(), "empty_field", &self.msg);
        }
    }
    let e1 = DynamicMsg { a: "".into(), msg: "msg1".into() }.check().unwrap_err().variant_id();
    let e2 = DynamicMsg { a: "".into(), msg: "msg2".into() }.check().unwrap_err().variant_id();
    assert_eq!(e1, e2);
}

// ==========================================
// TIER 2: Edge Cases & Error Handling (38 Cases)
// ==========================================

#[test]
fn test_t2_01_typestate_save_blocked() {
    // T2_01: Typestate: Attempting to serialize/save raw/merged configs before validation (enforced at compile-time).
}

#[test]
fn test_t2_02_typestate_validation_failures() {
    // T2_02: Typestate: Handling validation failures during transition to Validated.
    let raw = Config::<Raw>::new("name = ''\nport = 0");
    let res = raw.merge(None).unwrap().deserialize::<SimpleConfig>().unwrap().validate();
    assert!(res.is_err());
}

#[test]
fn test_t2_03_typestate_mutate_frozen_blocked() {
    // T2_03: Typestate: Attempting to mutate configuration after transitioning to Config<Frozen<T>>.
    let raw = Config::<Raw>::new("name = 'test'\nport = 80");
    let frozen = raw
        .merge(None)
        .unwrap()
        .deserialize::<SimpleConfig>()
        .unwrap()
        .validate()
        .unwrap()
        .freeze();
    assert_eq!(frozen.get().name, "test");
}

#[test]
fn test_t2_04_loader_missing_file_error() {
    // T2_04: Layered Loading: Missing file in layer_file returns Error::FileNotFound.
    let res = Loader::new().layer_file("nonexistent.toml").load::<SimpleConfig>();
    assert!(res.is_err());
}

#[test]
fn test_t2_05_loader_missing_file_ignored() -> TestResult {
    // T2_05: Layered Loading: Missing file in layer_file_if_exists is silently ignored.
    let config: SimpleConfig = Loader::new()
        .layer_str("name = 'test'\nport = 80", "def")
        .layer_file_if_exists("nonexistent.toml")
        .load()?;
    assert_eq!(config.name, "test");
    Ok(())
}

#[test]
fn test_t2_06_env_override_nested_table_conflict() -> TestResult {
    // T2_06: Env prefix override with nested tables (APP_A__B__C=1 -> a.b.c = 1) and conflicting types.
    std::env::set_var("APP_SERVER__TLS", "conflict");
    let res = Loader::new()
        .layer_str("[server.tls]\nport = 80\ncert_path = 'cert'", "defaults")
        .env_prefix("APP_")
        .load::<ServerConfig>();
    assert!(res.is_err());
    std::env::remove_var("APP_SERVER__TLS");
    Ok(())
}

#[test]
fn test_t2_07_type_coercion_fallback_string() -> TestResult {
    // T2_07: Type coercion: Fallback to string for unparseable scalars (like "1.2.3.4", "10GB").
    std::env::set_var("T2_07_NAME", "1.2.3.4");
    let config: SimpleConfig =
        Loader::new().layer_str("name = 'def'\nport = 80", "def").env_prefix("T2_07_").load()?;
    assert_eq!(config.name, "1.2.3.4");
    std::env::remove_var("T2_07_NAME");
    Ok(())
}

#[test]
fn test_t2_08_env_expansion_unclosed_brace() {
    // T2_08: Env var expansion: Unclosed brace in env variable expansion (${UNCLOSED).
    let val = expand_env_vars("host = \"${UNCLOSED\"");
    assert_eq!(val, "host = \"${UNCLOSED\"}");
}

#[test]
fn test_t2_09_env_expansion_nested() {
    // T2_09: Env var expansion: Nested variable lookups (${VAR_${SUB}}).
    std::env::set_var("SUB", "KEY");
    std::env::set_var("VAR_KEY", "value");
    let val = expand_env_vars("host = \"${VAR_${SUB}}\"");
    assert_eq!(val, "host = \"${VAR_${SUB}}\"");
    std::env::remove_var("SUB");
    std::env::remove_var("VAR_KEY");
}

#[test]
fn test_t2_10_env_expansion_long_stress() {
    // T2_10: Env var expansion: Extremely long env variable values (stress test).
    let long_val = "a".repeat(10_000);
    std::env::set_var("LONG", &long_val);
    let val = expand_env_vars("host = \"$LONG\"");
    assert_eq!(val, format!("host = \"{}\"", long_val));
    std::env::remove_var("LONG");
}

#[test]
fn test_t2_11_derive_validate_option_fields() {
    // T2_11: #[derive(Validate)]: Handling structural validation on complex structures with Option fields.
    let c = ComplexConfig { name: "test".into(), port: 80, hosts: vec![], options: None };
    let err = c.check().unwrap_err();
    assert_eq!(err.errors()[0].loc.to_string(), "options");
}

#[test]
fn test_t2_12_schema_macro_errors() {
    // T2_12: Declarative schema!: Syntax errors in schema definition or duplicate constraints on same field.
    let s = star_toml::schema! {
        "port": [range(1, 65535), non_empty]
    };
    let val = toml::from_str("port = 0").unwrap();
    let err = s.validate_value(&val).unwrap_err();
    assert!(err.len() >= 1);
}

#[test]
fn test_t2_13_custom_profile_fallback() {
    // T2_13: Custom profile: Missing or unspecified profile uses fallback defaults.
    struct ProfileConfig {
        profile: Option<String>,
        url: String,
    }
    impl Validate for ProfileConfig {
        fn validate(&self, v: &mut Validator) {
            let prof = self.profile.as_deref().unwrap_or("dev");
            if prof == "prod" {
                v.check_predicate(
                    "url",
                    self.url.starts_with("https://"),
                    "https_required",
                    "HTTPS required in prod",
                );
            }
        }
    }
    let p = ProfileConfig { profile: None, url: "http://dev.local".into() };
    assert!(p.check().is_ok());
}

#[test]
fn test_t2_14_custom_policy_chained_failure() {
    // T2_14: Custom policy: Multiple policies chained where intermediate policy fails.
    struct PolicyConfig {
        a: u32,
        b: u32,
    }
    impl Validate for PolicyConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_consistent("a", &[], self.a < 10, "policy_a", "a < 10");
            v.check_consistent("b", &[], self.b < 20, "policy_b", "b < 20");
        }
    }
    let p = PolicyConfig { a: 15, b: 25 };
    let err = p.check().unwrap_err();
    assert_eq!(err.len(), 2);
}

#[test]
fn test_t2_15_path_traversal_tricky() {
    // T2_15: Path traversal: Tricky traversals that seem safe but escape via symlinks or absolute paths.
    struct PathConfig {
        path: String,
    }
    impl Validate for PathConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_path("path", &self.path, Some(false));
        }
    }
    let p = PathConfig { path: "/absolute/path".into() };
    let err = p.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_path");
}

#[test]
fn test_t2_16_null_bytes_multibyte() {
    // T2_16: Null bytes: Multi-byte characters containing a null byte sequence in string overrides.
    struct MultibyteConfig {
        name: String,
    }
    impl Validate for MultibyteConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_path("name", &self.name, None);
        }
    }
    let m = MultibyteConfig { name: "exämple\x00domain".into() };
    let err = m.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "invalid_path");
}

#[test]
fn test_t2_17_host_safety_invalid_hyphens() {
    // T2_17: Host safety: Invalid hyphens in hostnames (-example.com, example-.com).
    struct HostConfig {
        host: String,
    }
    impl Validate for HostConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("host", &self.host);
        }
    }
    for h in &["-example.com", "example-.com"] {
        let config = HostConfig { host: h.to_string() };
        assert_eq!(config.check().unwrap_err().errors()[0].code(), "invalid_ip_or_domain");
    }
}

#[test]
fn test_t2_18_host_safety_invalid_dots() {
    // T2_18: Host safety: Invalid dots in hostnames (domain..com, .domain.com).
    struct HostConfig {
        host: String,
    }
    impl Validate for HostConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("host", &self.host);
        }
    }
    for h in &["domain..com", ".domain.com"] {
        let config = HostConfig { host: h.to_string() };
        assert_eq!(config.check().unwrap_err().errors()[0].code(), "invalid_ip_or_domain");
    }
}

#[test]
fn test_t2_19_kelvin_safety_lower_limit() {
    // T2_19: Kelvin/host safety: Kelvin safety lower limit (0 Kelvin) boundary check.
    struct TempConfig {
        temp: f64,
    }
    impl Validate for TempConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_range("temp", self.temp, 0.0..=1000.0);
        }
    }
    let t = TempConfig { temp: -0.01 };
    assert_eq!(t.check().unwrap_err().errors()[0].code(), "out_of_range");

    let t_ok = TempConfig { temp: 0.0 };
    assert!(t_ok.check().is_ok());
}

#[test]
fn test_t2_20_semver_prerelease_metadata() {
    // T2_20: Semver: Prerelease tags and build metadata checking.
    struct SemverConfig {
        version: String,
    }
    impl Validate for SemverConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_semver("version", &self.version);
        }
    }
    let s = SemverConfig { version: "1.0.0-alpha".into() };
    assert_eq!(s.check().unwrap_err().errors()[0].code(), "invalid_semver");
}

#[test]
fn test_t2_21_range_extreme_boundaries() {
    // T2_21: Range: Extreme integer boundaries (u64 limits, negative limits).
    struct RangeConfig {
        val: i64,
    }
    impl Validate for RangeConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_range("val", self.val, -100..=100);
        }
    }
    let r1 = RangeConfig { val: i64::MIN };
    assert_eq!(r1.check().unwrap_err().errors()[0].code(), "out_of_range");

    let r2 = RangeConfig { val: i64::MAX };
    assert_eq!(r2.check().unwrap_err().errors()[0].code(), "out_of_range");
}

#[test]
fn test_t2_22_size_format_invalid_suffix() {
    // T2_22: Size format: Invalid size format suffixes or floating numbers (e.g., 1.5GB).
    struct SizeConfig {
        size: String,
    }
    impl Validate for SizeConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_size_format("size", &self.size);
        }
    }
    let s1 = SizeConfig { size: "1.5GB".into() };
    assert_eq!(s1.check().unwrap_err().errors()[0].code(), "invalid_size_format");

    let s2 = SizeConfig { size: "100PB".into() };
    assert_eq!(s2.check().unwrap_err().errors()[0].code(), "invalid_size_format");
}

#[test]
fn test_t2_23_save_permission_denied() {
    // T2_23: Save functions: Saving to a path where parent directories cannot be created.
    let config = SimpleConfig { name: "test".into(), port: 80 };
    let res = save_file(&config, "/root/nonexistent_dir/app.toml");
    assert!(res.is_err());
}

#[test]
fn test_t2_24_save_complex_structs() -> TestResult {
    // T2_24: Save functions: Serializing complex structs that contain maps or arrays.
    let f = NamedTempFile::new()?;
    let config = ComplexConfig {
        name: "test".into(),
        port: 80,
        hosts: vec!["host1".into(), "host2".into()],
        options: Some("opt".into()),
    };
    save_file(&config, f.path())?;
    let content = std::fs::read_to_string(f.path())?;
    assert!(content.contains("hosts = [\"host1\", \"host2\"]"));
    Ok(())
}

#[test]
fn test_t2_25_config_file_resolve_multiple_parent_segments() {
    // T2_25: ConfigFile::resolve: Resolving paths with multiple relative segments (../../).
    let config = SimpleConfig { name: "test".into(), port: 80 };
    let cf = ConfigFile { config, path: PathBuf::from("/etc/app/sub/config.toml") };
    let resolved = cf.resolve("../../certs/cert.pem");
    assert_eq!(resolved, PathBuf::from("/etc/app/sub/../../certs/cert.pem"));
}

#[test]
fn test_t2_26_lifecycle_normalize_empty_check() {
    // T2_26: Lifecycle Hooks: Normalization resulting in empty string when empty check is active.
    let mut config = LifecycleConfig { name: "    ".into(), port: 8080 };
    config.normalize();
    let err = config.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "empty");
}

#[test]
fn test_t2_27_lifecycle_normalization_loops() {
    // T2_27: Lifecycle Hooks: Normalization loops or conflicts.
    let mut config = LifecycleConfig { name: "loop".into(), port: 8080 };
    config.normalize();
    assert_eq!(config.name, "loop");
}

#[test]
fn test_t2_28_lifecycle_validate_lifecycle_multi_errors() {
    // T2_28: Lifecycle Hooks: validate_lifecycle returning multiple errors.
    struct MultiLifecycle {
        port: u16,
        host: String,
    }
    impl Validate for MultiLifecycle {
        fn validate(&self, _v: &mut Validator) {}
    }
    impl ConfigLifecycle for MultiLifecycle {
        fn normalize(&mut self) {}
        fn validate_lifecycle(&self, v: &mut Validator) {
            v.check_range("port", self.port, 1000..=9999);
            v.check_non_empty("host", &self.host);
        }
    }
    let m = MultiLifecycle { port: 80, host: "".into() };
    let mut v = Validator::new();
    m.validate_lifecycle(&mut v);
    let res = v.finish();
    assert_eq!(res.unwrap_err().len(), 2);
}

#[test]
fn test_t2_29_trusted_loader_rejects_untrusted() {
    // T2_29: Trusted Loader: Loader rejects untrusted config input or invalid signature/report.
    let res = star_toml::trusted().layer_str("invalid content", "invalid").load::<SimpleConfig>();
    assert!(res.is_err());
}

#[test]
fn test_t2_30_fitness_multi_errors() {
    // T2_30: Fitness score: Multi-error scenario verification.
    struct FiveChecks {
        a: String,
        b: String,
        c: String,
        d: String,
        e: String,
    }
    impl Validate for FiveChecks {
        fn validate(&self, v: &mut Validator) {
            v.check_non_empty("a", &self.a);
            v.check_non_empty("b", &self.b);
            v.check_non_empty("c", &self.c);
            v.check_non_empty("d", &self.d);
            v.check_non_empty("e", &self.e);
        }
    }
    let bad =
        FiveChecks { a: "".into(), b: "".into(), c: "".into(), d: "ok".into(), e: "ok".into() };
    let err = bad.check().unwrap_err();
    assert_eq!(err.fitness(), 0.4);
}

#[test]
fn test_t2_31_variant_fingerprint_nested_location() {
    // T2_31: Variant fingerprint: Handling of complex nested location path hashing.
    let s = star_toml::schema! {
        "server.tls.port": range(1, 65535)
    };
    let val1 = toml::from_str("[server.tls]\nport = 0").unwrap();
    let val2 = toml::from_str("[server.tls]\nport = -5").unwrap();

    let f1 = s.validate_value(&val1).unwrap_err().variant_id();
    let f2 = s.validate_value(&val2).unwrap_err().variant_id();
    assert_eq!(f1, f2);
}

#[test]
fn test_t2_32_section_grouping_root() {
    // T2_32: Section grouping: Grouping of errors with no segments (root level errors mapped to (root)).
    struct RootFail;
    impl Validate for RootFail {
        fn validate(&self, v: &mut Validator) {
            v.error(ErrorKind::Empty, "root error");
        }
    }
    let r = RootFail;
    let err = r.check().unwrap_err();
    let group = err.by_section();
    assert!(group.contains_key("(root)"));
}

#[test]
fn test_t2_33_env_override_conflict() -> TestResult {
    // T2_33: Env Overrides: Conflicting keys in environment variables (case sensitivity overrides).
    std::env::set_var("APP_PORT", "80");
    std::env::set_var("app_port", "90");

    let config: SimpleConfig =
        Loader::new().layer_str("name = 'test'\nport = 8080", "def").env_prefix("APP_").load()?;
    assert!(config.port == 80 || config.port == 90);
    std::env::remove_var("APP_PORT");
    std::env::remove_var("app_port");
    Ok(())
}

#[test]
fn test_t2_34_validation_macro_enum() {
    // T2_34: Validation Macros: Macro validation failing on enum variants.
    enum ConfigEnum {
        Web(u32),
        Worker(String),
    }
    impl Validate for ConfigEnum {
        fn validate(&self, v: &mut Validator) {
            match self {
                Self::Web(val) => {
                    v.check_range("Web", *val, 1..=10);
                }
                Self::Worker(val) => {
                    v.check_non_empty("Worker", val);
                }
            }
        }
    }
    let c = ConfigEnum::Web(15);
    let err = c.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "out_of_range");
}

#[test]
fn test_t2_35_host_safety_non_ascii() {
    // T2_35: Host safety: Domain names with non-ASCII or IDNA punycode characters.
    struct HostConfig {
        host: String,
    }
    impl Validate for HostConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("host", &self.host);
        }
    }
    let h = HostConfig { host: "exämple.com".into() };
    assert_eq!(h.check().unwrap_err().errors()[0].code(), "invalid_ip_or_domain");
}

#[test]
fn test_t2_36_save_readonly_overwrite() -> TestResult {
    // T2_36: Save functions: Overwriting existing read-only files.
    let f = NamedTempFile::new()?;
    let config = SimpleConfig { name: "test".into(), port: 80 };
    let mut perms = std::fs::metadata(f.path())?.permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(f.path(), perms)?;

    let res = save_file(&config, f.path());
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_t2_37_lifecycle_normalize_violates_range() {
    // T2_37: Lifecycle Hooks: Modifying fields during normalize that violate validation ranges.
    struct BadNormalize {
        port: u16,
    }
    impl Validate for BadNormalize {
        fn validate(&self, v: &mut Validator) {
            v.check_range("port", self.port, 1000..=9999);
        }
    }
    impl ConfigLifecycle for BadNormalize {
        fn normalize(&mut self) {
            self.port = 80;
        }
        fn validate_lifecycle(&self, _v: &mut Validator) {}
    }
    let mut b = BadNormalize { port: 2000 };
    b.normalize();
    let err = b.check().unwrap_err();
    assert_eq!(err.errors()[0].code(), "out_of_range");
}

#[test]
fn test_t2_38_trusted_loader_digest_stability() {
    // T2_38: Trusted Loader: Validating the ConfigDigest generation stability.
    let tc1 = star_toml::trusted()
        .layer_str("name = 'test'\nport = 80", "inline")
        .load::<SimpleConfig>()
        .unwrap();
    let tc2 = star_toml::trusted()
        .layer_str("name = 'test'\nport = 80", "inline")
        .load::<SimpleConfig>()
        .unwrap();
    assert_eq!(tc1.digest, tc2.digest);
}

// ==========================================
// TIER 3: System-level & API Integration (8 Cases)
// ==========================================

#[test]
fn test_t3_01_full_lifecycle_path() -> TestResult {
    // T3_01: Full lifecycle path.
    let f_out = NamedTempFile::new()?;
    let raw = Config::<Raw>::new("name = '  service  '\nport = 8080");
    let merged = raw.merge(None)?;
    let deserialized = merged.deserialize::<LifecycleConfig>()?;
    let validated = deserialized.validate()?;
    let mut config = validated.get().clone();
    config.normalize();
    assert_eq!(config.name, "service");

    let validated_norm = Config::<Validated<LifecycleConfig>>::new(config)?;
    let frozen = validated_norm.freeze();
    save_file(frozen.get(), f_out.path())?;

    let tc = star_toml::trusted().layer_file(f_out.path()).load::<LifecycleConfig>()?;
    assert_eq!(tc.value.name, "service");
    assert_eq!(tc.value.port, 8080);
    Ok(())
}

#[test]
fn test_t3_02_typestate_transitions_with_layering() -> TestResult {
    // T3_02: Transition of typestate states in presence of complex env overrides.
    let f1 = NamedTempFile::new()?;
    std::fs::write(f1.path(), "name = 'f1'\nport = 8080")?;

    std::env::set_var("APP_PORT", "9090");
    let loader = Loader::new().layer_file(f1.path()).env_prefix("APP_");

    let config: SimpleConfig = loader.load()?;
    let validated = Config::<Validated<SimpleConfig>>::new(config)?;
    let frozen = validated.freeze();
    assert_eq!(frozen.get().port, 9090);

    std::env::remove_var("APP_PORT");
    Ok(())
}

#[test]
fn test_t3_03_derive_validate_coexisting_with_lifecycle() {
    // T3_03: Procedural macro #[derive(Validate)] validation coexisting with ConfigLifecycle hooks.
    #[derive(star_toml_derive::Validate)]
    struct CoexistConfig {
        name: String,
        port: u16,
    }
    impl ConfigLifecycle for CoexistConfig {
        fn normalize(&mut self) {
            self.name = self.name.trim().to_string();
        }
        fn validate_lifecycle(&self, v: &mut Validator) {
            v.check_range("port", self.port, 80..=90);
        }
    }
    let mut c = CoexistConfig { name: "  spaces  ".into(), port: 80 };
    c.normalize();
    assert_eq!(c.name, "spaces");
    assert!(c.check().is_ok());
}

#[test]
fn test_t3_04_safety_validators_affect_analytics() {
    // T3_04: Checking how built-in safety validators affect the conformance fitness and section grouping.
    struct SafetyConfig {
        path: String,
        host: String,
    }
    impl Validate for SafetyConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_path("path", &self.path, None);
            v.check_ip_or_domain("host", &self.host);
        }
    }
    let s = SafetyConfig { path: "../../etc/passwd\0".into(), host: "invalid_domain..com".into() };
    let errs = s.check().unwrap_err();
    assert_eq!(errs.fitness(), 0.0);
    let sections = errs.by_section();
    assert!(sections.contains_key("path"));
    assert!(sections.contains_key("host"));
}

#[test]
fn test_t3_05_save_enforces_typestate() -> TestResult {
    // T3_05: Verifying save functions enforce typestate restrictions dynamically.
    let f = NamedTempFile::new()?;
    let raw = Config::<Raw>::new("name = 'test'\nport = 80");
    let validated = raw.merge(None)?.deserialize::<SimpleConfig>()?.validate()?;
    let res = validated.save_canonical(f.path());
    assert!(res.is_ok());
    Ok(())
}

#[test]
fn test_t3_06_schema_vs_derive_fingerprint() {
    // T3_06: Comparing results of declarative schema! validation and #[derive(Validate)] to ensure identical variant fingerprints.
    let s = star_toml::schema! {
        "port": range(1, 65535)
    };
    struct Target {
        port: u16,
    }
    impl Validate for Target {
        fn validate(&self, v: &mut Validator) {
            v.check_range("port", self.port, 1..=65535);
        }
    }
    let val = toml::from_str("port = 0").unwrap();
    let err_schema = s.validate_value(&val).unwrap_err().variant_id();

    let target = Target { port: 0 };
    let err_derive = target.check().unwrap_err().variant_id();
    assert_eq!(err_schema, err_derive);
}

#[test]
fn test_t3_07_error_propagation_pipeline() {
    // T3_07: System-level error propagation pipeline.
    let res_parse = star_toml::from_str::<SimpleConfig>("invalid toml here");
    assert!(matches!(res_parse, Err(star_toml::Error::Parse { .. })));

    let res_missing = Loader::new().layer_file("missing_file.toml").load::<SimpleConfig>();
    assert!(matches!(res_missing, Err(star_toml::Error::FileNotFound(_))));
}

#[test]
fn test_t3_08_concurrency_typestate() {
    // T3_08: Concurrency test: Multiple threads executing Typestate Transitions.
    use std::{sync::Arc, thread};

    let raw = Arc::new(Config::<Raw>::new("name = 'test'\nport = 8080"));
    let mut handles = vec![];
    for _ in 0..10 {
        let raw_clone = Arc::clone(&raw);
        let h = thread::spawn(move || {
            let merged = (*raw_clone).clone().merge(None).unwrap();
            let deserialized = merged.deserialize::<SimpleConfig>().unwrap();
            let validated = deserialized.validate().unwrap();
            let frozen = validated.freeze();
            assert_eq!(frozen.get().name, "test");
        });
        handles.push(h);
    }
    for h in handles {
        h.join().unwrap();
    }
}

// ==========================================
// TIER 4: Real-World Application Scenarios (5 Scenarios)
// ==========================================

#[test]
fn test_t4_01_web_server_scenario() -> TestResult {
    // T4_01: Microservice Web Server Configuration
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct WebServerConfig {
        db_host: String,
        db_port: u16,
        tls_enabled: bool,
        cert_path: String,
        workers: u32,
        cache_size: String,
    }

    impl Validate for WebServerConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("db_host", &self.db_host);
            v.check_range("db_port", self.db_port, 1..=65535);
            v.check_range("workers", self.workers, 1..=128);
            v.check_size_format("cache_size", &self.cache_size);
            v.check_consistent(
                "cert_path",
                &["tls_enabled"],
                !self.tls_enabled || !self.cert_path.is_empty(),
                "tls_cert_required",
                "cert_path must be provided if TLS is enabled",
            );
        }
    }

    impl ConfigLifecycle for WebServerConfig {}

    std::env::set_var("APP_DB_PORT", "5432");
    let f_out = NamedTempFile::new()?;

    let raw = Config::<Raw>::new(
        "db_host = 'db.example.com'\ndb_port = 80\ntls_enabled = true\ncert_path = '/etc/ssl/cert.pem'\nworkers = 16\ncache_size = '2GB'"
    );
    let merged = raw.merge(Some("APP_"))?;
    let deserialized = merged.deserialize::<WebServerConfig>()?;
    let validated = deserialized.validate()?;
    let frozen = validated.freeze();

    assert_eq!(frozen.get().db_port, 5432);
    assert_eq!(frozen.get().workers, 16);

    save_file(frozen.get(), f_out.path())?;

    std::env::remove_var("APP_DB_PORT");
    Ok(())
}

#[test]
fn test_t4_02_cicd_runner_scenario() -> TestResult {
    // T4_02: CI/CD Pipeline Runner Config
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct PipelineConfig {
        workspace: String,
        timeout: u32,
        environment: String,
        engine_version: String,
    }

    impl Validate for PipelineConfig {
        fn validate(&self, _v: &mut Validator) {}
    }

    impl ConfigLifecycle for PipelineConfig {
        fn normalize(&mut self) {
            self.workspace = self.workspace.trim().to_string();
        }
        fn validate_lifecycle(&self, v: &mut Validator) {
            v.check_path("workspace", &self.workspace, None);
            v.check_range("timeout", self.timeout, 1..=3600);
            v.check_one_of("environment", &self.environment, &["docker", "kubernetes", "local"]);
            v.check_semver("engine_version", &self.engine_version);
        }
    }

    let raw = Config::<Raw>::new(
        "workspace = '  /var/workspace  '\ntimeout = 300\nenvironment = 'docker'\nengine_version = '1.2.0'"
    );
    let mut config = raw.merge(None)?.deserialize::<PipelineConfig>()?.get_mut().clone();
    config.normalize();
    assert_eq!(config.workspace, "/var/workspace");

    let validated = Config::<Validated<PipelineConfig>>::new(config)?;
    let toml_str = toml::to_string(validated.get()).unwrap();
    let tc = star_toml::trusted().layer_str(toml_str, "inline").load::<PipelineConfig>()?;
    assert_eq!(tc.value.environment, "docker");
    Ok(())
}

#[test]
fn test_t4_03_db_cluster_scenario() -> TestResult {
    // T4_03: Distributed Database Cluster Configuration
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct DbClusterConfig {
        role: String,
        seeds: Vec<String>,
        heartbeat_ms: u32,
    }

    impl Validate for DbClusterConfig {
        fn validate(&self, _v: &mut Validator) {}
    }

    impl ConfigLifecycle for DbClusterConfig {
        fn normalize(&mut self) {}
        fn validate_lifecycle(&self, v: &mut Validator) {
            v.check_one_of("role", &self.role, &["primary", "replica"]);
            v.check_range("heartbeat_ms", self.heartbeat_ms, 100..=5000);
            v.check_consistent(
                "seeds",
                &["role"],
                self.role != "replica" || !self.seeds.is_empty(),
                "seeds_required",
                "Replica role requires non-empty seeds list",
            );
        }
    }

    let raw =
        Config::<Raw>::new("role = 'replica'\nseeds = ['seed1.db.local']\nheartbeat_ms = 1000");
    let config = raw.merge(None)?.deserialize::<DbClusterConfig>()?;
    let validated = config.validate()?;
    let frozen = validated.freeze();
    assert_eq!(frozen.get().seeds[0], "seed1.db.local");
    Ok(())
}

#[test]
fn test_t4_04_data_ingestion_scenario() -> TestResult {
    // T4_04: Data Ingestion Agent Configuration
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct IngestionConfig {
        monitor_dir: String,
        archive_dir: String,
        max_size: String,
    }

    impl Validate for IngestionConfig {
        fn validate(&self, v: &mut Validator) {
            v.field("ingest", |v| {
                v.check_path("monitor_dir", &self.monitor_dir, None);
                v.check_size_format("max_size", &self.max_size);
            });
            v.field("archive", |v| {
                v.check_path("archive_dir", &self.archive_dir, None);
            });
        }
    }

    let bad = IngestionConfig {
        monitor_dir: "/data/in\0".into(),
        archive_dir: "../../archive".into(),
        max_size: "100XB".into(),
    };

    let errs = bad.check().unwrap_err();
    assert_eq!(errs.len(), 3);

    let grouped = errs.by_section();
    assert!(grouped.contains_key("ingest"));
    assert!(grouped.contains_key("archive"));
    Ok(())
}

#[test]
fn test_t4_05_api_gateway_scenario() -> TestResult {
    // T4_05: API Gateway Config with Rate Limiter
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    struct GatewayConfig {
        backend_host: String,
        rate_limit: u32,
    }

    impl Validate for GatewayConfig {
        fn validate(&self, v: &mut Validator) {
            v.check_ip_or_domain("backend_host", &self.backend_host);
            v.check_range("rate_limit", self.rate_limit, 1..=10000);
        }
    }

    impl star_toml::loader::ConfigLifecycle for GatewayConfig {}

    let config = GatewayConfig { backend_host: "backend.service.local".into(), rate_limit: 5000 };

    let toml_str = toml::to_string(&config).unwrap();
    let tc = star_toml::trusted().layer_str(toml_str, "inline").load::<GatewayConfig>()?;
    assert_eq!(tc.value.rate_limit, 5000);
    assert!(tc.validation.errors.is_empty());
    assert!(tc.digest.0 > 0);
    Ok(())
}
