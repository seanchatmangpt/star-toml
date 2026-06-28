//! Demonstrates exploratory mode for configurations with unknown fields.
//!
//! Run with: `cargo run --example exploratory_unknown_fields`

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
    println!("--- Exploratory Unknown Fields Example ---");

    // Load invalid_unknown_field.toml which has the extra field: `extra_field = "must_fail"`
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/invalid_unknown_field.toml");

    // DfCM Invariant: load_admitted_exploratory() allows unknown fields,
    // but should only be used in non-production environments.
    match loader.load_admitted_exploratory::<AppConfig>() {
        Ok(admitted) => {
            println!("Exploratory Admission Successful (Unknown fields ignored)!");
            println!("Value: {:?}", admitted.value());
            println!("Witness: {}", admitted.witness().hash());
        }
        Err(e) => {
            println!("Failed to load in exploratory mode: {:?}", e);
        }
    }
}
