# ST-104: Layer Bounds and Win-Layer Tracing

## Description
Implements the fourth of the critical configuration bounds (Layer Bounds) to enforce a deterministic layering precedence, recursive merging of tables, whole-value replacement of arrays and scalars, and winning-layer tracing metadata to verify configuration provenance and prevent silent overrides.

## Key Requirements
1. **Deterministic Layer Ordering (`star_toml::layer::precedence`)**:
   - Enforce a strict, immutable layer hierarchy: `Defaults < Files < Env` (where defaults have lowest precedence and environment variables have highest).
   - If multiple configuration files are loaded, they must be processed and merged in the exact, deterministic order in which they were added/configured in the [`Loader`](file:///Users/sac/star-toml/src/loader.rs#L136-L151).
   - Reject any loading/merging logic that allows non-deterministic evaluation of config values, preventing the `ambiguous_layer_order` counterexample.

2. **Table Merging vs. Array/Scalar Replacement (`star_toml::layer::merge`)**:
   - **Deep Table Merging**: When merging two [`Config`](file:///Users/sac/star-toml/src/loader.rs#L422) layers, TOML tables must be merged recursively. Sibling keys in lower precedence layers must be preserved intact if they are not explicitly overridden by higher layers (e.g. by [`deep_merge`](file:///Users/sac/star-toml/src/merge.rs#L39-L53)).
   - **Array/Scalar Replacement**: Non-table types (arrays, integers, floats, booleans, strings, datetimes) must be completely replaced by the value in the higher precedence layer. Do not append, concatenate, or perform element-wise merging.

3. **Winning-Layer Tracing Metadata (`star_toml::layer::trace`)**:
   - The final [`TrustedConfig`](file:///Users/sac/star-toml/src/loader.rs#L746-L755) and its [`ConfigSourceReport`](file:///Users/sac/star-toml/src/loader.rs#L722-L725) must expose or track a winning-layer tracing map.
   - For every leaf path in the resolved TOML configuration tree, track which source layer (e.g., defaults, a specific file path, or a prefixed environment variable name) supplied the final winning value.
   - If a layer override is applied to a configuration field, it must be recorded in this trace metadata.
   - If an override is applied but not reported in the winning-layer trace history or the final witness, reject the configuration immediately with a validation error to prevent the `unreported_layer_override` counterexample.

## Acceptance Criteria
- [ ] Confirms the layer evaluation priority order is strictly `Defaults < Files < Env`.
- [ ] Recursively merges nested tables without discarding sibling keys present in lower layers.
- [ ] Replaces arrays, booleans, numbers, strings, and datetimes in their entirety when overridden by higher layers.
- [ ] Captures the winning source descriptor (including default labels, absolute file paths, or env variable name) for every active configuration path in a queryable metadata lookup.
- [ ] Rejects the configuration as inadmissible if an override has been applied but is missing or unreported in the tracing history.
- [ ] Emits identical merging outputs and winning-layer trace mappings for identical inputs.

## Counterexamples Covered
- `ambiguous_layer_order`: Non-deterministic resolution of configuration values across multiple files or source layers.
- `unreported_layer_override`: Value overrides applied from higher layers that are not recorded in the tracing metadata or witness history.

## Verification Method
- **Unit Tests**:
  - Verify that nested tables merge recursively and preserve non-overridden sibling keys in [tests/e2e_tests.rs](file:///Users/sac/star-toml/tests/e2e_tests.rs).
  - Verify that arrays and scalars from higher layers replace the lower layers completely.
  - Verify that shuffling the registration order of identical input layers does not lead to non-deterministic output values.
- **Integration Tests**:
  - Load a config with defaults, multiple files, and environment variable overrides, asserting that final values correspond to the precedence order and their winning layers are tracked correctly.
  - Assert that attempting to inject a secret, unreported override triggers a validation failure.
