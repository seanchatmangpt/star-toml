//! Source, layer, and env-override provenance reports for config admission.
//!
//! Designed so a future OCEL lifecycle-history export can consume these structures
//! directly — every field is stable and serializable.

use std::path::PathBuf;

use crate::merge::WinnerMap;

// ---------------------------------------------------------------------------
// Source provenance
// ---------------------------------------------------------------------------

/// How a source was provided to the loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    /// A literal TOML string embedded in code (built-in defaults).
    Str,
    /// A required file on disk — missing file is a load error.
    File,
    /// An optional file on disk — missing file is recorded but not fatal.
    OptionalFile,
    /// File discovered by walking parent directories — missing is non-fatal.
    FindFile,
}

/// One config source discovered during a load operation.
#[derive(Debug, Clone)]
pub struct SourceEntry {
    /// Monotonic 0-based index within this load operation.
    pub source_id: usize,
    /// How the source was specified to the loader.
    pub source_kind: SourceKind,
    /// Human-readable label (e.g., `"built-in defaults"` or a file path).
    pub label: String,
    /// Resolved filesystem path, if file-based.
    pub path: Option<PathBuf>,
    /// `true` if a missing source is a hard error.
    pub required: bool,
    /// `true` if the source was found and loaded successfully.
    pub found: bool,
    /// BLAKE3 hex digest of the raw source bytes, present when `found`.
    pub digest: Option<String>,
    /// Byte length of the source, present when `found`.
    pub size_bytes: Option<u64>,
    /// Directory that contains the source file (`path.parent()`), if applicable.
    pub source_root: Option<PathBuf>,
    /// Grandparent of the source file (`path.parent().parent()`), if applicable.
    pub source_parent: Option<PathBuf>,
}

/// All sources registered during a single load operation, in registration order.
#[derive(Debug, Clone, Default)]
pub struct SourceReport {
    pub entries: Vec<SourceEntry>,
}

impl SourceReport {
    /// `true` if any required source was not found.
    #[must_use]
    pub fn has_missing_required(&self) -> bool {
        self.entries.iter().any(|e| e.required && !e.found)
    }

    /// Iterate over entries that were found and loaded.
    pub fn found_entries(&self) -> impl Iterator<Item = &SourceEntry> {
        self.entries.iter().filter(|e| e.found)
    }

    /// Iterate over optional entries that were not found.
    pub fn missing_optional_entries(&self) -> impl Iterator<Item = &SourceEntry> {
        self.entries.iter().filter(|e| !e.required && !e.found)
    }
}

// ---------------------------------------------------------------------------
// Layer merge provenance
// ---------------------------------------------------------------------------

/// One merge layer applied during loading.
#[derive(Debug, Clone)]
pub struct LayerEntry {
    /// Monotonic 0-based index in merge order (0 = lowest priority).
    pub layer_id: usize,
    /// Human-readable name for this layer.
    pub layer_name: String,
    /// Merge priority: lower = applied earlier = lower priority.
    pub priority: usize,
    /// The `SourceEntry::source_id` that produced this layer's content.
    pub source_id: usize,
    /// BLAKE3 hex digest of this layer's raw TOML content.
    pub digest: String,
    /// BLAKE3 hex of all layer digests concatenated in merge order — a merge-order witness.
    pub layer_order_digest: String,
    /// Every leaf field path this layer wrote, mapped to this layer's `layer_id` string.
    ///
    /// Only fields actually touched by this overlay are present. Use the global
    /// `BoundedSources::global_winner_map` for the final cumulative state.
    pub winning_field_map: WinnerMap,
}

/// All merge layers applied during a single load operation, in merge order.
#[derive(Debug, Clone, Default)]
pub struct LayerReport {
    pub entries: Vec<LayerEntry>,
}

// ---------------------------------------------------------------------------
// Env-override provenance
// ---------------------------------------------------------------------------

/// Type a raw env-var string was coerced to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercedType {
    Bool,
    Integer,
    Float,
    Str,
}

impl std::fmt::Display for CoercedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::Str => write!(f, "str"),
        }
    }
}

/// One env-var override considered during loading.
///
/// Only variables that match the configured `env_prefix` appear here.
/// Ambient OS variables (`PATH`, `HOME`, `SHELL`, etc.) that do not match
/// the prefix are never recorded.
#[derive(Debug, Clone)]
pub struct EnvOverrideEntry {
    /// The raw env var key (e.g., `APP_SERVER__PORT`).
    pub raw_env_key: String,
    /// The prefix that was matched against (e.g., `APP_`).
    pub configured_prefix: String,
    /// The TOML path this var maps to (e.g., `server.port`). Empty string = rejected.
    pub mapped_path: String,
    /// BLAKE3 hex digest of the raw env value string.
    pub raw_value_digest: String,
    /// Coerced TOML type, present if accepted.
    pub coerced_type: Option<CoercedType>,
    /// BLAKE3 hex digest of the coerced value's string representation, present if accepted.
    pub coerced_value_digest: Option<String>,
    /// `true` if this override was applied to the merged config.
    pub accepted: bool,
    /// Machine-readable rejection code, present if `!accepted`.
    pub rejection_code: Option<String>,
}

/// All env-var overrides considered during a single load operation.
///
/// Only variables that matched the configured prefix are recorded.
#[derive(Debug, Clone, Default)]
pub struct EnvOverrideReport {
    /// The prefix that was used to filter variables.
    pub prefix: String,
    /// All matching entries, in OS iteration order.
    pub entries: Vec<EnvOverrideEntry>,
}

impl EnvOverrideReport {
    /// Iterate over accepted overrides.
    pub fn accepted(&self) -> impl Iterator<Item = &EnvOverrideEntry> {
        self.entries.iter().filter(|e| e.accepted)
    }

    /// Iterate over rejected overrides.
    pub fn rejected(&self) -> impl Iterator<Item = &EnvOverrideEntry> {
        self.entries.iter().filter(|e| !e.accepted)
    }
}

// ---------------------------------------------------------------------------
// BLAKE3 helper
// ---------------------------------------------------------------------------

/// Compute the BLAKE3 hex digest of `data`.
#[must_use]
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}
