//! Evidence gate — validates configuration state transitions using BLAKE3 receipts.
//!
//! The evidence gate enforces that configuration changes are auditable by:
//! 1. Generating a BLAKE3 receipt after evidence gathering
//! 2. Validating receipts against current provenance state
//! 3. Preventing unauthorized state transitions
//!
//! AC5: Evidence gate uses BLAKE3 receipts

use crate::receipt_generation::{generate_receipt, validate_receipt};
use crate::reports::{EnvOverrideReport, LayerReport, SourceReport};

/// A gate that guards evidence-based configuration validation.
///
/// The gate maintains a receipt that certifies the configuration state
/// at a particular point in time, and can validate that the current state
/// matches the recorded receipt.
#[derive(Debug, Clone)]
pub struct EvidenceGate {
    /// BLAKE3 receipt from the last successful gate passage
    receipt: Option<String>,
}

impl EvidenceGate {
    /// Creates a new, empty evidence gate.
    #[must_use]
    pub fn new() -> Self {
        Self { receipt: None }
    }

    /// Advances through the gate after gathering evidence (provenance reports).
    ///
    /// Generates a new BLAKE3 receipt that binds:
    /// - Source digests
    /// - Layer merge order
    /// - Environment variable overrides
    /// - Validation fitness
    /// - Canonical config bytes
    ///
    /// Returns the generated receipt for audit trail recording.
    pub fn pass_evidence(
        &mut self,
        source_report: &SourceReport,
        layer_report: &LayerReport,
        env_report: &EnvOverrideReport,
        validation_fitness: f64,
        canonical_bytes: &[u8],
    ) -> String {
        let receipt = generate_receipt(source_report, layer_report, env_report, validation_fitness, canonical_bytes);
        self.receipt = Some(receipt.clone());
        receipt
    }

    /// Checks if the gate is locked (has a receipt on record).
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.receipt.is_some()
    }

    /// Returns the current receipt, if one has been recorded.
    #[must_use]
    pub fn receipt(&self) -> Option<&str> {
        self.receipt.as_deref()
    }

    /// Validates that the current provenance state matches the recorded receipt.
    ///
    /// Returns `true` if the gate is unlocked (no receipt) or if the receipt
    /// validates against the current state. Returns `false` if the receipt exists
    /// but does not validate (indicating a state change or tampering).
    pub fn validate(
        &self,
        source_report: &SourceReport,
        layer_report: &LayerReport,
        env_report: &EnvOverrideReport,
        validation_fitness: f64,
        canonical_bytes: &[u8],
    ) -> bool {
        match self.receipt.as_deref() {
            None => true, // Unlocked gate always validates
            Some(receipt) => validate_receipt(receipt, source_report, layer_report, env_report, validation_fitness, canonical_bytes),
        }
    }

    /// Clears the recorded receipt, unlocking the gate.
    pub fn reset(&mut self) {
        self.receipt = None;
    }
}

impl Default for EvidenceGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::{EnvOverrideEntry, LayerEntry, SourceEntry, SourceKind};

    fn make_test_reports() -> (SourceReport, LayerReport, EnvOverrideReport) {
        use crate::reports::CoercedType;
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
                coerced_type: Some(CoercedType::Str),
                coerced_value_digest: Some("coerced_digest".to_string()),
                accepted: true,
                rejection_code: None,
            }],
        };

        (source_report, layer_report, env_report)
    }

    #[test]
    fn test_gate_new_is_unlocked() {
        let gate = EvidenceGate::new();
        assert!(!gate.is_locked());
        assert!(gate.receipt().is_none());
    }

    #[test]
    fn test_gate_unlocked_always_validates() {
        let gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"test";

        // Unlocked gate should always validate
        assert!(gate.validate(&source, &layer, &env, 1.0, canonical));
    }

    #[test]
    fn test_gate_pass_evidence_locks_gate() {
        let mut gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";

        let receipt = gate.pass_evidence(&source, &layer, &env, 1.0, canonical);

        assert!(gate.is_locked());
        assert_eq!(gate.receipt(), Some(receipt.as_str()));
        assert!(!receipt.is_empty());
    }

    #[test]
    fn test_gate_locked_validates_same_state() {
        let mut gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";

        gate.pass_evidence(&source, &layer, &env, 1.0, canonical);

        // Same state should validate
        assert!(gate.validate(&source, &layer, &env, 1.0, canonical));
    }

    #[test]
    fn test_gate_locked_rejects_modified_canonical() {
        let mut gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";

        gate.pass_evidence(&source, &layer, &env, 1.0, canonical);

        // Modified canonical bytes should fail validation
        let modified = b"name = \"modified\"";
        assert!(!gate.validate(&source, &layer, &env, 1.0, modified));
    }

    #[test]
    fn test_gate_locked_rejects_modified_fitness() {
        let mut gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";

        gate.pass_evidence(&source, &layer, &env, 1.0, canonical);

        // Modified fitness should fail validation
        assert!(!gate.validate(&source, &layer, &env, 0.9, canonical));
    }

    #[test]
    fn test_gate_reset_unlocks() {
        let mut gate = EvidenceGate::new();
        let (source, layer, env) = make_test_reports();
        let canonical = b"name = \"test\"";

        gate.pass_evidence(&source, &layer, &env, 1.0, canonical);
        assert!(gate.is_locked());

        gate.reset();
        assert!(!gate.is_locked());

        // After reset, gate should accept any state
        assert!(gate.validate(&source, &layer, &env, 0.5, b"anything"));
    }

    #[test]
    fn test_gate_default_creates_unlocked() {
        let gate = EvidenceGate::default();
        assert!(!gate.is_locked());
    }
}
