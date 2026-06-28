//! Demonstrates environment variable overrides and reporting.
//!
//! Run with: `cargo run --example env_overrides`

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
        v.check_non_empty("level", &self.level);
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
    println!("--- Environment Overrides & Reporting Example ---");

    // DfCM Invariant: Ambient environment variables are ignored unless explicitly prefixed.
    // Set a matching prefixed variable, and an un-prefixed one.
    std::env::set_var("STAR_TOML_EXAMPLE_PORT", "9999");
    std::env::set_var("OTHER_VAR_PORT", "1234"); // Should be ignored

    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml")
        .env_prefix("STAR_TOML_EXAMPLE_");

    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            println!("Admission Successful!");
            println!("Port: {} (should be overridden to 9999)", admitted.value().port);
            
            println!("\nEnv Override Report:");
            for entry in &admitted.env_report().entries {
                println!(
                    "  - Key: '{}', Path: '{}', Accepted: {}, Coerced Digest: {:?}",
                    entry.raw_env_key, entry.mapped_path, entry.accepted, entry.coerced_value_digest
                );
            }
        }
        Err(e) => {
            println!("Admission failed: {:?}", e);
        }
    }

    // Clean up
    std::env::remove_var("STAR_TOML_EXAMPLE_PORT");
    std::env::remove_var("OTHER_VAR_PORT");
}
