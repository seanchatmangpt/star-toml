//! Demonstrates the Design for Combinatorial Maximalism (DfCM) axes matrix.
//!
//! Run with: `cargo run --example dfcm_axes_matrix`

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

/// The DfCM Matrix maps the complete variant space:
///
/// | Axis | Variants |
/// | --- | --- |
/// | **Source** | inline | required file | optional file | discovered file |
/// | **Layer** | defaults | base | profile | env |
/// | **Environment** | ignored ambient | prefixed admitted override | invalid prefixed override |
/// | **Type** | scalar | table | array | array-of-tables |
/// | **Validation** | pass | error | fatal |
/// | **PathPolicy** | sandbox | relative-only | forbidden absolute |
/// | **Rewrite** | canonical save | no comment-preservation claim |
/// | **Witness** | complete | missing component |
/// | **OCEL** | lifecycle history | not q_config |
#[derive(Debug, Deserialize, Serialize, Clone)]
struct MatrixConfig {
    name: String,
    port: u16,
    debug: bool,
    hosts: Vec<String>,
}

impl Validate for MatrixConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
        // Assert that hosts array is not empty
        v.check_predicate("hosts", !self.hosts.is_empty(), "hosts_not_empty", "at least one host must be specified");
    }
}
impl ConfigLifecycle for MatrixConfig {}

fn main() {
    println!("--- DfCM Configuration Axes Matrix ---");

    // Axis 1: Source & Layering (defaults + inline base + optional file)
    let loader = TrustedLoader::new()
        .layer_str(r#"
            name = "defaults"
            port = 8080
            debug = false
            hosts = ["localhost"]
        "#, "defaults")
        .layer_str(r#"
            name = "matrix-service"
            port = 9090
        "#, "base");

    // Load admitted config
    let admitted = loader.load_admitted::<MatrixConfig>().unwrap();
    
    println!("Matrix axes loaded successfully:");
    println!("  - Resolved Name: {}", admitted.value().name);
    println!("  - Resolved Port: {}", admitted.value().port);
    println!("  - Resolved Debug: {}", admitted.value().debug);
    println!("  - Resolved Hosts: {:?}", admitted.value().hosts);
    println!("  - Witness Hash: {}", admitted.witness().hash());
}
