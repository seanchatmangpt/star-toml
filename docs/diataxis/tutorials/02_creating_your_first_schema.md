# Creating Your First Schema

This tutorial shows how to define validation rules for your configuration using a JSON schema file.

## Learning Objectives

In this tutorial, you will:
1. Define a schema rule constraints structure.
2. Bind the schema to your configuration domain.
3. Validate the configurations against the schema.

## Step 1: Define the Schema

Create a JSON Schema document named `schema.json` with the following validation rules:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "log_path": {
      "type": "string",
      "minLength": 1
    },
    "cache_size": {
      "type": "integer",
      "minimum": 1024
    }
  },
  "required": ["log_path"]
}
```

## Step 2: Ingest the Schema

In your Rust code, load the schema to define the admission boundaries:

```rust
use star_toml::schema::JsonSchemaBridge;

let schema = JsonSchemaBridge::from_file("schema.json")
    .expect("Failed to parse JSON Schema");
```

## Step 3: Run Validated Loader

Load configuration data passing the schema mapping rule context:

```rust
let loader = TrustedLoader::new().with_schema(schema);
let config = loader.load_file("config.toml")
    .expect("Validation failed or schema mismatches");
```
