//! Demonstrates the most common configuration patterns represented as named axes
//! in the star-toml framework.
//!
//! Run with: `cargo run --example dfcm_common_patterns`

#![allow(dead_code, unused_variables)]

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    Validate, Validator,
};

// --- 1. Service Config Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ServiceConfig {
    name: String,
    port: u16,
    profile: String,
}

impl Validate for ServiceConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("port", self.port, 1024..=65535);
    }
}
impl ConfigLifecycle for ServiceConfig {}

// --- 2. Web App Config Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct WebAppConfig {
    host: String,
    secure: bool,
}

impl Validate for WebAppConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("host", &self.host);
    }
}
impl ConfigLifecycle for WebAppConfig {}

// --- 3. Database Config Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct DatabaseConfig {
    url: String,
    pool_size: u32,
}

impl Validate for DatabaseConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("url", &self.url);
        v.check_range("pool_size", self.pool_size, 1..=100);
    }
}
impl ConfigLifecycle for DatabaseConfig {}

// --- 4. Feature Flags Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct FeatureFlags {
    enable_beta: bool,
}

impl Validate for FeatureFlags {
    fn validate(&self, _v: &mut Validator) {}
}
impl ConfigLifecycle for FeatureFlags {}

// --- 5. Worker Queues Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkerQueueConfig {
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

impl Validate for WorkerQueueConfig {
    fn validate(&self, v: &mut Validator) {
        for (i, worker) in self.workers.iter().enumerate() {
            v.field(&format!("workers[{}]", i), |v| worker.validate(v));
        }
    }
}
impl ConfigLifecycle for WorkerQueueConfig {}

// --- 6. Paths/Filesystem Config Axis ---
#[derive(Debug, Deserialize, Serialize, Clone)]
struct PathsConfig {
    data_dir: String,
}

impl Validate for PathsConfig {
    fn validate(&self, v: &mut Validator) {
        // Paths validation is a critical DfCM axis
        v.check_path("data_dir", &self.data_dir, None);
    }
}
impl ConfigLifecycle for PathsConfig {}

fn main() {
    println!("DfCM Common Patterns Example");

    // DfCM enforces that raw parsing alone is never treated as trusted config.
    // Instead, we use TrustedLoader to build an admitted config from structured layers,
    // apply env policies, run type schema checks, perform path validation, rewrite,
    // and finally generate a witness.

    let loader = TrustedLoader::new()
        .layer_str(r#"
            name = "worker-service"
            port = 9000
            profile = "default"
        "#, "defaults");

    let admitted = loader.load_admitted::<ServiceConfig>();
    match admitted {
        Ok(cfg) => {
            println!("Successfully admitted ServiceConfig: {:?}", cfg.value());
            println!("Cryptographic Witness: {}", cfg.witness().hash());
        }
        Err(e) => {
            println!("Admission failed: {:?}", e);
        }
    }
}
