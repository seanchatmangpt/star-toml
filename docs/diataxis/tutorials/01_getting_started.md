# Getting Started with star-toml

Welcome to star-toml! This tutorial guides you through your first steps with the framework. You will learn how to initialize the library, load a basic TOML configuration, and read its values.

## Learning Objectives

In this tutorial, you will:
1. Initialize the `TrustedLoader`.
2. Load a simple TOML configuration from disk.
3. Access values using typestate lifecycle wrappers.

## Step 1: Initialize the Loader

First, import the necessary modules and create an instance of `TrustedLoader`:

```rust
use star_toml::loader::TrustedLoader;

fn main() {
    let loader = TrustedLoader::new();
    println!("Loader initialized!");
}
```

## Step 2: Load Your Configuration

Next, use the loader to ingest a basic TOML configuration file:

```rust
let config_content = r#"
    [system]
    log_path = "/var/log/app.log"
    cache_size = 10485760
"#;

let raw_config = loader.load_str(config_content)
    .expect("Failed to parse TOML structure");
```

## Step 3: Transition to Admitted State

To guarantee your configuration meets security constraints, transition it through the admission pipeline:

```rust
let admitted_config = raw_config.admit()
    .expect("Configuration did not pass admission checks");

println!("Log path: {}", admitted_config.system.log_path);
```

Congratulations! You have loaded and admitted your first configuration with star-toml.
