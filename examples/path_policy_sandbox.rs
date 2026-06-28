//! Demonstrates path validation policy checks and path witness generation.
//!
//! Run with: `cargo run --example path_policy_sandbox`

use star_toml::path::{resolve_and_validate, PathPolicy};
use std::path::Path;

fn main() {
    println!("--- Path Policy Sandbox Example ---");

    // NOTE: This example does not claim symlink-safe sandboxing.
    // It verifies path policies via lexical clean-path rules.

    let source_path = Path::new("examples/config_patterns/paths.toml");

    // 1. Valid path under Sandbox policy
    let sandbox = PathPolicy::Sandbox {
        root: std::path::PathBuf::from("examples"),
    };
    match resolve_and_validate("config_patterns/service.toml", source_path, &sandbox) {
        Ok((resolved, witness)) => {
            println!("Valid Path Resolved: {:?}", resolved);
            println!("  - Witness Raw Path: {}", witness.raw_path);
            println!("  - Witness Accepted: {}", witness.accepted);
            println!("  - Witness Rejection Code: {:?}", witness.rejection_code);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    // 2. Traversal path (../) under Sandbox policy
    match resolve_and_validate("../secret.toml", source_path, &sandbox) {
        Ok(_) => println!("Error: Traversal path was accepted!"),
        Err(e) => println!("\nTraversal Blocked: {}", e),
    }

    // 3. Windows-style traversal (foo\..\bar) under Sandbox policy
    match resolve_and_validate("config_patterns\\..\\..\\secret.toml", source_path, &sandbox) {
        Ok(_) => println!("Error: Windows traversal path was accepted!"),
        Err(e) => println!("Windows Separator Traversal Blocked: {}", e),
    }

    // 4. Absolute path under RelativeOnly policy
    let relative_only = PathPolicy::RelativeOnly;
    match resolve_and_validate("/etc/passwd", source_path, &relative_only) {
        Ok(_) => println!("Error: Absolute path was accepted under RelativeOnly!"),
        Err(e) => println!("Absolute Path under RelativeOnly Blocked: {}", e),
    }
}
