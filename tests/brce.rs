//! BRCE test suite for WP-1 through WP-3.
//!
//! BRCE categories used in this file:
//!   truth         — positive correctness under normal inputs
//!   falsification — inputs that must be rejected / produce errors
//!   invariant     — structural rules that must always hold
//!   conservation  — nothing is silently lost (provenance completeness)
//!   determinism   — same inputs always produce same outputs
//!   provenance    — every piece of data can be traced to its origin

use std::io::Write;

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{BoundedSources, ConfigLifecycle},
    merge::{deep_merge_traced, WinnerMap},
    reports::{SourceKind},
    Config, Error, Loader, TrustedLoader, Validate, Validator,
};
use tempfile::{NamedTempFile, TempDir};
use toml::Value;
// Suppress unused imports from earlier patterns
#[allow(unused_imports)]
use star_toml::{PathPolicy, PathWitness, resolve_and_validate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_toml(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Collect all dot-separated leaf paths from a TOML Value tree.
fn leaf_paths(val: &Value, prefix: &str) -> Vec<String> {
    match val {
        Value::Table(t) => t
            .iter()
            .flat_map(|(k, v)| {
                let p =
                    if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                leaf_paths(v, &p)
            })
            .collect(),
        _ => vec![prefix.to_owned()],
    }
}

fn parse(s: &str) -> Value {
    toml::from_str(s).unwrap()
}

// ---------------------------------------------------------------------------
// Simple validated config type used across tests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Cfg {
    name: String,
    port: u16,
}

impl Validate for Cfg {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
    }
}

impl ConfigLifecycle for Cfg {}

// ---------------------------------------------------------------------------
// WP-1: Typestate lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_load_frozen_requires_validation() {
    // BRCE: invariant
    // load_frozen must run validation; a config that fails validation must error.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"\"\nport = 80\n"); // both fields invalid

    let res = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_frozen::<Cfg>();

    assert!(res.is_err(), "load_frozen must not succeed when validation fails");
    assert!(matches!(res, Err(Error::Invalid(_))));
}

#[test]
fn test_load_frozen_succeeds_with_valid_config() {
    // BRCE: invariant
    // load_frozen succeeds when config is valid.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_frozen::<Cfg>()
        .unwrap();

    assert_eq!(result.config.get().name, "app");
    assert_eq!(result.config.get().port, 8080);
}

// ---------------------------------------------------------------------------
// WP-2: SourceReport
// ---------------------------------------------------------------------------

#[test]
fn test_source_report_records_required_file() {
    // BRCE: provenance
    // SourceReport must contain an entry for every required file that was loaded.
    let dir = TempDir::new().unwrap();
    let path = write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    let result = TrustedLoader::new()
        .layer_file(path.clone())
        .load_frozen::<Cfg>()
        .unwrap();

    let entries = &result.source_report.entries;
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.source_kind, SourceKind::File);
    assert!(e.found);
    assert!(e.required);
    assert_eq!(e.path.as_ref().unwrap(), &path);
    assert!(e.digest.is_some(), "digest must be recorded for a found file");
    assert!(e.size_bytes.is_some());
}

#[test]
fn test_missing_required_file_fails() {
    // BRCE: falsification
    // A required file that does not exist must produce an immediate error.
    let res = TrustedLoader::new()
        .layer_file("/nonexistent/does-not-exist.toml")
        .load_frozen::<Cfg>();

    assert!(matches!(res, Err(Error::FileNotFound(_))));
}

#[test]
fn test_optional_missing_file_is_reported() {
    // BRCE: provenance
    // An optional file that is missing must appear in SourceReport with found=false.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "base.toml", "name = \"app\"\nport = 8080\n");

    let bounded = Loader::new()
        .layer_file(dir.path().join("base.toml"))
        .layer_file_if_exists("/nonexistent/optional.toml")
        .load_bounded()
        .unwrap();

    let missing: Vec<_> = bounded.state.source_report.missing_optional_entries().collect();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].source_kind, SourceKind::OptionalFile);
    assert!(!missing[0].found);
    assert!(!missing[0].required);
}

#[test]
fn test_source_report_str_layer() {
    // BRCE: provenance
    // Str layers also appear in SourceReport with found=true.
    let bounded = Loader::new()
        .layer_str("name = \"x\"\nport = 1025\n", "defaults")
        .load_bounded()
        .unwrap();

    let entries = &bounded.state.source_report.entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].source_kind, SourceKind::Str);
    assert!(entries[0].found);
    assert!(entries[0].digest.is_some());
}

// ---------------------------------------------------------------------------
// WP-2: EnvOverrideReport
// ---------------------------------------------------------------------------

#[test]
fn test_env_override_report_records_prefix_mapping() {
    // BRCE: provenance
    // EnvOverrideReport must record accepted overrides with their mapped path.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"original\"\nport = 8080\n");

    std::env::set_var("BRCE_TEST1_PORT", "9090");
    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .env_prefix("BRCE_TEST1_")
        .load_frozen::<Cfg>();
    std::env::remove_var("BRCE_TEST1_PORT");

    let result = result.unwrap();
    assert_eq!(result.config.get().port, 9090);

    let accepted: Vec<_> = result.env_report.accepted().collect();
    assert!(!accepted.is_empty(), "BRCE_TEST1_PORT must appear as an accepted override");
    let entry = accepted.iter().find(|e| e.mapped_path == "port").unwrap();
    assert!(entry.accepted);
    assert_eq!(entry.configured_prefix, "BRCE_TEST1_");
    assert!(!entry.raw_value_digest.is_empty());
    assert!(entry.coerced_type.is_some());
}

#[test]
fn test_unprefixed_admitted_env_override_fails() {
    // BRCE: falsification
    // An env var without the configured prefix must NOT be admitted as a config override.
    // It must not appear in EnvOverrideReport and must not affect the merged config.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"original\"\nport = 8080\n");

    // Set an env var with a DIFFERENT prefix — should be completely ignored.
    std::env::set_var("TOTALLY_DIFFERENT_PORT", "1111");
    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .env_prefix("BRCE_TEST2_") // none of the test vars match this prefix
        .load_frozen::<Cfg>();
    std::env::remove_var("TOTALLY_DIFFERENT_PORT");

    let result = result.unwrap();
    // port must remain 8080 — the unprefixed var was not admitted
    assert_eq!(result.config.get().port, 8080);
    // And it must not appear in the env report at all
    let found = result
        .env_report
        .entries
        .iter()
        .any(|e| e.raw_env_key == "TOTALLY_DIFFERENT_PORT");
    assert!(!found, "unprefixed var must not appear in EnvOverrideReport");
}

// ---------------------------------------------------------------------------
// WP-3: deep_merge_traced
// ---------------------------------------------------------------------------

#[test]
fn test_table_merge_recursive() {
    // BRCE: truth
    // Tables merge key-by-key; keys absent in overlay are preserved from base.
    let mut base = parse("[db]\nhost = \"localhost\"\nport = 5432\n");
    let overlay = parse("[db]\nport = 5433\n");
    let mut wm = WinnerMap::new();
    deep_merge_traced(&mut base, overlay, "layer-1", "", &mut wm);

    assert_eq!(base["db"]["host"].as_str(), Some("localhost")); // preserved
    assert_eq!(base["db"]["port"].as_integer(), Some(5433)); // overridden
    assert_eq!(wm.get("db.port").map(String::as_str), Some("layer-1"));
    assert!(!wm.contains_key("db.host"), "host was not touched by overlay");
}

#[test]
fn test_array_replacement_not_merge() {
    // BRCE: invariant
    // Arrays are replaced entirely by the higher-priority array, never merged element-wise.
    let mut base = parse("tags = [\"a\", \"b\", \"c\"]\n");
    let overlay = parse("tags = [\"x\"]\n");
    let mut wm = WinnerMap::new();
    deep_merge_traced(&mut base, overlay, "layer-1", "", &mut wm);

    let arr = base["tags"].as_array().unwrap();
    assert_eq!(arr.len(), 1, "array must be replaced, not appended");
    assert_eq!(arr[0].as_str(), Some("x"));
    assert_eq!(wm.get("tags").map(String::as_str), Some("layer-1"));
}

#[test]
fn test_scalar_replacement() {
    // BRCE: invariant
    // A scalar in overlay replaces the scalar in base.
    let mut base = parse("count = 1\n");
    let mut wm = WinnerMap::new();
    deep_merge_traced(&mut base, parse("count = 99\n"), "env", "", &mut wm);

    assert_eq!(base["count"].as_integer(), Some(99));
    assert_eq!(wm.get("count").map(String::as_str), Some("env"));
}

#[test]
fn test_layer_order_defaults_files_env() {
    // BRCE: determinism
    // Priority: defaults (layer-0) < file (layer-1) < env (layer-2).
    // Each higher-priority layer wins over the lower one.
    let mut merged = parse("port = 1\nname = \"defaults\"\n");
    let mut wm = WinnerMap::new();

    deep_merge_traced(&mut merged, parse("port = 2\n"), "layer-file", "", &mut wm);
    deep_merge_traced(&mut merged, parse("port = 3\n"), "env", "", &mut wm);

    assert_eq!(merged["port"].as_integer(), Some(3));
    assert_eq!(wm.get("port").map(String::as_str), Some("env"));
    // name was never touched by file or env layers
    assert_eq!(merged["name"].as_str(), Some("defaults"));
}

#[test]
fn test_winning_layer_tracing() {
    // BRCE: conservation
    // Every field written by a layer must appear in winner_map with that layer's id.
    let mut base = parse("a = 1\n[s]\nb = 2\n");
    let mut wm = WinnerMap::new();

    // layer-0 sets a and s.b
    deep_merge_traced(&mut base, parse("a = 1\n[s]\nb = 2\n"), "layer-0", "", &mut wm);
    // layer-1 adds s.c
    deep_merge_traced(&mut base, parse("[s]\nc = 3\n"), "layer-1", "", &mut wm);

    assert_eq!(wm.get("a").map(String::as_str), Some("layer-0"));
    assert_eq!(wm.get("s.b").map(String::as_str), Some("layer-0"));
    assert_eq!(wm.get("s.c").map(String::as_str), Some("layer-1"));
}

#[test]
fn test_every_final_field_has_winning_layer() {
    // BRCE: conservation
    // After all layers are applied, every leaf in the merged value has an entry in
    // the global_winner_map. A field without provenance is a conservation failure.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "base.toml", "name = \"app\"\nport = 8080\n");
    write_toml(&dir, "override.toml", "port = 9090\n");

    let bounded = Loader::new()
        .layer_file(dir.path().join("base.toml"))
        .layer_file(dir.path().join("override.toml"))
        .load_bounded()
        .unwrap();

    let global_winner_map = &bounded.state.global_winner_map;
    let final_value = &bounded.state.value;

    let leaves = leaf_paths(final_value, "");
    for path in &leaves {
        assert!(
            global_winner_map.contains_key(path),
            "conservation failure: field '{path}' has no winning layer in global_winner_map"
        );
    }
}

// ---------------------------------------------------------------------------
// WP-2: LayerReport
// ---------------------------------------------------------------------------

#[test]
fn test_layer_report_records_all_layers() {
    // BRCE: provenance
    // LayerReport must have one entry per source that contributed content.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "base.toml", "name = \"app\"\nport = 8080\n");
    write_toml(&dir, "override.toml", "port = 9090\n");

    let bounded = Loader::new()
        .layer_str("name = \"default\"\nport = 1000\n", "defaults")
        .layer_file(dir.path().join("base.toml"))
        .layer_file(dir.path().join("override.toml"))
        .load_bounded()
        .unwrap();

    assert_eq!(bounded.state.layer_report.entries.len(), 3);
    // Layers are in merge order (lowest priority first)
    assert_eq!(bounded.state.layer_report.entries[0].layer_name, "defaults");
}

#[test]
fn test_layer_order_digest_is_deterministic() {
    // BRCE: determinism
    // The same sequence of layers must always produce the same layer_order_digest.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    let b1 = Loader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_bounded()
        .unwrap();
    let b2 = Loader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_bounded()
        .unwrap();

    let d1 = &b1.state.layer_report.entries[0].layer_order_digest;
    let d2 = &b2.state.layer_report.entries[0].layer_order_digest;
    assert_eq!(d1, d2, "layer_order_digest must be deterministic");
}

// ---------------------------------------------------------------------------
// WP-1: load_frozen with env (integration)
// ---------------------------------------------------------------------------

#[test]
fn test_load_frozen_env_override_applied() {
    // BRCE: truth
    // load_frozen must apply env overrides as the highest priority layer.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    std::env::set_var("BRCE_LF_PORT", "7777");
    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .env_prefix("BRCE_LF_")
        .load_frozen::<Cfg>();
    std::env::remove_var("BRCE_LF_PORT");

    let result = result.unwrap();
    assert_eq!(result.config.get().port, 7777);
    assert_eq!(result.config.get().name, "app");
    // global_winner_map must show env winning on port
    assert_eq!(result.global_winner_map.get("port").map(String::as_str), Some("env"));
}

// ---------------------------------------------------------------------------
// ST-109: ConfigWitness tests
// ---------------------------------------------------------------------------

#[test]
fn test_witness_is_deterministic() {
    // BRCE: determinism
    // Same inputs must always produce the same ConfigWitness hash.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    let r1 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    let r2 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    assert_eq!(r1.witness.hash, r2.witness.hash, "witness must be deterministic");
}

#[test]
fn test_witness_changes_on_source_change() {
    // BRCE: determinism
    // Different file content must produce a different ConfigWitness hash.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg1.toml", "name = \"app\"\nport = 8080\n");
    write_toml(&dir, "cfg2.toml", "name = \"app\"\nport = 9090\n");

    let r1 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg1.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    let r2 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg2.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    assert_ne!(r1.witness.hash, r2.witness.hash, "witness must change when source changes");
}

// ---------------------------------------------------------------------------
// ST-108: Path bounds tests
// ---------------------------------------------------------------------------

#[test]
fn test_path_traversal_fails() {
    // BRCE: falsification
    let source = std::path::Path::new("/tmp/project/config.toml");
    let policy = star_toml::PathPolicy::BlockForbidden;
    let result = star_toml::resolve_and_validate("../secret.toml", source, &policy);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "path_traversal_detected");
}

#[test]
fn test_null_byte_fails() {
    // BRCE: falsification
    let source = std::path::Path::new("/tmp/config.toml");
    let policy = star_toml::PathPolicy::BlockForbidden;
    let result = star_toml::resolve_and_validate("foo\0bar", source, &policy);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "null_byte_detected");
}

#[test]
fn test_forbidden_path_fails() {
    // BRCE: falsification
    let source = std::path::Path::new("/tmp/config.toml");
    let policy = star_toml::PathPolicy::BlockForbidden;
    let result = star_toml::resolve_and_validate("/etc/passwd", source, &policy);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "forbidden_path");
}

#[test]
fn test_relative_resolved_against_source_parent() {
    // BRCE: truth
    // A relative path is resolved against the directory containing the source file.
    let source = std::path::Path::new("/home/user/project/config.toml");
    let policy = star_toml::PathPolicy::Sandbox { root: std::path::PathBuf::from("/home/user/project") };
    let (resolved, witness) = star_toml::resolve_and_validate("data/input.csv", source, &policy).unwrap();
    assert_eq!(resolved, std::path::PathBuf::from("/home/user/project/data/input.csv"));
    assert!(witness.accepted);
}

#[test]
fn test_path_witness_emitted() {
    // BRCE: provenance
    // check_path_safe must emit a PathWitness regardless of pass/fail.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");
    let source = dir.path().join("cfg.toml");

    let mut v = star_toml::Validator::new();
    v.check_path_safe(
        "output",
        "output/result.txt",
        &source,
        star_toml::PathPolicy::BlockForbidden,
    );
    assert_eq!(v.path_witnesses.len(), 1);
    assert!(v.path_witnesses[0].accepted);

    // Also test rejection
    let mut v2 = star_toml::Validator::new();
    v2.check_path_safe("bad", "/etc/hosts", &source, star_toml::PathPolicy::BlockForbidden);
    assert_eq!(v2.path_witnesses.len(), 1);
    assert!(!v2.path_witnesses[0].accepted);
}

// ---------------------------------------------------------------------------
// ST-102: AdmittedConfig tests
// ---------------------------------------------------------------------------

#[test]
fn test_load_admitted_succeeds() {
    // BRCE: truth
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"admitted\"\nport = 8080\n");

    let admitted = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    assert_eq!(admitted.name, "admitted");
    assert_eq!(admitted.port, 8080);
    assert!(!admitted.witness.hash.is_empty());
    assert_eq!(admitted.source_report.entries.len(), 1);
}

#[test]
fn test_load_admitted_strict_rejects_unknown_fields() {
    // BRCE: falsification
    // load_admitted_strict must reject configs with fields not in the struct.
    let dir = TempDir::new().unwrap();
    // Cfg only has name and port — extra_field is unknown
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\nextra_field = \"oops\"\n");

    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted_strict::<Cfg>();

    assert!(result.is_err(), "unknown fields must cause load_admitted_strict to fail");
    if let Err(star_toml::Error::Invalid(errs)) = result {
        assert!(errs.errors().iter().any(|e| e.code() == "unknown_field"));
    } else {
        panic!("Expected Error::Invalid with unknown_field");
    }
}

// ---------------------------------------------------------------------------
// ST-111: BRCE ladder additions
// ---------------------------------------------------------------------------

#[test]
fn test_brce_metamorphic_canonical_stability() {
    // BRCE: determinism
    // Re-loading the same file twice must produce identical canonical TOML.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"stable\"\nport = 4321\n");

    let r1 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();
    let r2 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    assert_eq!(r1.witness.hash, r2.witness.hash);
}

#[test]
fn test_brce_idempotence_canonical() {
    // BRCE: determinism
    // ConfigWitness hash must be identical across two admissions of the same file.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"idempotent\"\nport = 5555\n");

    let a = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();
    let b = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load_admitted::<Cfg>()
        .unwrap();

    assert_eq!(a.witness.hash, b.witness.hash, "admission is idempotent");
}

#[test]
fn test_brce_env_coercion_deterministic() {
    // BRCE: determinism
    // Same env var value must always coerce to the same type.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"app\"\nport = 8080\n");

    std::env::set_var("BRCE_DET_PORT", "9000");
    let r1 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .env_prefix("BRCE_DET_")
        .load_frozen::<Cfg>()
        .unwrap();
    let r2 = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .env_prefix("BRCE_DET_")
        .load_frozen::<Cfg>()
        .unwrap();
    std::env::remove_var("BRCE_DET_PORT");

    let t1 = r1.env_report.accepted().next().unwrap().coerced_type.clone();
    let t2 = r2.env_report.accepted().next().unwrap().coerced_type.clone();
    assert_eq!(t1, t2, "coercion type must be deterministic");
}

#[test]
fn test_brce_fitness_score() {
    // BRCE: truth
    // A config that passes all validation checks must yield fitness 1.0.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "cfg.toml", "name = \"ok\"\nport = 8080\n");

    let result = TrustedLoader::new()
        .layer_file(dir.path().join("cfg.toml"))
        .load::<Cfg>()
        .unwrap();

    assert_eq!(result.validation.fitness, 1.0);
}

#[test]
fn test_brce_repair_hint_nonempty() {
    // BRCE: truth
    // Every validation error must have a non-empty repair hint.
    let dir = TempDir::new().unwrap();
    write_toml(&dir, "bad.toml", "name = \"\"\nport = 80\n");

    let err = TrustedLoader::new()
        .layer_file(dir.path().join("bad.toml"))
        .load_frozen::<Cfg>()
        .unwrap_err();

    if let star_toml::Error::Invalid(errs) = err {
        for e in errs.errors() {
            assert!(!e.repair_hint().is_empty(), "repair_hint must not be empty for {}", e.code());
        }
    } else {
        panic!("Expected Error::Invalid");
    }
}
