//! Demonstrates one-way export of star-toml lifecycle events to wasm4pm-compat OCEL.
//!
//! Run with: `cargo run --example ocel_lifecycle_export`

use star_toml::events::{AdmissionEvent, ConfigEventKind};
use star_toml::export_events_to_ocel;

fn main() {
    println!("--- OCEL Lifecycle Export Example ---");

    // 1. Create a log of core lifecycle admission events
    let events = vec![
        AdmissionEvent::new(
            "run_example_123",
            "evt_001",
            1000,
            ConfigEventKind::SourceDiscovered,
            vec!["source_service.toml".to_string()],
            vec![("path".to_string(), "examples/config_patterns/service.toml".to_string())],
        ),
        AdmissionEvent::new(
            "run_example_123",
            "evt_002",
            1005,
            ConfigEventKind::ConfigValidated,
            vec!["validation_report_ok".to_string()],
            vec![],
        ),
    ];

    // 2. Export the events to a wasm4pm-compat OCEL log
    // DfCM Invariant: OCEL is lifecycle history only; it does not grant or calculate q_config.
    let ocel_log = export_events_to_ocel(&events);

    println!("Successfully exported {} events to OCEL Log:", ocel_log.events().len());
    for event in ocel_log.events() {
        println!(
            "  - Event ID: '{}', Activity: '{}', Timestamp: {:?}",
            event.id(),
            event.activity(),
            event.timestamp_ns()
        );
    }
}
