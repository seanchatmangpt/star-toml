//! Demonstrates arrays of tables configuration and validation checks.
//!
//! Run with: `cargo run --example arrays_of_tables`

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AppConfig {
    workers: Vec<WorkerConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkerConfig {
    name: String,
    threads: u32,
}

impl Validate for WorkerConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("threads", self.threads, 1..=32);
    }
}

impl Validate for AppConfig {
    fn validate(&self, v: &mut Validator) {
        for (i, worker) in self.workers.iter().enumerate() {
            v.field(&format!("workers[{}]", i), |v| worker.validate(v));
        }
    }
}
impl ConfigLifecycle for AppConfig {}

fn main() {
    println!("--- Arrays of Tables Example ---");

    // 1. Load valid workers.toml
    let loader = TrustedLoader::new()
        .layer_file("examples/config_patterns/workers.toml");

    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            println!("Valid Workers Admission Successful!");
            for worker in &admitted.value().workers {
                println!("  - Worker: '{}', Threads: {}", worker.name, worker.threads);
            }
        }
        Err(e) => {
            println!("Unexpected error: {:?}", e);
        }
    }

    // 2. Load invalid arrays of tables setup with unknown fields
    // We expect this to fail under strict mode (by default).
    let loader_bad = TrustedLoader::new()
        .layer_str(r#"
            [[workers]]
            name = "email"
            threads = 4
            extra = "must_fail"
        "#, "invalid_workers");

    match loader_bad.load_admitted::<AppConfig>() {
        Ok(_) => {
            println!("Error: Admitted invalid workers config!");
        }
        Err(e) => {
            println!("\nInvalid Workers Admission Blocked (as expected)!");
            println!("Error: {:?}", e);
        }
    }
}
