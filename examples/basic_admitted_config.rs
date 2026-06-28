//! Demonstrates basic config loading using the trusted admission pipeline.
//!
//! Run with: `cargo run --example basic_admitted_config`

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
    println!("--- Basic Config Admission Example ---");

    // DfCM Invariant: Raw parsing is never treated as trusted config.
    // Instead, we construct a TrustedLoader to enforce all boundaries.
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml");

    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            println!("Admission Successful ($q_{{config}} = 1$)!");
            println!("Value: {:?}", admitted.value());
            println!("Witness: {}", admitted.witness().hash());
        }
        Err(e) => {
            println!("Admission Refused ($q_{{config}} = 0$): {:?}", e);
        }
    }
}
