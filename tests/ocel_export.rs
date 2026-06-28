//! Test suite for validating the wasm4pm-compat OCEL export mapping and adapter integration.
//! wasm4pm-compat is a required dependency; no feature flag is needed.

use star_toml::events::{AdmissionEvent, ConfigEventKind};
use star_toml::ocel::export_events_to_ocel;

#[test]
fn test_ocel_export_does_not_compute_q() {
    // The OCEL export should only represent the audit process log, without computing or defining q_config.
    let event = AdmissionEvent::new(
        "run123",
        "evt_001",
        1,
        ConfigEventKind::SourceDiscovered,
        vec!["source_file.toml".to_string()],
        vec![("path".to_string(), "config.toml".to_string())],
    );

    let log = export_events_to_ocel(&[event]);
    
    // Validate that no q_config attribute exists on the objects or events (ensuring no q_config is computed or defined here).
    for obj in log.objects() {
        assert!(obj.attributes().iter().all(|attr| attr.key != "q_config"));
    }
    for ev in log.events() {
        assert!(ev.attributes().iter().all(|attr| attr.key != "q_config"));
    }
}

#[test]
fn test_lifecycle_events_export_to_ocel() {
    let events = vec![
        AdmissionEvent::new(
            "run123",
            "evt_001",
            1,
            ConfigEventKind::SourceDiscovered,
            vec!["source_config.toml".to_string()],
            vec![("source".to_string(), "main".to_string())],
        ),
        AdmissionEvent::new(
            "run123",
            "evt_002",
            2,
            ConfigEventKind::LayerMerged,
            vec!["layer_main".to_string()],
            vec![("strategy".to_string(), "recursive".to_string())],
        ),
    ];

    let log = export_events_to_ocel(&events);

    assert_eq!(log.events().len(), 2);
    assert_eq!(log.events()[0].activity(), "SourceDiscovered");
    assert_eq!(log.events()[1].activity(), "LayerMerged");
    
    // Check ConfigRun object is mapped
    let run_obj_found = log.objects().iter().any(|obj| obj.object_type() == "ConfigRun");
    assert!(run_obj_found);
}

#[test]
fn test_ocel_export_preserves_event_order() {
    let events = vec![
        AdmissionEvent::new(
            "run_999",
            "first_id",
            10,
            ConfigEventKind::SourceDiscovered,
            vec![],
            vec![],
        ),
        AdmissionEvent::new(
            "run_999",
            "second_id",
            20,
            ConfigEventKind::ConfigValidated,
            vec![],
            vec![],
        ),
    ];

    let log = export_events_to_ocel(&events);
    assert_eq!(log.events().len(), 2);
    assert_eq!(log.events()[0].id(), "first_id");
    assert_eq!(log.events()[0].timestamp_ns(), Some(10));
    assert_eq!(log.events()[1].id(), "second_id");
    assert_eq!(log.events()[1].timestamp_ns(), Some(20));
}

#[test]
fn test_ocel_export_has_config_run_object() {
    let event = AdmissionEvent::new(
        "my_test_run",
        "evt_010",
        100,
        ConfigEventKind::CanonicalSaved,
        vec!["canonical_output.toml".to_string()],
        vec![],
    );

    let log = export_events_to_ocel(&[event]);
    let run_obj = log.objects().iter().find(|obj| obj.object_type() == "ConfigRun").expect("ConfigRun object should exist");
    assert_eq!(run_obj.id(), "run_my_test_run");
}
