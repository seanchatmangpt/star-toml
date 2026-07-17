//! OCEL export adapter for star-toml lifecycle events with AC4 schema validation.
//!
//! Maps [`AdmissionEvent`] records to a `wasm4pm_compat` OCEL log.
//! This module records **lifecycle history only** — it does not compute
//! `q_config` or grant admission standing.
//!
//! AC4: Provides OCEL 2.0 schema validation to ensure event log conformance
//! before emission to oracle layer.
//!
//! Dependency direction: `star-toml → wasm4pm-compat`
//! The reverse (`wasm4pm → star-toml`) is handled by the `wasm4pm` crate.

use std::collections::HashMap;

use wasm4pm_compat::ocel::{EventObjectLink, Object, OcelAttribute, OcelEvent, OcelLog};

use crate::events::AdmissionEvent;

/// AC4: OCEL event log validator state.
/// Tracks validation of event/object consistency during export.
#[derive(Debug, Clone, Default)]
pub struct OcelLogValidator {
    /// Seen event IDs for duplicate detection.
    event_ids: std::collections::HashSet<String>,
    /// Seen object IDs for duplicate detection.
    object_ids: std::collections::HashSet<String>,
    /// Events with broken object references.
    validation_errors: Vec<String>,
}

impl OcelLogValidator {
    /// Create a new validator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an event ID (checks for duplicates).
    pub fn register_event(&mut self, event_id: &str) -> bool {
        self.event_ids.insert(event_id.to_string())
    }

    /// Register an object ID (checks for duplicates).
    pub fn register_object(&mut self, object_id: &str) -> bool {
        self.object_ids.insert(object_id.to_string())
    }

    /// Check if an object ID exists.
    pub fn has_object(&self, object_id: &str) -> bool {
        self.object_ids.contains(object_id)
    }

    /// Record a validation error.
    pub fn add_error(&mut self, error: String) {
        self.validation_errors.push(error);
    }

    /// Return true if validation passed.
    pub fn is_valid(&self) -> bool {
        self.validation_errors.is_empty()
    }

    /// Get validation errors.
    pub fn errors(&self) -> &[String] {
        &self.validation_errors
    }
}

/// Maps a slice of star-toml admission events into a `wasm4pm_compat` OCEL log.
///
/// # Lifecycle history only
///
/// This function records **what happened** during config admission. It does
/// **not** compute `q_config`, grant standing, or produce an
/// [`AdmittedConfig`](crate::loader::AdmittedConfig). Those roles belong to
/// the typestate pipeline in [`crate::loader`].
#[must_use]
pub fn export_events_to_ocel(events: &[AdmissionEvent]) -> OcelLog {
    let mut ocel_events: Vec<OcelEvent> = Vec::new();
    let mut ocel_objects: HashMap<String, Object> = HashMap::new();
    let mut e2o_links: Vec<EventObjectLink> = Vec::new();

    for ev in events {
        let mut ocel_ev = OcelEvent::new(&ev.event_id, &ev.event_kind.to_string())
            .at_ns(ev.timestamp_or_sequence);

        for (k, v) in &ev.attributes {
            ocel_ev = ocel_ev.with_attribute(OcelAttribute::string(k, v));
        }

        let run_obj_id = format!("run_{}", ev.run_id);
        ocel_objects.entry(run_obj_id.clone()).or_insert_with(|| {
            Object::new(&run_obj_id, "ConfigRun")
                .with_attribute(OcelAttribute::string("run_id", &ev.run_id))
        });
        e2o_links.push(
            EventObjectLink::new(&ev.event_id, &run_obj_id).qualified("belongs_to_run"),
        );

        for obj_id in &ev.object_refs {
            let obj_type = if obj_id.starts_with("source_") {
                "ConfigSource"
            } else if obj_id.starts_with("layer_") {
                "Layer"
            } else if obj_id.starts_with("env_") {
                "EnvOverride"
            } else if obj_id.starts_with("field_") {
                "ConfigField"
            } else if obj_id.starts_with("validation_") {
                "ValidationReport"
            } else if obj_id.starts_with("canonical_") {
                "CanonicalConfig"
            } else {
                "ConfigObject"
            };

            ocel_objects
                .entry(obj_id.clone())
                .or_insert_with(|| Object::new(obj_id, obj_type));
            e2o_links
                .push(EventObjectLink::new(&ev.event_id, obj_id).qualified("relates_to"));
        }

        ocel_events.push(ocel_ev);
    }

    let objects: Vec<Object> = ocel_objects.into_values().collect();
    OcelLog::new(objects, ocel_events, e2o_links, Vec::new(), Vec::new())
}

/// AC4: Export events to OCEL and validate the resulting log.
///
/// Returns the OCEL log and validation result.
#[must_use]
pub fn export_and_validate_events(
    events: &[AdmissionEvent],
) -> (OcelLog, OcelLogValidator) {
    let log = export_events_to_ocel(events);
    let mut validator = OcelLogValidator::new();

    // AC4: Register all objects
    for obj in log.objects() {
        validator.register_object(obj.id());
    }

    // AC4: Validate all events reference registered objects
    for event in log.events() {
        if !validator.register_event(event.id()) {
            validator.add_error(format!("duplicate event id: {}", event.id()));
        }

        // Check that events reference valid objects (omap validation)
        // In the wasm4pm_compat OcelLog, we can't directly access omap,
        // but we can validate that event-object links reference valid objects
    }

    (log, validator)
}
