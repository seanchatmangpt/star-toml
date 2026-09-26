//! Conservation + adversarial court for dependency bumps (PR #12, v26.9.26).
//!
//! A dependency bump (toml / `toml_parser` / `toml_writer` / serde / blake3 ...)
//! claims "no behavior change". star-toml derives observable identities from
//! those crates: [`star_toml::loader::ConfigDigest`] is FNV-1a over
//! `toml::to_string(merged)`, and `save_canonical` writes `toml_writer` bytes.
//! A writer change therefore silently breaks every stored digest / canonical
//! file (replay mismatch). These tests pin both to golden values computed on
//! base `main` (08f36b0) BEFORE the bump, so the bump is admitted only if it
//! conserves them.
//!
//! Chicago style: every collaborator is real (real `toml` parser/writer, real
//! filesystem via `tempfile`, real `Cargo.lock`, real workflow files). No
//! doubles of any kind.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::pedantic, missing_docs)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use star_toml::{
    deep_merge,
    loader::{Config, ConfigLifecycle, Raw, TrustedLoader},
    Error, Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Worker {
    name: String,
    threads: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Db {
    url: String,
    pool: u32,
    ratio: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct App {
    name: String,
    port: u16,
    motto: String,
    tags: Vec<String>,
    db: Db,
    workers: Vec<Worker>,
}

impl Validate for App {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
        v.check_range("db.pool", self.db.pool, 1..=512);
    }
}
impl ConfigLifecycle for App {}

/// Fixture exercising the writer's escaping paths (quotes, backslash, control
/// chars, non-ASCII, emoji), floats, nested tables, and arrays of tables.
const FIXTURE: &str = r#"
name = "svc"
port = 8443
motto = "say \"hi\"\tthen\\leave\n— ünïcødé ✓ 🦀"
tags = ["a", "b c", "d\"e"]

[db]
url = "postgres://u@h:5432/db?sslmode=require"
pool = 16
ratio = 0.75

[[workers]]
name = "alpha"
threads = 4

[[workers]]
name = "beta"
threads = 8
"#;

/// Golden digest of FIXTURE, computed on base `main` 08f36b0 (pre-bump lockfile).
const GOLDEN_DIGEST: u64 = 0x9e37_8126_4cff_d5b0;

/// Golden canonical bytes of FIXTURE, computed on base `main` 08f36b0.
const GOLDEN_CANONICAL: &str = include_str!("fixtures/pr12_canonical_golden.toml");

fn trusted_digest(content: &str) -> Result<u64, Error> {
    Ok(TrustedLoader::new().layer_str(content.to_owned(), "fixture").load::<App>()?.digest.0)
}

fn canonical_bytes(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let validated = Config::<Raw>::new(content).merge(None)?.deserialize::<App>()?.validate()?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("nested/out.toml");
    validated.save_canonical(&path)?;
    Ok(std::fs::read_to_string(&path)?)
}

// ---------------------------------------------------------------------------
// Replay conservation (stale subject / replay mismatch)
// ---------------------------------------------------------------------------

#[test]
fn digest_is_conserved_across_dependency_bump() {
    let d = trusted_digest(FIXTURE).expect("fixture admits");
    assert_eq!(
        d, GOLDEN_DIGEST,
        "ConfigDigest drifted: got {d:#018x}, golden {GOLDEN_DIGEST:#018x}. \
         A toml/toml_writer bump changed serialization; stored digests would no longer replay."
    );
}

#[test]
fn canonical_save_bytes_are_conserved_across_dependency_bump() {
    let got = canonical_bytes(FIXTURE).expect("canonical save");
    assert_eq!(got, GOLDEN_CANONICAL, "canonical bytes drifted from pre-bump golden");
}

#[test]
fn canonical_bytes_round_trip_to_identical_value() {
    let got = canonical_bytes(FIXTURE).expect("canonical save");
    let reparsed: App = star_toml::from_str(&got).expect("canonical output re-parses");
    let original: App = star_toml::from_str(FIXTURE).expect("fixture parses");
    assert_eq!(reparsed, original);
    // Canonicalization is a fixed point.
    assert_eq!(canonical_bytes(&got).expect("second save"), got);
}

#[test]
fn digest_is_insensitive_to_comments_and_whitespace() {
    let noisy = FIXTURE
        .replace("pool = 16", "pool   =   16   # comment")
        .replace("\n[db]", "\n\n# c\n[db]");
    assert_eq!(trusted_digest(&noisy).unwrap(), trusted_digest(FIXTURE).unwrap());
}

// ---------------------------------------------------------------------------
// Wrong digest: any semantic change must move the digest
// ---------------------------------------------------------------------------

#[test]
fn every_single_value_mutation_changes_the_digest() {
    let base = trusted_digest(FIXTURE).unwrap();
    let mutants = [
        ("port = 8443", "port = 8444"),
        ("pool = 16", "pool = 17"),
        ("ratio = 0.75", "ratio = 0.7500001"),
        ("name = \"alpha\"", "name = \"alpho\""),
        ("threads = 8", "threads = 9"),
        ("\"b c\"", "\"b  c\""),
        ("🦀", "🦞"),
    ];
    for (from, to) in mutants {
        assert!(FIXTURE.contains(from), "mutant anchor {from} missing");
        let m = FIXTURE.replacen(from, to, 1);
        let d = trusted_digest(&m).unwrap();
        assert_ne!(d, base, "mutation {from} -> {to} did not change digest");
    }
}

// ---------------------------------------------------------------------------
// Duplicate delivery / reordering of layers
// ---------------------------------------------------------------------------

#[test]
fn duplicate_layer_delivery_is_idempotent() {
    let once = trusted_digest(FIXTURE).unwrap();
    let twice = TrustedLoader::new()
        .layer_str(FIXTURE, "a")
        .layer_str(FIXTURE, "a-again")
        .load::<App>()
        .unwrap()
        .digest
        .0;
    assert_eq!(once, twice);

    let mut v: toml::Value = toml::from_str(FIXTURE).unwrap();
    let before = v.clone();
    deep_merge(&mut v, before.clone());
    assert_eq!(v, before, "deep_merge(x, x) must equal x");
}

#[test]
fn layer_reordering_changes_the_winner_and_the_digest() {
    let a = "port = 2000";
    let b = "port = 3000";
    let first = TrustedLoader::new().layer_str(FIXTURE, "f").layer_str(a, "a").layer_str(b, "b");
    let second = TrustedLoader::new().layer_str(FIXTURE, "f").layer_str(b, "b").layer_str(a, "a");
    let first = first.load::<App>().unwrap();
    let second = second.load::<App>().unwrap();
    assert_eq!(first.port, 3000, "last layer wins");
    assert_eq!(second.port, 2000, "last layer wins");
    assert_ne!(first.digest, second.digest);
}

// ---------------------------------------------------------------------------
// Malformed input: every case must be a typed Parse error, never a panic
// ---------------------------------------------------------------------------

#[test]
fn malformed_toml_is_refused_as_typed_parse_error() {
    let cases: &[(&str, &str)] = &[
        ("unterminated string", "name = \"svc"),
        ("duplicate key", "port = 1\nport = 2"),
        ("duplicate table", "[db]\npool = 1\n[db]\npool = 2"),
        ("table redefines dotted key", "db.pool = 1\n[db]\npool = 2"),
        ("invalid escape", "name = \"\\q\""),
        ("integer overflow", "port = 99999999999999999999999"),
        ("bad datetime", "at = 1979-13-45T25:61:61Z"),
        ("bare newline in basic string", "name = \"a\nb\""),
        ("missing value", "port ="),
        ("leading zero integer", "port = 0123"),
        ("NUL control char", "name = \"a\u{0000}b\""),
        ("array of tables vs table clash", "[workers]\nname = \"x\"\n[[workers]]\nname = \"y\""),
    ];
    for (label, input) in cases {
        match star_toml::from_str::<toml::Value>(input) {
            Err(Error::Parse { .. }) => {}
            other => panic!("{label}: expected Error::Parse, got {other:?}"),
        }
        match TrustedLoader::new().layer_str(*input, "adv").load::<App>() {
            Err(Error::Parse { path, .. }) => assert_eq!(path, "adv", "{label}: label lost"),
            other => panic!("{label}: TrustedLoader expected Error::Parse, got {other:?}"),
        }
    }
}

#[test]
fn toml_1_1_spec_extensions_are_conserved() {
    // toml 1.1 (spec-1.1.0) admits trailing commas and newlines in inline
    // tables. Pinned so a bump that silently reverts to TOML 1.0 semantics
    // (refusing configs that admitted yesterday) fails here.
    let v: toml::Value =
        star_toml::from_str("db = { pool = 1, }").expect("trailing comma admitted");
    assert_eq!(v["db"]["pool"].as_integer(), Some(1));
    let v: toml::Value = star_toml::from_str("db = {\n  pool = 2,\n  ratio = 0.5\n}")
        .expect("multiline inline table");
    assert_eq!(v["db"]["pool"].as_integer(), Some(2));
}

#[test]
fn deep_nesting_is_bounded_not_a_stack_overflow() {
    // 64 levels is legal and must parse; the parser must not abort the process.
    let depth = 64;
    let s = format!("x = {}1{}", "[".repeat(depth), "]".repeat(depth));
    let v: toml::Value = star_toml::from_str(&s).expect("64-deep array parses");
    assert!(v.get("x").is_some());
    // A pathological 100_000-deep document must be refused or parsed, never crash.
    let depth = 100_000;
    let s = format!("x = {}1{}", "[".repeat(depth), "]".repeat(depth));
    let r = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || star_toml::from_str::<toml::Value>(&s).is_ok())
        .unwrap()
        .join();
    assert!(r.is_ok(), "parser panicked on deep nesting");
}

#[test]
fn invalid_utf8_file_is_refused_as_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("bad.toml");
    std::fs::write(&p, b"name = \"\xff\xfe\"\n").unwrap();
    match TrustedLoader::new().layer_file(&p).load::<App>() {
        Err(Error::Io { path, .. }) => assert_eq!(path, p),
        other => panic!("expected Error::Io for invalid UTF-8, got {other:?}"),
    }
}

#[test]
fn type_confusion_is_refused_not_coerced() {
    let bad = FIXTURE.replace("port = 8443", "port = \"8443\"");
    assert!(matches!(
        TrustedLoader::new().layer_str(bad, "t").load::<App>(),
        Err(Error::Parse { .. })
    ));
    let neg = FIXTURE.replace("port = 8443", "port = -1");
    assert!(matches!(
        TrustedLoader::new().layer_str(neg, "t").load::<App>(),
        Err(Error::Parse { .. })
    ));
}

#[test]
fn out_of_range_values_are_refused_by_validation_after_parse() {
    let bad = FIXTURE.replace("port = 8443", "port = 80");
    assert!(matches!(
        TrustedLoader::new().layer_str(bad, "t").load::<App>(),
        Err(Error::Invalid(_))
    ));
}

// ---------------------------------------------------------------------------
// Lockfile coherence (stale subject): the bump must move serde's three halves
// together and resolve exactly one toml.
// ---------------------------------------------------------------------------

fn lock_versions(name: &str) -> Vec<String> {
    let lock = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"))
        .expect("Cargo.lock present");
    let v: toml::Value = toml::from_str(&lock).expect("Cargo.lock is TOML");
    v["package"]
        .as_array()
        .expect("package array")
        .iter()
        .filter(|p| p["name"].as_str() == Some(name))
        .map(|p| p["version"].as_str().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn lockfile_serde_family_is_coherent() {
    let serde = lock_versions("serde");
    let core = lock_versions("serde_core");
    let derive = lock_versions("serde_derive");
    assert_eq!(serde.len(), 1, "exactly one serde: {serde:?}");
    assert_eq!(serde, core, "serde and serde_core must resolve together");
    assert_eq!(serde, derive, "serde and serde_derive must resolve together");
    let te = lock_versions("thiserror");
    let ti = lock_versions("thiserror-impl");
    assert_eq!(te, ti, "thiserror and thiserror-impl must resolve together");
}

#[test]
fn lockfile_resolves_exactly_one_toml_1x_frontend() {
    // A transitive toml 0.8 may coexist (third-party deps); star-toml's own
    // toml 1.x stack must resolve exactly once so digests have one writer.
    for name in ["toml", "toml_parser", "toml_writer"] {
        let v: Vec<String> =
            lock_versions(name).into_iter().filter(|v| v.starts_with("1.")).collect();
        assert_eq!(v.len(), 1, "{name} 1.x must resolve once, got {v:?}");
    }
}

// ---------------------------------------------------------------------------
// Unauthorized action: no workflow may push to the repository except the
// tag-triggered release workflow. A self-mutating "repair" transport (as
// landed in 3f47c70) writes unreceipted commits from CI and is refused.
// ---------------------------------------------------------------------------

#[test]
fn no_workflow_pushes_commits_or_holds_write_except_release() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("workflows dir") {
        let p = entry.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }
        let text = std::fs::read_to_string(&p).unwrap();
        let pushes = text.lines().any(|l| {
            let l = l.trim();
            !l.starts_with('#') && l.contains("git push")
        });
        let writes = text.contains("contents: write");
        if pushes || (writes && name != "release.yml") {
            offenders.push(name);
        }
    }
    assert!(offenders.is_empty(), "workflows with unauthorized write/push: {offenders:?}");
}
