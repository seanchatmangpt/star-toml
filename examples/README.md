# star-toml DfCM Examples

This directory demonstrates the **Design for Combinatorial Maximalism (DfCM)** principle applied to configuration admission in `star-toml`.

## What is DfCM?

Design for Combinatorial Maximalism (DfCM) rejects the Minimal Viable Product (MVP) happy-path approach to safety. 
- **MVP asks**: *"What is the smallest happy path we can ship?"*
- **DfCM asks**: *"What is the complete bounded variant space this system must survive?"*

In a DfCM-driven architecture, we establish that:
* **Configuration is Operational Law**: A config controls the system's execution capabilities and boundary invariants.
* **Raw Parse ≠ Trusted Config**: Merely compiling or syntactically parsing a TOML string does not grant it standing.
* **OCEL = Lifecycle/Process History**: Process-mining history tracks what happened but is not standing authority.
* **q_config = Standing Decision**: The standing bit $q_{config} = 1$ is earned only when the config satisfies all bounds, constraints, witnesses, and has a clean/empty failset.
* **AdmittedConfig<T> = Terminal Witness-Backed Config**: The only constructible representation of a config with standing.

```text
Do not build a path.
Build the bounded space of paths,
the laws that admit them,
the generators that manufacture them,
the detectors that falsify them,
the witnesses that preserve them,
and the gate that decides standing.
```

---

## DfCM Examples Matrix

| Example | Axes Covered | Counterexample Killed |
| ------- | ------------ | --------------------- |
| [basic_admitted_config.rs](file:///Users/sac/star-toml/examples/basic_admitted_config.rs) | Sources, Type, Validation, q_config | `parse_valid_treated_as_trusted` |
| [layered_profiles.rs](file:///Users/sac/star-toml/examples/layered_profiles.rs) | Source, Layer, Type | `ambiguous_layer_order`, `unreported_layer_override` |
| [env_overrides.rs](file:///Users/sac/star-toml/examples/env_overrides.rs) | Source, Layer, Environment, Type | `env_override_without_prefix`, `env_override_not_reported` |
| [strict_unknown_fields.rs](file:///Users/sac/star-toml/examples/strict_unknown_fields.rs) | Type, Validation, q_config | `unknown_field_accepted_in_trusted_mode` |
| [exploratory_unknown_fields.rs](file:///Users/sac/star-toml/examples/exploratory_unknown_fields.rs) | Type, Validation | None (Exploratory mode bypass warning demonstration) |
| [arrays_of_tables.rs](file:///Users/sac/star-toml/examples/arrays_of_tables.rs) | Type, Validation | `unknown_field_accepted_in_trusted_mode` (Nested tables) |
| [path_policy_sandbox.rs](file:///Users/sac/star-toml/examples/path_policy_sandbox.rs) | PathPolicy, Validation | `path_traversal_accepted`, `null_byte_path_accepted` |
| [canonical_save.rs](file:///Users/sac/star-toml/examples/canonical_save.rs) | Rewrite, Validation | `nondeterministic_save`, `rewrite_without_validation` |
| [witness_and_q_config.rs](file:///Users/sac/star-toml/examples/witness_and_q_config.rs) | Witness, q_config | `witness_missing_source_digest`, `witness_nondeterministic` |
| [ocel_lifecycle_export.rs](file:///Users/sac/star-toml/examples/ocel_lifecycle_export.rs) | OCEL, wasm4pm-compat boundary | `ocel_treated_as_standing_authority` |
| [red_team_counterexamples.rs](file:///Users/sac/star-toml/examples/red_team_counterexamples.rs) | Counterexamples, Validation | `downstream_policy_inside_star_toml` |
| [dfcm_axes_matrix.rs](file:///Users/sac/star-toml/examples/dfcm_axes_matrix.rs) | All Axes | Multiple ontology violations |
| [dfcm_common_patterns.rs](file:///Users/sac/star-toml/examples/dfcm_common_patterns.rs) | Common configs | Structural configuration flaws |
