# How to Generate Configuration Documentation

This guide describes how to run the documentation projection CLI to generate and export reference docs and JSON schemas.

## Goal

Generate fresh markdown projections and verify them against the schema model definition.

## Implementation Steps

### 1. Export Documentation Model

Run the documentation generator binary to dump schemas and canonical reference files:

```bash
cargo run --bin star_toml_docs export
```

This exports three key artifacts:
- Markdown Reference: `docs/config_reference.md`
- Schema JSON: `docs/schema.json`
- Schema Metadata: `docs/config_metadata.json`

### 2. Verify Freshness

Validate that the generated files match the schema facts:

```bash
cargo run --bin star_toml_docs verify
```

If files are out of sync, the verify task outputs `DOC-010` (GeneratedDocsOutOfDate) and exits non-zero.
