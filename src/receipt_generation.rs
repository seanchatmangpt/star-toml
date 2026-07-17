//! In-process BLAKE3 receipt generation for configuration provenance.
//!
//! This module provides functions to generate cryptographic receipts that bind
//! source digests, layer order, environment variable overrides, validation
//! fitness, and canonical config bytes into a single BLAKE3 hash.
//!
//! AC1: BLAKE3 receipt generation (in-process, not shell-out)

use crate::reports::{EnvOverrideReport, LayerReport, SourceReport, blake3_hex};

/// Generates a BLAKE3-based receipt from configuration provenance data.
///
/// The receipt binds together:
/// 1. Source digests (sorted by source_id)
/// 2. Last layer order digest
/// 3. Accepted environment variable entries (sorted)
/// 4. Validation fitness score
/// 5. Canonical configuration bytes
///
/// Returns the BLAKE3 hex digest as a String.
///
/// # Arguments
///
/// * `source_report` - Source digest information
/// * `layer_report` - Layer merge order and digests
/// * `env_report` - Environment variable override report
/// * `validation_fitness` - Fitness score (0.0–1.0) from validation
/// * `canonical_bytes` - The final canonical config bytes
#[must_use]
pub fn generate_receipt(
    source_report: &SourceReport,
    layer_report: &LayerReport,
    env_report: &EnvOverrideReport,
    validation_fitness: f64,
    canonical_bytes: &[u8],
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // 1. Source digests sorted by source_id
    let mut source_entries: Vec<_> = source_report.entries.iter().collect();
    source_entries.sort_by_key(|e| e.source_id);
    let source_part: String = source_entries
        .iter()
        .filter_map(|e| e.digest.as_deref())
        .collect::<Vec<_>>()
        .join(",");
    parts.push(source_part);

    // 2. Last layer_order_digest
    let last_lod = layer_report
        .entries
        .last()
        .map(|e| e.layer_order_digest.as_str())
        .unwrap_or("")
        .to_owned();
    parts.push(last_lod);

    // 3. Accepted env entries sorted
    let mut env_entries: Vec<String> = env_report
        .entries
        .iter()
        .filter(|e| e.accepted)
        .map(|e| format!("{}={}:{}", e.raw_env_key, e.mapped_path, e.raw_value_digest))
        .collect();
    env_entries.sort();
    parts.push(env_entries.join(","));

    // 4. Validation fitness
    parts.push(format!("{:.6}", validation_fitness));

    // 5. Canonical bytes (as hex to avoid embedding binary)
    parts.push(blake3_hex(canonical_bytes));

    // Compute BLAKE3 of all parts joined with |
    let joined = parts.join("|");
    blake3_hex(joined.as_bytes())
}

/// Validates a receipt against the current provenance configuration.
///
/// This function recomputes the receipt and compares it against the provided
/// reference receipt. Both must match byte-for-byte.
///
/// Returns `true` if the receipt is valid, `false` otherwise.
///
/// # Arguments
///
/// * `expected_receipt` - The previously-generated receipt to validate against
/// * `source_report` - Current source digest information
/// * `layer_report` - Current layer merge order and digests
/// * `env_report` - Current environment variable override report
/// * `validation_fitness` - Current fitness score
/// * `canonical_bytes` - Current canonical config bytes
#[must_use]
pub fn validate_receipt(
    expected_receipt: &str,
    source_report: &SourceReport,
    layer_report: &LayerReport,
    env_report: &EnvOverrideReport,
    validation_fitness: f64,
    canonical_bytes: &[u8],
) -> bool {
    let computed = generate_receipt(source_report, layer_report, env_report, validation_fitness, canonical_bytes);
    computed == expected_receipt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::{EnvOverrideEntry, LayerEntry, SourceEntry, SourceKind};

    fn make_test_reports() -> (SourceReport, LayerReport, EnvOverrideReport) {
        use crate::merge::WinnerMap;
        use std::path::PathBuf;

        let source_report = SourceReport {
            entries: vec![SourceEntry {
                source_id: 0,
                source_kind: SourceKind::File,
                label: "test.toml".to_string(),
                path: Some(PathBuf::from("test.toml")),
                required: true,
                found: true,
                digest: Some("abc123".to_string()),
                size_bytes: Some(42),
                source_root: None,
                source_parent: None,
            }],
        };

        let layer_report = LayerReport {
            entries: vec![LayerEntry {
                layer_id: 0,
                layer_name: "layer_0".to_string(),
                priority: 0,
                source_id: 0,
                digest: "layer_digest".to_string(),
                layer_order_digest: "def456".to_string(),
                winning_field_map: WinnerMap::default(),
            }],
        };

        let env_report = EnvOverrideReport {
            prefix: "TEST_".to_string(),
            entries: vec![EnvOverrideEntry {
                raw_env_key: "TEST_KEY".to_string(),
                configured_prefix: "TEST_".to_string(),
                mapped_path: "key".to_string(),
                raw_value_digest: "ghi789".to_string(),
                coerced_type: Some(crate::reports::CoercedType::Str),
                coerced_value_digest: Some("coerced_digest".to_string()),
                accepted: true,
                rejection_code: None,
            }],
        };

        (source_report, layer_report, env_report)
    }

    #[test]
    fn test_generate_receipt_consistency() {
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";
        let fitness = 1.0;

        let receipt1 = generate_receipt(&source, &layer, &env, fitness, canonical);
        let receipt2 = generate_receipt(&source, &layer, &env, fitness, canonical);

        assert_eq!(receipt1, receipt2, "Receipt generation must be deterministic");
        assert!(!receipt1.is_empty(), "Receipt should not be empty");
        assert_eq!(receipt1.len(), 64, "BLAKE3 hex digest should be 64 characters");
    }

    #[test]
    fn test_validate_receipt_valid() {
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";
        let fitness = 1.0;

        let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);
        assert!(validate_receipt(&receipt, &source, &layer, &env, fitness, canonical));
    }

    #[test]
    fn test_validate_receipt_invalid_on_modification() {
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";
        let fitness = 1.0;

        let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);

        // Change canonical bytes — receipt should no longer validate
        let modified = b"name = \"modified\"";
        assert!(
            !validate_receipt(&receipt, &source, &layer, &env, fitness, modified),
            "Receipt should fail validation when canonical bytes change"
        );
    }

    #[test]
    fn test_validate_receipt_invalid_on_fitness_change() {
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";
        let fitness = 1.0;

        let receipt = generate_receipt(&source, &layer, &env, fitness, canonical);

        // Change fitness — receipt should no longer validate
        let modified_fitness = 0.8;
        assert!(
            !validate_receipt(&receipt, &source, &layer, &env, modified_fitness, canonical),
            "Receipt should fail validation when fitness changes"
        );
    }

    #[test]
    fn test_generate_receipt_with_no_env_entries() {
        use crate::merge::WinnerMap;
        use std::path::PathBuf;

        let source_report = SourceReport {
            entries: vec![SourceEntry {
                source_id: 0,
                source_kind: SourceKind::File,
                label: "test.toml".to_string(),
                path: Some(PathBuf::from("test.toml")),
                required: true,
                found: true,
                digest: Some("abc123".to_string()),
                size_bytes: Some(42),
                source_root: None,
                source_parent: None,
            }],
        };

        let layer_report = LayerReport {
            entries: vec![LayerEntry {
                layer_id: 0,
                layer_name: "layer_0".to_string(),
                priority: 0,
                source_id: 0,
                digest: "layer_digest".to_string(),
                layer_order_digest: "def456".to_string(),
                winning_field_map: WinnerMap::default(),
            }],
        };

        // Empty env report
        let env_report = EnvOverrideReport {
            prefix: "TEST_".to_string(),
            entries: vec![],
        };

        let canonical = b"name = \"test\"";
        let receipt = generate_receipt(&source_report, &layer_report, &env_report, 1.0, canonical);

        assert!(!receipt.is_empty());
        assert_eq!(receipt.len(), 64);
    }
}
