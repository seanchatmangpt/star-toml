# Implementing Custom Validators

This tutorial demonstrates how to extend `star-toml` with custom domain-specific validators.

## Learning Objectives

In this tutorial, you will:
1. Implement the `Validator` trait.
2. Bind a validator to a config structure.
3. Validate configuration rules.

## Step 1: Define Your Custom Validator

Create a custom validator struct verifying path formats:

```rust
use star_toml::validation::{Validator, Diagnostic, Severity};

pub struct PathExtensionValidator {
    pub required_ext: String,
}

impl Validator for PathExtensionValidator {
    fn validate(&self, value: &str) -> Result<(), Diagnostic> {
        if value.ends_with(&self.required_ext) {
            Ok(())
        } else {
            Err(Diagnostic::new(
                "CUSTOM-001",
                "Invalid path extension",
                Severity::Error,
                format!("File must end with {}", self.required_ext),
            ))
        }
    }
}
```

## Step 2: Register & Run validation

Attach the validator instance dynamically inside your loader initialization pipeline:

```rust
let val = PathExtensionValidator { required_ext: ".log".to_string() };
let loader = TrustedLoader::new().with_validator("log_path", val);
```
