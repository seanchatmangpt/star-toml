# ST-104: Layer Bounds and Win-Layer Tracing

## Description
Implements the fourth of the critical configuration bounds (Layer Bounds) to enforce a deterministic layering precedence, recursive merging of tables, whole-value replacement of arrays and scalars, and winning-layer tracing metadata to verify configuration provenance and prevent silent overrides.

## Key Requirements
1. **Deterministic Layer Ordering (`star_toml::layer::precedence`)**:
   - Enforce a layer hierarchy following registration order, with Env applied last (where defaults registered first have lowest precedence and environment variables have highest).
   - If multiple configuration files are loaded, they must be processed and merged in the exact, deterministic order in which they were added/configured in the [`Loader`](file:///Users/sac/star-toml/src/loader.rs#L136-L151).
   - Reject any loading/merging logic that allows non-deterministic evaluation of config values, preventing the `ambiguous_layer_order` counterexample.

2. **Table Merging vs. Array/Scalar Replacement (`star_toml::layer::merge`)**:
   - **Deep Table Merging**: When merging two [`Config`](file:///Users/sac/star-toml/src/loader.rs#L422) layers, TOML tables must be merged recursively. Sibling keys in lower precedence layers must be preserved intact if they are not explicitly overridden by higher layers (e.g. by [`deep_merge`](file:///Users/sac/star-toml/src/merge.rs#L39-L53)).
   - **Array/Scalar Replacement**: Non-table types (arrays, integers, floats, booleans, strings, datetimes) must be completely replaced by the value in the higher precedence layer. Do not append, concatenate, or perform element-wise merging.

3. **Winning-Layer Tracing Metadata (`star_toml::layer::trace`)**:
   - The final [`AdmittedConfig`](file:///Users/sac/star-toml/src/loader.rs#L1561-L1568) must expose a detailed [`LayerReport`](file:///Users/sac/star-toml/src/reports.rs#L104-L106) containing a sequence of [`LayerEntry`](file:///Users/sac/star-toml/src/reports.rs#L81-L100)s.
   - For every layer, track the `layer_name`, `priority`, `source_id`, `digest`, `layer_order_digest`, and the individual fields written by that layer via a `winning_field_map` ([`WinnerMap`](file:///Users/sac/star-toml/src/merge.rs#L18)).
   - The cumulative winning layer (descriptor/label) for each leaf path in the resolved TOML configuration tree must be exposed via `global_winner_map` ([`WinnerMap`](file:///Users/sac/star-toml/src/merge.rs#L18)).
   - If a layer override is applied to a configuration field, it must be recorded in this trace metadata.
   - The loader is correct-by-construction: the code applying overrides automatically logs to the winner map and reports, preventing the `unreported_layer_override` counterexample.

## Acceptance Criteria
- [x] Confirms the layer evaluation priority order is strictly determined by registration order with Env overrides on top.
- [x] Recursively merges nested tables without discarding sibling keys present in lower layers.
- [x] Replaces arrays, booleans, numbers, strings, and datetimes in their entirety when overridden by higher layers.
- [x] Captures the winning source descriptor (including default labels, relative file paths, or env variable name) for every active configuration path in a queryable metadata lookup ([`WinnerMap`](file:///Users/sac/star-toml/src/merge.rs#L18)) on [`AdmittedConfig`](file:///Users/sac/star-toml/src/loader.rs#L1561-L1568).
- [x] Guarantees correct-by-construction recording of all overrides in the tracing history.
- [x] Emits identical merging outputs and winning-layer trace mappings for identical inputs.

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
   - Assert that applied overrides are recorded in the tracing history.
