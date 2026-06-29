# Refactoring cargo-cicd to Use star-toml

This document guides the refactoring of `cargo-cicd` to leverage the `star-toml` library for configuration parsing, schema validation, and cryptographic receipt binding.

## Integration Architecture

`cargo-cicd` acts as the execution orchestrator, while `star-toml` provides the underlying security guarantees. 

```
┌───────────────────────────────────────┐
│              cargo-cicd               │
│  (Orchestrator, CLI, Agent Tasks)     │
└───────────┬───────────────▲───────────┘
            │               │
      Loads Configuration   Verifies Receipts
            │               │
┌───────────▼───────────────┴───────────┐
│              star-toml                │
│  (Typestates, Path Sandbox, Receipts) │
└───────────────────────────────────────┘
```

## Refactoring Steps

### 1. Replace Configuration Parser

Replace custom TOML loading logic in `cargo-cicd` with the `star-toml` loader pipeline:

```rust
// Before: Custom TOML deserialization
// let config: CicdConfig = toml::from_str(&content)?;

// After: star-toml Typestate Admission Loader
use star_toml::loader::TrustedLoader;
use star_toml::path::{PathPolicy, PathSandbox};

let sandbox = PathSandbox::new("/Users/sac/star-toml")?;
let path_policy = PathPolicy::default().with_sandbox(sandbox);

let loader = TrustedLoader::new()
    .strict_admission(true)
    .with_path_policy(path_policy);

let raw_config = loader.load_file("cargo-cicd.toml")?;
let admitted_config = raw_config.admit()?; // Returns AdmittedConfig<CicdConfig>
```

### 2. Standardize Diagnostics

Utilize `star-toml`'s structured diagnostics matrix (`DOC-xxx` and `SCH-JSON-xxx`) for CI/CD rules and validation failures.

- When task schemas fail to compile: Emit `SCH-JSON-002`.
- When configuration values bypass bounds: Return `SCH-JSON-008`.
- When build outputs drift from generated specs: Trigger `DOC-010`.

### 3. Implement Cryptographic Task Receipts

Instead of tracking task states in transient memory or simple JSON logs, bind each execution to a `ReceiptBinding` record.

```rust
use star_toml::docs::ReceiptBinding;

let task_receipt = ReceiptBinding {
    json_schema_digest: schema_hash,
    admitted_schema_digest: admitted_hash,
    admitted_config_digest: config_hash,
    documentation_model_digest: model_hash,
    projection_digest: output_file_hash,
    generator_version: env!("CARGO_PKG_VERSION").to_string(),
    signer: std::env::var("CARGO_CICD_SIGNER").unwrap_or_else(|_| "system-signer".to_string()),
};
```

This guarantees that every executed task produces immutable cryptographic evidence.

## Verification Checklist for Refactor

- [ ] All `cargo-cicd.toml` configurations are loaded strictly using `AdmittedConfig<T>`.
- [ ] Unknown configuration keys fail with path-precise segments.
- [ ] Task execution blocks until all dependencies have valid receipts bound.
- [ ] Path policies prevent files from reading outside the cargo workspace sandbox.
