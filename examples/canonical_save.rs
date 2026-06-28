//! Demonstrates deterministic canonical serialization of validated configs.
//!
//! Run with: `cargo run --example canonical_save`

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};
use tempfile::NamedTempFile;

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
    println!("--- Canonical Save Example ---");

    // DfCM Invariant: save_canonical is only callable after validation (e.g. Frozen state).
    // Comment preservation is NOT claimed.
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml");

    match loader.load_frozen::<AppConfig>() {
        Ok(result) => {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();

            match result.config.save_canonical(path) {
                Ok(_) => {
                    let content = std::fs::read_to_string(path).unwrap();
                    println!("Canonical TOML Written Successfully:");
                    print!("{}", content);
                }
                Err(e) => {
                    println!("Failed to save: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to load config: {:?}", e);
        }
    }
}
