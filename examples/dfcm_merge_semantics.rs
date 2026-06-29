//! DfCM combinatorial integration test — layer merge invariants.
//!
//! Asserts the 7 core merge-semantics invariants using the public API:
//! `deep_merge`, `deep_merge_traced`, `WinnerMap`, and `Loader` env-prefix
//! injection. Panics on any violation.

use star_toml::{deep_merge, deep_merge_traced, WinnerMap};
use toml::Value;

fn main() {
    scalar_overlay_wins();
    array_replacement_not_concat();
    table_recursive_merge();
    winner_map_attribution();
    three_layer_ordering();
    env_dotted_path_injection();
    env_coercion_types();
    println!("✓ all dfcm_merge_semantics checks passed");
}

// ---------------------------------------------------------------------------
// 1. Scalar overlay wins
// ---------------------------------------------------------------------------

fn scalar_overlay_wins() {
    let mut base: Value = toml::from_str(r#"key = "base_value""#).unwrap();
    let overlay: Value = toml::from_str(r#"key = "overlay_value""#).unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(base["key"].as_str(), Some("overlay_value"), "overlay scalar must win");
}

// ---------------------------------------------------------------------------
// 2. Array replacement — not concatenation
// ---------------------------------------------------------------------------

fn array_replacement_not_concat() {
    let mut base: Value = toml::from_str(r#"list = [1, 2, 3]"#).unwrap();
    let overlay: Value = toml::from_str(r#"list = [99]"#).unwrap();
    deep_merge(&mut base, overlay);
    let list = base["list"].as_array().unwrap();
    assert_eq!(list.len(), 1, "overlay array must replace, not concat");
    assert_eq!(list[0].as_integer(), Some(99));
}

// ---------------------------------------------------------------------------
// 3. Table recursive merge — base-only keys preserved
// ---------------------------------------------------------------------------

fn table_recursive_merge() {
    let mut base: Value = toml::from_str(
        r#"[server]
host = "localhost"
port = 8080
timeout = 30
"#,
    )
    .unwrap();
    let overlay: Value = toml::from_str(
        r#"[server]
host = "remotehost"
"#,
    )
    .unwrap();
    deep_merge(&mut base, overlay);
    let srv = base["server"].as_table().unwrap();
    assert_eq!(srv["host"].as_str(), Some("remotehost"), "overlay table key must win");
    assert_eq!(srv["port"].as_integer(), Some(8080), "base-only key must be preserved");
    assert_eq!(srv["timeout"].as_integer(), Some(30), "second base-only key must be preserved");
}

// ---------------------------------------------------------------------------
// 4. WinnerMap attribution
// ---------------------------------------------------------------------------

fn winner_map_attribution() {
    let mut base: Value = toml::from_str(
        r#"[db]
host = "localhost"
port = 5432
name = "mydb"
"#,
    )
    .unwrap();
    let overlay: Value = toml::from_str(
        r#"[db]
host = "prodhost"
port = 5433
"#,
    )
    .unwrap();

    let mut winner_map = WinnerMap::new();
    // Seed base layer attribution
    deep_merge_traced(&mut base.clone(), base.clone(), "base", "", &mut winner_map);
    // Apply overlay
    deep_merge_traced(&mut base, overlay, "overlay", "", &mut winner_map);

    // Fields written by overlay must be attributed to "overlay"
    assert_eq!(winner_map.get("db.host").map(String::as_str), Some("overlay"));
    assert_eq!(winner_map.get("db.port").map(String::as_str), Some("overlay"));
    // Field not touched by overlay retains its attribution
    // (base layer attribution depends on call order — just verify it's present)
    assert!(winner_map.contains_key("db.name"), "db.name must appear in winner_map");
}

// ---------------------------------------------------------------------------
// 5. Three-layer ordering: A → B → C; C wins, A-only preserved
// ---------------------------------------------------------------------------

fn three_layer_ordering() {
    let mut combined: Value = toml::from_str(
        r#"a_only = "from_a"
shared = "from_a"
"#,
    )
    .unwrap();

    let layer_b: Value = toml::from_str(
        r#"shared = "from_b"
b_only = "from_b"
"#,
    )
    .unwrap();

    let layer_c: Value = toml::from_str(
        r#"shared = "from_c"
c_only = "from_c"
"#,
    )
    .unwrap();

    deep_merge(&mut combined, layer_b);
    deep_merge(&mut combined, layer_c);

    assert_eq!(combined["shared"].as_str(), Some("from_c"), "last layer (C) must win shared");
    assert_eq!(combined["a_only"].as_str(), Some("from_a"), "A-only key must be preserved");
    assert_eq!(combined["b_only"].as_str(), Some("from_b"), "B-only key must be preserved");
    assert_eq!(combined["c_only"].as_str(), Some("from_c"), "C-only key must be present");
}

// ---------------------------------------------------------------------------
// 6. Env dotted-path injection via Loader API (observable through load result)
// ---------------------------------------------------------------------------

fn env_dotted_path_injection() {
    use serde::{Deserialize, Serialize};
    use star_toml::{loader::Loader, Validate, Validator};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Cfg {
        server: Server,
    }
    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Server {
        port: u16,
        host: String,
    }
    impl Validate for Cfg {
        fn validate(&self, _v: &mut Validator) {}
    }

    let base_toml = r#"
[server]
host = "localhost"
port = 8080
"#;

    // Inject server.port via env prefix — double underscore (__) separates nested keys
    // DFCM_SERVER__PORT → server.port (not server_port)
    std::env::set_var("DFCM_SERVER__PORT", "9090");
    let result = Loader::new().layer_str(base_toml, "base").env_prefix("DFCM_").load::<Cfg>();
    std::env::remove_var("DFCM_SERVER__PORT");

    match result {
        Ok(cfg) => {
            assert_eq!(cfg.server.port, 9090, "env override must inject server.port=9090");
            assert_eq!(cfg.server.host, "localhost", "non-overridden field must be preserved");
        }
        Err(e) => panic!("env dotted-path injection failed: {e}"),
    }
}

// ---------------------------------------------------------------------------
// 7. Env coercion types: string → bool / i64 / f64 / string
// ---------------------------------------------------------------------------

fn env_coercion_types() {
    use serde::{Deserialize, Serialize};
    use star_toml::{loader::Loader, Validate, Validator};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Coerced {
        flag: bool,
        count: i64,
        ratio: f64,
        label: String,
    }
    impl Validate for Coerced {
        fn validate(&self, _v: &mut Validator) {}
    }

    let base_toml = r#"
flag = false
count = 0
ratio = 0.0
label = "original"
"#;

    // Flat top-level fields: single-segment env key maps directly
    std::env::set_var("COERCE_FLAG", "true");
    std::env::set_var("COERCE_COUNT", "42");
    std::env::set_var("COERCE_RATIO", "3.14");
    std::env::set_var("COERCE_LABEL", "injected");

    let result = Loader::new().layer_str(base_toml, "base").env_prefix("COERCE_").load::<Coerced>();

    std::env::remove_var("COERCE_FLAG");
    std::env::remove_var("COERCE_COUNT");
    std::env::remove_var("COERCE_RATIO");
    std::env::remove_var("COERCE_LABEL");

    match result {
        Ok(cfg) => {
            assert!(cfg.flag, "\"true\" must coerce to bool true");
            assert_eq!(cfg.count, 42, "\"42\" must coerce to i64");
            assert!((cfg.ratio - 3.14).abs() < 0.001, "\"3.14\" must coerce to f64");
            assert_eq!(cfg.label, "injected", "string must remain string");
        }
        Err(e) => panic!("env coercion test failed: {e}"),
    }
}
