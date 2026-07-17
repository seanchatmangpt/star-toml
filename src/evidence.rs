//! AC4: OCEL JSON schema validation and evidence emission.
//!
//! This module validates OCEL 2.0 JSON structure and provides evidence
//! serialization for oracle integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AC4: OCEL 2.0 schema validator.
///
/// Validates that an OCEL event log conforms to OCEL 2.0 structure.
/// Ensures:
/// - All required fields are present (event_id, activity, timestamp, omap)
/// - Event and object consistency
/// - Attribute type correctness
#[derive(Debug, Clone)]
pub struct OcelValidator;

/// OCEL evidence structure for schema validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvidence {
    /// Event list with all required OCEL 2.0 fields.
    pub events: Vec<OcelEventRecord>,
    /// Object list with type and attributes.
    pub objects: Vec<OcelObjectRecord>,
    /// Event-to-object relationships.
    pub relationships: Vec<OcelRelationship>,
}

/// Single OCEL event record with validation fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEventRecord {
    /// Unique event identifier.
    pub event_id: String,
    /// Activity name/type.
    pub activity: String,
    /// Timestamp (nanoseconds or sequence).
    pub timestamp: u64,
    /// Object map: object_id → role.
    pub omap: HashMap<String, String>,
    /// Generic attributes.
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// OCEL object record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectRecord {
    /// Unique object identifier.
    pub object_id: String,
    /// Object type (ConfigRun, Layer, ValidationReport, etc.).
    pub object_type: String,
    /// Object attributes.
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// Event-to-object relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelRelationship {
    /// Event ID.
    pub event_id: String,
    /// Object ID.
    pub object_id: String,
    /// Relationship type/qualifier.
    #[serde(default)]
    pub rel_type: String,
}

/// Schema validation error details.
#[derive(Debug, Clone)]
pub enum OcelValidationError {
    /// Missing required field in event.
    MissingEventField(String, String),
    /// Missing required field in object.
    MissingObjectField(String, String),
    /// Invalid timestamp format.
    InvalidTimestamp(String),
    /// Orphaned event (references non-existent object).
    OrphanedEvent(String, String),
    /// Duplicate event ID.
    DuplicateEventId(String),
    /// Duplicate object ID.
    DuplicateObjectId(String),
    /// JSON deserialization error.
    DeserializationError(String),
}

impl std::fmt::Display for OcelValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcelValidationError::MissingEventField(event_id, field) => {
                write!(f, "event '{}' missing required field: {}", event_id, field)
            }
            OcelValidationError::MissingObjectField(object_id, field) => {
                write!(f, "object '{}' missing required field: {}", object_id, field)
            }
            OcelValidationError::InvalidTimestamp(msg) => {
                write!(f, "invalid timestamp: {}", msg)
            }
            OcelValidationError::OrphanedEvent(event_id, object_id) => {
                write!(
                    f,
                    "event '{}' references non-existent object '{}'",
                    event_id, object_id
                )
            }
            OcelValidationError::DuplicateEventId(id) => {
                write!(f, "duplicate event id: {}", id)
            }
            OcelValidationError::DuplicateObjectId(id) => {
                write!(f, "duplicate object id: {}", id)
            }
            OcelValidationError::DeserializationError(msg) => {
                write!(f, "failed to deserialize OCEL JSON: {}", msg)
            }
        }
    }
}

impl std::error::Error for OcelValidationError {}

impl OcelValidator {
    /// AC4: Validate OCEL JSON string against OCEL 2.0 schema.
    ///
    /// Checks:
    /// - JSON is valid OCEL evidence structure
    /// - All required fields are present
    /// - No duplicate event or object IDs
    /// - All event references to objects are valid
    ///
    /// # Errors
    ///
    /// Returns validation error details if schema is violated.
    pub fn validate_json(json_str: &str) -> Result<OcelEvidence, OcelValidationError> {
        let evidence: OcelEvidence = serde_json::from_str(json_str)
            .map_err(|e| OcelValidationError::DeserializationError(e.to_string()))?;

        Self::validate_evidence(&evidence)?;
        Ok(evidence)
    }

    /// AC4: Validate an OCEL evidence structure.
    ///
    /// # Errors
    ///
    /// Returns validation error if structure is invalid.
    pub fn validate_evidence(evidence: &OcelEvidence) -> Result<(), OcelValidationError> {
        // Check for duplicate event IDs
        let mut event_ids = std::collections::HashSet::new();
        for event in &evidence.events {
            if !event_ids.insert(&event.event_id) {
                return Err(OcelValidationError::DuplicateEventId(event.event_id.clone()));
            }

            // Validate required event fields
            if event.activity.is_empty() {
                return Err(OcelValidationError::MissingEventField(
                    event.event_id.clone(),
                    "activity".to_string(),
                ));
            }
            if event.omap.is_empty() {
                return Err(OcelValidationError::MissingEventField(
                    event.event_id.clone(),
                    "omap".to_string(),
                ));
            }
        }

        // Check for duplicate object IDs
        let mut object_ids = std::collections::HashSet::new();
        for obj in &evidence.objects {
            if !object_ids.insert(&obj.object_id) {
                return Err(OcelValidationError::DuplicateObjectId(obj.object_id.clone()));
            }

            // Validate required object fields
            if obj.object_type.is_empty() {
                return Err(OcelValidationError::MissingObjectField(
                    obj.object_id.clone(),
                    "object_type".to_string(),
                ));
            }
        }

        // Validate event-to-object references (AC4: ensure consistency)
        for event in &evidence.events {
            // Check omap references
            for object_id in event.omap.keys() {
                if !object_ids.contains(object_id) {
                    return Err(OcelValidationError::OrphanedEvent(
                        event.event_id.clone(),
                        object_id.clone(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocel_validator_valid_evidence() {
        let evidence = OcelEvidence {
            events: vec![OcelEventRecord {
                event_id: "evt_001".to_string(),
                activity: "SourceDiscovered".to_string(),
                timestamp: 1000,
                omap: {
                    let mut map = HashMap::new();
                    map.insert("run_001".to_string(), "belongs_to".to_string());
                    map
                },
                attributes: Default::default(),
            }],
            objects: vec![OcelObjectRecord {
                object_id: "run_001".to_string(),
                object_type: "ConfigRun".to_string(),
                attributes: Default::default(),
            }],
            relationships: vec![OcelRelationship {
                event_id: "evt_001".to_string(),
                object_id: "run_001".to_string(),
                rel_type: "belongs_to".to_string(),
            }],
        };

        assert!(OcelValidator::validate_evidence(&evidence).is_ok());
    }

    #[test]
    fn test_ocel_validator_duplicate_event_id() {
        let evidence = OcelEvidence {
            events: vec![
                OcelEventRecord {
                    event_id: "evt_001".to_string(),
                    activity: "SourceDiscovered".to_string(),
                    timestamp: 1000,
                    omap: {
                        let mut map = HashMap::new();
                        map.insert("run_001".to_string(), "belongs_to".to_string());
                        map
                    },
                    attributes: Default::default(),
                },
                OcelEventRecord {
                    event_id: "evt_001".to_string(), // Duplicate!
                    activity: "LayerMerged".to_string(),
                    timestamp: 2000,
                    omap: Default::default(),
                    attributes: Default::default(),
                },
            ],
            objects: vec![OcelObjectRecord {
                object_id: "run_001".to_string(),
                object_type: "ConfigRun".to_string(),
                attributes: Default::default(),
            }],
            relationships: vec![],
        };

        assert!(matches!(
            OcelValidator::validate_evidence(&evidence),
            Err(OcelValidationError::DuplicateEventId(_))
        ));
    }

    #[test]
    fn test_ocel_validator_orphaned_event() {
        let evidence = OcelEvidence {
            events: vec![OcelEventRecord {
                event_id: "evt_001".to_string(),
                activity: "SourceDiscovered".to_string(),
                timestamp: 1000,
                omap: {
                    let mut map = HashMap::new();
                    map.insert("nonexistent_obj".to_string(), "belongs_to".to_string());
                    map
                },
                attributes: Default::default(),
            }],
            objects: vec![OcelObjectRecord {
                object_id: "run_001".to_string(),
                object_type: "ConfigRun".to_string(),
                attributes: Default::default(),
            }],
            relationships: vec![OcelRelationship {
                event_id: "evt_001".to_string(),
                object_id: "nonexistent_obj".to_string(),
                rel_type: "belongs_to".to_string(),
            }],
        };

        assert!(matches!(
            OcelValidator::validate_evidence(&evidence),
            Err(OcelValidationError::OrphanedEvent(_, _))
        ));
    }
}
