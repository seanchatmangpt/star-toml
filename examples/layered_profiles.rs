//! Demonstrates layered configuration profiles with field provenance tracking.
//!
//! Run with: `cargo run --example layered_profiles`

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AppConfig {
    name: String,
    port: u16,
    profile: String,
    logging: Logging,
    paths: Paths,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Logging {
    level: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Paths {
    data_dir: String,
}

impl Validate for Logging {
    fn validate(&self, v: &mut Validator) {
        v.check_one_of("level", &self.level, &["debug", "info", "warn", "error"]);
    }
}

impl Validate for Paths {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("data_dir", &self.data_dir);
    }
}

impl Validate for AppConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
        v.field("logging", |v| self.logging.validate(v));
        v.field("paths", |v| self.paths.validate(v));
    }
}
impl ConfigLifecycle for AppConfig {}

fn main() {
    println!("--- Layered Profiles & Winner Mapping Example ---");

    // DfCM Invariant: Layer priority order is defaults < base < profile.
    // Each field's final value must trace back to its winning layer.
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml")
        .layer_file("examples/config_patterns/service.dev.toml");

    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            println!("Admission Successful!");
            println!("Config value: {:?}", admitted.value());
            println!("\nWinner Map (Provenance Trace):");
            for (field, winning_layer) in admitted.global_winner_map() {
                println!("  - Field '{}' won by layer: '{}'", field, winning_layer);
            }
        }
        Err(e) => {
            println!("Admission failed: {:?}", e);
        }
    }
}
