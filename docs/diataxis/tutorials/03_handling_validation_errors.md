# Handling Validation Errors

This tutorial explains how to capture, inspect, and handle validation diagnostics generated when configuration admission fails.

## Learning Objectives

In this tutorial, you will:
1. Trigger a validation failure by providing out-of-bounds data.
2. Inspect the returned diagnostic codes.
3. Access diagnostic repair hints.

## Step 1: Trigger a Diagnostic

Provide a TOML config containing an invalid value (e.g. `cache_size` below minimum):

```rust
let invalid_config = r#"
    [system]
    log_path = "/var/log/app.log"
    cache_size = 512
"#;
```

## Step 2: Extract Diagnostics

Attempt to load the configuration and handle the returned error result:

```rust
match loader.load_str(invalid_config) {
    Ok(_) => println!("Unexpected success!"),
    Err(err) => {
        for diagnostic in err.diagnostics() {
            println!("Code: {}", diagnostic.code);
            println!("Message: {}", diagnostic.message);
            println!("Repair: {}", diagnostic.repair_hint);
        }
    }
}
```

This ensures errors contain precise path coordinates indicating exactly which field failed validation.
