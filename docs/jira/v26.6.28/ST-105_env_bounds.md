# ST-105: Environment Bounds

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for enforcing strict Environment Bounds on the Star TOML configuration substrate. Environment overrides must be tightly bounded to prevent ambient environment variables from polluting the configuration space or bypassing the audit trail.

## Key Requirements
1. **Prefixed Variables**:
   - Environment variable overrides are only admitted if they start with a designated uppercase prefix (e.g., `STAR_`).
   - Any environment variable overrides that do not start with the prefix are ignored.
2. **Double-Underscore Nesting**:
   - Support nested configuration paths using double-underscore (`__`) as a key separator (e.g., `STAR_DB__PORT` must map to the key path `db.port`).
   - Single underscores represent normal word separators within a single key level (e.g., `STAR_DB_SETTINGS__MAX_CONN` maps to `db_settings.max_conn`).
3. **Data Coercion**:
   - Environment variables (which are natively strings) must be strictly coerced to match the target TOML/JSON scalar types defined by the schema: `String -> Bool | Integer | Float | String`.
   - String values like `"true"` and `"false"` (case-insensitive) coerce to booleans `true` and `false`.
   - String values containing valid integers coerce to integers.
   - String values containing valid floating-point representation coerce to floats.
   - If a type coercion fails (e.g. attempting to override an integer field with a non-numeric string), it falls back to string Value, triggering type mismatch during Serde deserialization.
4. **Failing on Unprefixed or Unreported Overrides**:
   - Any active environment override must be reported in the final `EnvOverrideReport` and captured in the cryptographic configuration witness.
   - The loader is correct-by-construction: the code applying overrides automatically logs to the winner map, reports, and witness.
   - Unprefixed variables are filtered out and ignored.

## Acceptance Criteria
- [x] Environment overrides starting with the designated prefix (e.g., `STAR_`) are correctly identified, while all other ambient environment variables are ignored.
- [x] Variables using double-underscore `__` separator correctly map to nested configuration tables (e.g., `STAR_A__B__C` to `a.b.c`).
- [x] String-to-type coercion converts strings like `"100"` to integer `100`, `"true"` to boolean `true`, and `"3.14"` to float `3.14`.
- [x] Type coercion failure (e.g., overriding a boolean field with `"not-a-bool"`) triggers type mismatch during Serde deserialization.
- [x] All successfully applied environment overrides are explicitly listed in the final `EnvOverrideReport` output.
- [x] Applied overrides are correct-by-construction registered in the audit trail and witness.
- [x] Ambient environment variables are ignored.

## Counterexamples Covered
- `env_override_without_prefix`: Ambient environment variables polluting the config space or modifying configuration values without the explicit prefix requirement.
- `env_override_not_reported`: Environmental overrides applied to the final configuration but omitted from the audit/report/witness metadata, bypassing security tracking.

## Verification Method
- **Unit Tests**:
  - Test double-underscore splitting and nesting logic for arbitrary depth.
  - Test strict data coercion rules across all supported scalar types (booleans, integers, floats, strings) and ensure malformed strings trigger coercion failures.
  - Test prefix detection rules to confirm correct isolation of variables.
- **Integration Tests**:
  - Initialize a configuration, set environment variables with and without the designated prefix, and verify that only prefixed variables modify the config.
  - Verify that the resulting `EnvOverrideReport` lists all and only the active env overrides.
  - Verify that modifying an environment variable causes the cryptographic witness hash to change.
- **Chaos & Failure Injection Tests**:
  - Set unprefixed environment variables (e.g., `DB__PORT` instead of `STAR_DB__PORT`) and verify that they have no effect on the configuration.
  - Assert that applied overrides are recorded in the tracing history.
