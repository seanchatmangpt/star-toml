//! Minimal dependency-free lifecycle event model representing config lifecycle history.

/// Event kinds representing configuration lifecycle stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigEventKind {
    /// Config source file or environment prefix discovered.
    SourceDiscovered,
    /// Overlays and defaults merged.
    LayerMerged,
    /// Environment variables resolved/applied.
    EnvResolved,
    /// Merged config deserialized into typed struct.
    ConfigDeserialized,
    /// Semantic validation checked on typed config.
    ConfigValidated,
    /// Canonical sorted TOML written to disk/string.
    CanonicalSaved,
}

impl std::fmt::Display for ConfigEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigEventKind::SourceDiscovered => write!(f, "SourceDiscovered"),
            ConfigEventKind::LayerMerged => write!(f, "LayerMerged"),
            ConfigEventKind::EnvResolved => write!(f, "EnvResolved"),
            ConfigEventKind::ConfigDeserialized => write!(f, "ConfigDeserialized"),
            ConfigEventKind::ConfigValidated => write!(f, "ConfigValidated"),
            ConfigEventKind::CanonicalSaved => write!(f, "CanonicalSaved"),
        }
    }
}

/// A structured lifecycle admission event representing configuration process history.
#[derive(Debug, Clone)]
pub struct AdmissionEvent {
    /// Identity of the configuration admission run.
    pub run_id: String,
    /// Identity of this specific event.
    pub event_id: String,
    /// Deterministic sequence number or timestamp.
    pub timestamp_or_sequence: u64,
    /// The event activity/kind.
    pub event_kind: ConfigEventKind,
    /// Associated object IDs (e.g. ConfigRun, ConfigSource, Layer, etc.)
    pub object_refs: Vec<String>,
    /// Generic string-based attribute annotations.
    pub attributes: Vec<(String, String)>,
}

impl AdmissionEvent {
    /// Create a new admission event.
    pub fn new(
        run_id: &str,
        event_id: &str,
        seq: u64,
        kind: ConfigEventKind,
        object_refs: Vec<String>,
        attributes: Vec<(String, String)>,
    ) -> Self {
        Self {
            run_id: run_id.to_owned(),
            event_id: event_id.to_owned(),
            timestamp_or_sequence: seq,
            event_kind: kind,
            object_refs,
            attributes,
        }
    }
}
