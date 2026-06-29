# How to Sandbox Path Policies

This guide provides steps to setup path validation gates restricting directory access to configured folders.

## Goal

Prevent traversal escapes (e.g., directory traversal or symlink escapes) outside the boundary root.

## Implementation Steps

### 1. Define Sandbox Root Directory

Initialize the path validation rule configuration:

```rust
use star_toml::path::{PathPolicy, PathSandbox};

let sandbox = PathSandbox::new("/Users/sac/star-toml/sandbox")
    .expect("Invalid sandbox path");
```

### 2. Configure Relative/Strict Sandbox Constraints

Apply restriction types to discard absolute configuration sources or files bypassing the prefix root:

```rust
let policy = PathPolicy::default()
    .with_sandbox(sandbox)
    .enforce_relative_only(true);
```

### 3. Attach Sandbox to Loader

Feed sandbox policies to the loader:

```rust
let loader = TrustedLoader::new()
    .with_path_policy(policy);
```

Attempts to load files outside `/Users/sac/star-toml/sandbox` will now be refused.
