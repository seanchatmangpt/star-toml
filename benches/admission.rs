//! Admission-path benchmark (deterministic timing, no external harness).
//!
//! Measures the real star-toml pipeline over the real `toml` parser/writer:
//! parse -> merge -> deserialize -> validate -> digest (`TrustedLoader::load`)
//! and `save_canonical`, for a small fixture and a 1000-table config.
//!
//! Run: `cargo bench --bench admission`. Emits one JSON line per case and exits
//! non-zero if any median exceeds its regression bound (`BOUND_NS`), so the
//! bench is itself a falsifier for performance regressions introduced by a
//! dependency bump. Numbers recorded in `docs/bench/pr12-admission-receipt.json`.

#![allow(missing_docs, clippy::pedantic, clippy::unwrap_used, clippy::expect_used)]

use std::{collections::BTreeMap, hint::black_box, time::Instant};

use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{Config, ConfigLifecycle, Raw, TrustedLoader},
    Validate, Validator,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Svc {
    host: String,
    port: u16,
    weight: f64,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Big {
    name: String,
    svc: BTreeMap<String, Svc>,
}

impl Validate for Big {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        for (k, s) in &self.svc {
            v.check_range(&format!("svc.{k}.port"), s.port, 1..=65535);
        }
    }
}
impl ConfigLifecycle for Big {}

fn big_config(n: usize) -> String {
    let mut s = String::from("name = \"fleet\"\n");
    for i in 0..n {
        s.push_str(&format!(
            "\n[svc.s{i:04}]\nhost = \"h{i}.internal\"\nport = {}\nweight = 0.{i}\ntags = [\"a\", \"b{i}\", \"q\\\"x\"]\n",
            1024 + (i % 60000)
        ));
    }
    s
}

/// Median ns/op over `iters` runs of `f`.
fn median_ns(iters: usize, mut f: impl FnMut()) -> u128 {
    for _ in 0..(iters / 10).max(1) {
        f(); // warm-up
    }
    let mut v: Vec<u128> = (0..iters)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_nanos()
        })
        .collect();
    v.sort_unstable();
    v[v.len() / 2]
}

/// Regression bounds (release profile). ~10x the medians recorded on the
/// operator's M-series host for PR #12 head 3f47c70; exceeding one fails.
const BOUND_NS: &[(&str, u128)] = &[
    ("trusted_load_small", 300_000),
    ("trusted_load_1000_tables", 50_000_000),
    ("save_canonical_1000_tables", 40_000_000),
];

fn main() {
    let small =
        "name = \"svc\"\n[svc.a]\nhost = \"h\"\nport = 8080\nweight = 0.5\ntags = [\"x\"]\n";
    let big = big_config(1000);
    let dir = std::env::temp_dir().join(format!("star-toml-bench-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("bench temp dir");
    let out = dir.join("canonical.toml");
    let validated = Config::<Raw>::new(&big)
        .merge(None)
        .unwrap()
        .deserialize::<Big>()
        .unwrap()
        .validate()
        .unwrap();

    let results = [
        (
            "trusted_load_small",
            median_ns(2000, || {
                black_box(TrustedLoader::new().layer_str(small, "s").load::<Big>().unwrap().digest);
            }),
        ),
        (
            "trusted_load_1000_tables",
            median_ns(30, || {
                black_box(
                    TrustedLoader::new().layer_str(big.clone(), "b").load::<Big>().unwrap().digest,
                );
            }),
        ),
        (
            "save_canonical_1000_tables",
            median_ns(30, || {
                validated.save_canonical(&out).unwrap();
            }),
        ),
    ];
    let _ = std::fs::remove_dir_all(&dir);

    let mut failed = false;
    for (name, ns) in results {
        let bound = BOUND_NS.iter().find(|(n, _)| *n == name).map_or(u128::MAX, |(_, b)| *b);
        let ok = ns <= bound;
        failed |= !ok;
        println!(
            "{{\"case\":\"{name}\",\"median_ns\":{ns},\"bound_ns\":{bound},\"within_bound\":{ok}}}"
        );
    }
    if failed {
        eprintln!("admission benchmark exceeded regression bound");
        std::process::exit(1);
    }
}
