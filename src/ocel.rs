//! OCEL export adapter for star-toml lifecycle events.
//!
//! Maps [`AdmissionEvent`] records to a `wasm4pm_compat` OCEL log.
//! This module records **lifecycle history only** — it does not compute
//! `q_config` or grant admission standing.
//!
//! Dependency direction: `star-toml → wasm4pm-compat`
//! The reverse (`wasm4pm → star-toml`) is handled by the `wasm4pm` crate.

use std::collections::HashMap;

use wasm4pm_compat::ocel::{EventObjectLink, Object, OcelAttribute, OcelEvent, OcelLog};

use crate::events::AdmissionEvent;

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
