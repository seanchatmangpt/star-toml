//! Deep-merge for `toml::Value` trees.
//!
//! Used by [`crate::Loader`] to compose layered configs: earlier layers supply defaults,
//! later layers override specific keys.

use std::collections::BTreeMap;

use toml::Value;

// ---------------------------------------------------------------------------
// Field provenance (WP-3)
// ---------------------------------------------------------------------------

/// Maps every final leaf config path (e.g. `"server.port"`) to the layer-id string
/// of the layer that last wrote it.
///
/// Layer-id strings are caller-supplied tags such as `"layer-0"`, `"layer-1"`, or `"env"`.
pub type WinnerMap = BTreeMap<String, String>;

/// Recursively merge `overlay` into `base`.
///
/// - **Tables**: keys present in `overlay` are inserted or recursively merged into `base`.
///   Keys absent from `overlay` are left unchanged in `base`.
/// - **Everything else** (arrays, strings, numbers, booleans, datetimes): `overlay`
///   replaces `base` entirely.
///
/// # Examples
///
/// ```
/// use toml::Value;
/// use star_toml::deep_merge;
///
/// let mut base: Value = toml::from_str(r#"
/// [server]
/// host = "localhost"
/// port = 8080
/// "#).unwrap();
///
/// let overlay: Value = toml::from_str(r#"
/// [server]
/// port = 9090
/// "#).unwrap();
///
/// deep_merge(&mut base, overlay);
///
/// let tbl = base.as_table().unwrap();
/// let server = tbl["server"].as_table().unwrap();
/// assert_eq!(server["host"].as_str(), Some("localhost")); // preserved
/// assert_eq!(server["port"].as_integer(), Some(9090));    // overridden
/// ```
pub fn deep_merge(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Table(base_tbl), Value::Table(overlay_tbl)) => {
            for (key, val) in overlay_tbl {
                match base_tbl.get_mut(&key) {
                    Some(existing) => deep_merge(existing, val),
                    None => {
                        base_tbl.insert(key, val);
                    }
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

/// Like [`deep_merge`] but records field-level provenance in `winner_map`.
///
/// Every leaf (scalar or array) that `overlay` writes is recorded as
/// `field_path → layer_id` in `winner_map`.  Fields `overlay` does not touch
/// retain whatever winner a prior call recorded.
///
/// `prefix` is a dot-separated path prefix used during recursion; pass `""` at the
/// top call site.
///
/// # Merge laws
///
/// - `table + table` → recursive key-by-key merge (each key independently contested)
/// - `array + array` → higher-priority array replaces lower-priority array entirely
/// - `scalar + scalar` → higher-priority scalar replaces lower-priority scalar entirely
///
/// Priority order (from lowest to highest): defaults < files < env
pub fn deep_merge_traced(
    base: &mut Value,
    overlay: Value,
    layer_id: &str,
    prefix: &str,
    winner_map: &mut WinnerMap,
) {
    match (base, overlay) {
        (Value::Table(base_tbl), Value::Table(overlay_tbl)) => {
            for (key, val) in overlay_tbl {
                let child_path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                match base_tbl.get_mut(&key) {
                    Some(existing) => {
                        deep_merge_traced(existing, val, layer_id, &child_path, winner_map);
                    }
                    None => {
                        record_all_leaves(&val, layer_id, &child_path, winner_map);
                        base_tbl.insert(key, val);
                    }
                }
            }
        }
        (base, overlay) => {
            // Scalar or array replacement: this layer wins the leaf.
            if !prefix.is_empty() {
                winner_map.insert(prefix.to_owned(), layer_id.to_owned());
            }
            *base = overlay;
        }
    }
}

/// Recursively mark every leaf under `value` as won by `layer_id`.
///
/// Called when `overlay` inserts a whole new key into `base` — the entire sub-tree
/// is owned by this layer.
fn record_all_leaves(value: &Value, layer_id: &str, prefix: &str, winner_map: &mut WinnerMap) {
    match value {
        Value::Table(tbl) => {
            for (key, val) in tbl {
                let child = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                record_all_leaves(val, layer_id, &child, winner_map);
            }
        }
        _ => {
            if !prefix.is_empty() {
                winner_map.insert(prefix.to_owned(), layer_id.to_owned());
            }
        }
    }
}

/// Inject a dotted key path into a `toml::Value::Table`, creating intermediate tables
/// as needed.
///
/// `path` is a dot-separated string like `"server.port"`. `value` is inserted at the
/// leaf; any intermediate table that does not exist is created automatically.
///
/// This is used internally by [`crate::Loader::env_prefix`] to convert env-var overrides
/// into TOML tree nodes.
pub(crate) fn set_dotted(root: &mut Value, path: &str, value: Value) {
    let segments: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return;
    }
    set_dotted_recursive(root, &segments, value);
}

fn set_dotted_recursive(root: &mut Value, segments: &[&str], value: Value) {
    if segments.is_empty() {
        return;
    }
    let head = segments[0];
    let tail = &segments[1..];

    let tbl = match root {
        Value::Table(t) => t,
        other => {
            *other = Value::Table(toml::map::Map::new());
            other.as_table_mut().unwrap()
        }
    };

    if tail.is_empty() {
        tbl.insert(head.to_owned(), value);
    } else {
        let entry =
            tbl.entry(head.to_owned()).or_insert_with(|| Value::Table(toml::map::Map::new()));
        set_dotted_recursive(entry, tail, value);
    }
}

/// Best-effort conversion of an env-var string to a typed `toml::Value`.
///
/// Tries bool → integer → float → string in that order, so `"true"` becomes
/// `Value::Boolean(true)`, `"42"` becomes `Value::Integer(42)`, etc.
pub(crate) fn env_str_to_value(s: &str) -> Value {
    if s.eq_ignore_ascii_case("true") {
        return Value::Boolean(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return Value::Boolean(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return Value::Integer(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    Value::String(s.to_owned())
}

#[cfg(test)]
mod tests {
    use toml::Value;

    use super::*;

    fn parse(s: &str) -> Value {
        toml::from_str(s).unwrap()
    }

    #[test]
    fn merges_tables_recursively() {
        let mut base = parse("[a]\nx = 1\ny = 2\n");
        let overlay = parse("[a]\ny = 99\nz = 3\n");
        deep_merge(&mut base, overlay);
        let a = base["a"].as_table().unwrap();
        assert_eq!(a["x"].as_integer(), Some(1)); // preserved
        assert_eq!(a["y"].as_integer(), Some(99)); // overridden
        assert_eq!(a["z"].as_integer(), Some(3)); // added
    }

    #[test]
    fn overlay_replaces_scalar() {
        let mut base = parse("x = 1\n");
        let overlay = parse("x = 42\n");
        deep_merge(&mut base, overlay);
        assert_eq!(base["x"].as_integer(), Some(42));
    }

    #[test]
    fn overlay_replaces_array_entirely() {
        let mut base = parse("arr = [1, 2, 3]\n");
        let overlay = parse("arr = [4, 5]\n");
        deep_merge(&mut base, overlay);
        let arr = base["arr"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_integer(), Some(4));
    }

    #[test]
    fn base_key_absent_in_overlay_preserved() {
        let mut base = parse("keep = \"me\"\nchange = 1\n");
        let overlay = parse("change = 2\n");
        deep_merge(&mut base, overlay);
        assert_eq!(base["keep"].as_str(), Some("me"));
        assert_eq!(base["change"].as_integer(), Some(2));
    }

    #[test]
    fn set_dotted_creates_nested() {
        let mut root = Value::Table(toml::map::Map::new());
        set_dotted(&mut root, "a.b.c", Value::Integer(7));
        assert_eq!(root["a"]["b"]["c"].as_integer(), Some(7));
    }

    #[test]
    fn set_dotted_handles_anomalous_dots() {
        let mut root = Value::Table(toml::map::Map::new());
        set_dotted(&mut root, ".a..b.c.", Value::Integer(42));
        assert_eq!(root["a"]["b"]["c"].as_integer(), Some(42));
    }

    #[test]
    fn env_str_parses_bool() {
        assert_eq!(env_str_to_value("true"), Value::Boolean(true));
        assert_eq!(env_str_to_value("false"), Value::Boolean(false));
        assert_eq!(env_str_to_value("True"), Value::Boolean(true));
    }

    #[test]
    fn env_str_parses_integer() {
        assert_eq!(env_str_to_value("42"), Value::Integer(42));
        assert_eq!(env_str_to_value("-1"), Value::Integer(-1));
    }

    #[test]
    fn env_str_parses_float() {
        assert!(matches!(env_str_to_value("3.14"), Value::Float(_)));
    }

    #[test]
    fn env_str_falls_back_to_string() {
        assert_eq!(env_str_to_value("hello"), Value::String("hello".to_owned()));
    }

    // --- deep_merge_traced ---

    #[test]
    fn traced_merge_records_winning_layers() {
        // BRCE: conservation / provenance
        let mut base = parse("x = 1\n");
        let mut wm = WinnerMap::new();
        deep_merge_traced(&mut base, parse("x = 99\n"), "layer-1", "", &mut wm);
        assert_eq!(wm.get("x").map(String::as_str), Some("layer-1"));
        assert_eq!(base["x"].as_integer(), Some(99));
    }

    #[test]
    fn traced_merge_table_recursive() {
        // BRCE: truth
        let mut base = parse("[a]\nx = 1\ny = 2\n");
        let overlay = parse("[a]\ny = 99\nz = 3\n");
        let mut wm = WinnerMap::new();
        deep_merge_traced(&mut base, overlay, "layer-1", "", &mut wm);
        let a = base["a"].as_table().unwrap();
        // x preserved from layer-0 (not in winner_map from this call)
        assert_eq!(a["x"].as_integer(), Some(1));
        // y overridden by layer-1
        assert_eq!(a["y"].as_integer(), Some(99));
        assert_eq!(wm.get("a.y").map(String::as_str), Some("layer-1"));
        // z newly added by layer-1
        assert_eq!(a["z"].as_integer(), Some(3));
        assert_eq!(wm.get("a.z").map(String::as_str), Some("layer-1"));
    }

    #[test]
    fn traced_merge_array_replaces_not_merges() {
        // BRCE: invariant
        let mut base = parse("arr = [1, 2, 3]\n");
        let overlay = parse("arr = [4, 5]\n");
        let mut wm = WinnerMap::new();
        deep_merge_traced(&mut base, overlay, "layer-1", "", &mut wm);
        let arr = base["arr"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_integer(), Some(4));
        assert_eq!(wm.get("arr").map(String::as_str), Some("layer-1"));
    }

    #[test]
    fn traced_merge_scalar_replacement() {
        // BRCE: invariant
        let mut base = parse("n = 1\n");
        let mut wm = WinnerMap::new();
        deep_merge_traced(&mut base, parse("n = 42\n"), "env", "", &mut wm);
        assert_eq!(base["n"].as_integer(), Some(42));
        assert_eq!(wm.get("n").map(String::as_str), Some("env"));
    }

    #[test]
    fn traced_merge_layer_order_defaults_files_env() {
        // BRCE: determinism
        // defaults < file < env  — later wins
        let mut merged = parse("port = 1\nname = \"default\"\n");
        let mut wm = WinnerMap::new();

        deep_merge_traced(&mut merged, parse("port = 2\n"), "layer-file", "", &mut wm);
        deep_merge_traced(&mut merged, parse("port = 3\n"), "env", "", &mut wm);

        assert_eq!(merged["port"].as_integer(), Some(3));
        assert_eq!(wm.get("port").map(String::as_str), Some("env"));
        // name was not touched by file or env, so not in wm from these calls
        // (it was set before the traced calls)
        assert_eq!(merged["name"].as_str(), Some("default"));
    }

    #[test]
    fn traced_merge_every_field_has_winning_layer() {
        // BRCE: conservation
        // After all layers applied, every leaf in the final value must have a winner.
        let mut merged = Value::Table(toml::map::Map::new());
        let mut wm = WinnerMap::new();

        deep_merge_traced(&mut merged, parse("a = 1\n[s]\nb = 2\n"), "layer-0", "", &mut wm);
        deep_merge_traced(&mut merged, parse("[s]\nc = 3\n"), "layer-1", "", &mut wm);

        // Collect all leaf paths from the merged value
        fn leaves(val: &Value, prefix: &str) -> Vec<String> {
            match val {
                Value::Table(t) => t
                    .iter()
                    .flat_map(|(k, v)| {
                        let p = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{prefix}.{k}")
                        };
                        leaves(v, &p)
                    })
                    .collect(),
                _ => vec![prefix.to_owned()],
            }
        }

        let leaf_paths = leaves(&merged, "");
        for path in &leaf_paths {
            assert!(wm.contains_key(path), "field '{path}' has no winning layer");
        }
    }
}
