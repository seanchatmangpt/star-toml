//! Demonstrates strict unknown field rejection.
//!
//! Run with: `cargo run --example strict_unknown_fields`

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AppConfig {
    name: String,
    port: u16,
}

impl Validate for AppConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
    }
}
impl ConfigLifecycle for AppConfig {}

fn main() {
    println!("--- Strict Unknown Fields Example ---");

    // Load invalid_unknown_field.toml which has the extra field: `extra_field = "must_fail"`
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/invalid_unknown_field.toml");

    // DfCM Invariant: load_admitted() is strict by default and rejects unknown fields
    match loader.load_admitted::<AppConfig>() {
        Ok(_) => {
            println!("Error: Admitted configuration with unknown fields!");
        }
        Err(e) => {
            println!("Admission Successfully Blocked (as expected)!");
            println!("Details: {:?}", e);
        }
    }
}
