# Configuration Options Reference

This reference lists the compilation features, variables, and runtime configurations governing star-toml behavior.

## Cargo Features

- `default`: standard features.
- `wasm4pm-compat`: Enables compatibility mappings for the wasm4pm environment.
- `e2e_tests`: Compiles the E2E verification test harness.

## Environment Variables

- `STAR_TOML_DEV`: Set to `1` to bypass the release verifier's git-clean check.
- `STAR_TOML_DOCS_TIMESTAMP`: Overrides the generated documentation timestamp.
- `STAR_TOML_DOCS_SIGNER`: Overrides the signer key signature used during receipt creation.

## Runtime Parameters

- `cache_size`: Default value: `10485760` bytes. Range: `1024` to `1073741824`.
- `log_path`: Target file path for the lifecycle event logs.
