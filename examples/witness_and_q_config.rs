//! Demonstrates cryptographic witness derivation and q_config standing.
//!
//! Run with: `cargo run --example witness_and_q_config`

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
    println!("--- Witness and q_config Example ---");

    // 1. Initial configuration load
    let loader1 = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml");
    let admitted1 = loader1.load_admitted::<AppConfig>().unwrap();
    let hash1 = admitted1.witness().hash().to_owned();
    println!("Initial Witness Hash: {}", hash1);

    // 2. Load identical configuration in another run
    let loader2 = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml");
    let admitted2 = loader2.load_admitted::<AppConfig>().unwrap();
    let hash2 = admitted2.witness().hash();
    println!("Second Run Witness Hash:  {}", hash2);

    // Assert determinism
    assert_eq!(hash1, hash2, "Witness must be fully deterministic across runs");
    println!("Determinism Verified: identical inputs yield identical witnesses.");

    // 3. Load slightly modified configuration (e.g. port changed via prod layer)
    let loader3 = TrustedLoader::new()
        .layer_file("examples/config_patterns/service.toml")
        .layer_file("examples/config_patterns/service.prod.toml");
    let admitted3 = loader3.load_admitted::<AppConfig>().unwrap();
    let hash3 = admitted3.witness().hash();
    println!("Modified Run Witness Hash: {}", hash3);

    // Assert that changing parameters changes the witness hash
    assert_ne!(hash1, hash3, "Witness hash must change when configuration changes");
    println!("Integrity Verified: config parameter alterations shift the cryptographic witness.");
}
