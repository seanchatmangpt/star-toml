//! OCEL export adapter for star-toml lifecycle events.
//!
//! Compiled only when the `wasm4pm-compat` optional feature is enabled.
//! Without the feature, the module exposes a no-op stub so the crate
//! compiles without the dependency.
//!
//! Future work: map SourceReport / LayerReport / EnvOverrideReport directly
//! to OCEL objects and events for full lifecycle-history export.
//!
//! Dependency direction: star-toml → wasm4pm-compat (OCEL types)
//! The reverse (wasm4pm → star-toml) is handled by the wasm4pm crate.

#[cfg(feature = "wasm4pm-compat")]
mod ocel_impl {
    use wasm4pm_compat::ocel::{EventObjectLink, Object, OcelAttribute, OcelEvent, OcelLog};

    use crate::events::AdmissionEvent;

    /// Maps a list of star-toml admission events to a wasm4pm-compat OCEL Log.
    #[must_use]
    pub fn export_events_to_ocel(events: &[AdmissionEvent]) -> OcelLog {
        let mut ocel_events = Vec::new();
        let mut ocel_objects = std::collections::HashMap::new();
        let mut e2o_links = Vec::new();

        for ev in events {
            let mut ocel_ev = OcelEvent::new(&ev.event_id, &ev.event_kind.to_string())
                .at_ns(ev.timestamp_or_sequence);

            for (k, v) in &ev.attributes {
                ocel_ev = ocel_ev.with_attribute(OcelAttribute::string(k, v));
            }

            let run_obj_id = format!("run_{}", ev.run_id);
            if !ocel_objects.contains_key(&run_obj_id) {
                ocel_objects.insert(
                    run_obj_id.clone(),
                    Object::new(&run_obj_id, "ConfigRun")
                        .with_attribute(OcelAttribute::string("run_id", &ev.run_id)),
                );
            }
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

                if !ocel_objects.contains_key(obj_id) {
                    ocel_objects.insert(obj_id.clone(), Object::new(obj_id, obj_type));
                }
                e2o_links
                    .push(EventObjectLink::new(&ev.event_id, obj_id).qualified("relates_to"));
            }

            ocel_events.push(ocel_ev);
        }

        let objects: Vec<Object> = ocel_objects.into_values().collect();
        OcelLog::new(objects, ocel_events, e2o_links, Vec::new(), Vec::new())
    }
}

#[cfg(feature = "wasm4pm-compat")]
pub use ocel_impl::export_events_to_ocel;

/// Stub: OCEL export requires the `wasm4pm-compat` feature.
///
/// Enable with `star-toml = { features = ["wasm4pm-compat"] }`.
#[cfg(not(feature = "wasm4pm-compat"))]
pub fn export_events_to_ocel(_events: &[crate::events::AdmissionEvent]) {
    // No-op stub. Enable the wasm4pm-compat feature for full OCEL lifecycle-history export.
}
