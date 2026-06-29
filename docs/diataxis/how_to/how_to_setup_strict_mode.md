# How to Setup Strict Admission Mode

This guide details the procedure to configure strict mode in star-toml, enforcing that no configurations validated against unadmitted schemas are ever allowed to load.

## Goal

Ensure the loader operates under strict admission rules, instantly refusing any configuration schema that fails the admission pipeline.

## Implementation Steps

### 1. Enable Strict Mode flag in loader pipeline

Initialize the `TrustedLoader` using the `strict_admission()` option block:

```rust
use star_toml::loader::TrustedLoader;

let loader = TrustedLoader::new()
    .strict_admission(true);
```

### 2. Guard Schema validation

Ensure that the loaded schema has passed admission criteria. In strict mode, loading config files without registered admitted schema facts will trigger error `SCH-JSON-008`:

```rust
let config = loader.load_file("config.toml");
match config {
    Err(err) if err.code() == "SCH-JSON-008" => {
        println!("Access Denied: Schema is not admitted!");
    }
    _ => {}
}
```

### 3. Handle Refusals

Explicitly format error diagnostics to notify operational systems of policy-level admissions blockades.
