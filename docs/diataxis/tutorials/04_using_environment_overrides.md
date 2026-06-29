# Using Environment Overrides

This tutorial shows how to layer configurations using environment variables securely.

## Learning Objectives

In this tutorial, you will:
1. Define env prefix namespaces to isolate variables.
2. Override schema values using target env keys.
3. Verify that environment variables respect validation bounds.

## Step 1: Initialize isolated env prefixes

To prevent arbitrary system env vars from polluting configuration state, configure your loader with a namespace prefix:

```rust
let loader = TrustedLoader::new()
    .with_env_prefix("MYAPP_");
```

## Step 2: Set the Overriding variables

Define the overriding variable prefix matching your schema key structure:

```bash
export MYAPP_SYSTEM_CACHE_SIZE=20971520
```

## Step 3: Run the layered loader

Load the TOML configuration:

```rust
let config = loader.load_file("config.toml")
    .expect("Environment layering failed");

// The cache_size will now be 20971520 bytes instead of the default value.
assert_eq!(config.system.cache_size, 20971520);
```
