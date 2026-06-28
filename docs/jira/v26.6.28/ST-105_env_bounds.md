# ST-105: Environment Bounds

## Description
This ticket defines the requirements, acceptance criteria, and verification methods for enforcing strict Environment Bounds on the Star TOML configuration substrate. Environment overrides must be tightly bounded to prevent ambient environment variables from polluting the configuration space or bypassing the audit trail.

## Key Requirements
1. **Prefixed Variables**:
   - Environment variable overrides are only admitted if they start with a designated uppercase prefix (e.g., `STAR_`).
   - Any attempt to feed unprefixed environment overrides or any ambient presence of unprefixed overrides trying to modify configuration keys must fail config admission.
2. **Double-Underscore Nesting**:
   - Support nested configuration paths using double-underscore (`__`) as a key separator (e.g., `STAR_DB__PORT` must map to the key path `db.port`).
   - Single underscores represent normal word separators within a single key level (e.g., `STAR_DB_SETTINGS__MAX_CONN` maps to `db_settings.max_conn`).
3. **Data Coercion**:
   - Environment variables (which are natively strings) must be strictly coerced to match the target TOML/JSON scalar types defined by the schema: `String -> Bool | Integer | Float | String`.
   - String values like `"true"` and `"false"` (case-insensitive) coerce to booleans `true` and `false`.
   - String values containing valid integers coerce to integers.
   - String values containing valid floating-point representation coerce to floats.
   - If a type coercion fails (e.g. attempting to override an integer field with a non-numeric string), config admission must fail immediately.
4. **Failing on Unprefixed or Unreported Overrides**:
   - Any active environment override must be reported in the final `EnvOverrideReport` and captured in the cryptographic configuration witness.
   - If an environmental override is detected or applied but not reported in the witness or layer history, the config must be rejected as inadmissible (`UnreportedOverride` or equivalent error).
   - Unprefixed variables matching configuration keys that are not explicitly authorized/prefixed must fail admission immediately to avoid silent pollution.

## Acceptance Criteria
- [ ] Environment overrides starting with the designated prefix (e.g., `STAR_`) are correctly identified, while all other ambient environment variables are ignored or cause admission failure if they attempt to override.
- [ ] Variables using double-underscore `__` separator correctly map to nested configuration tables (e.g., `STAR_A__B__C` to `a.b.c`).
- [ ] String-to-type coercion converts strings like `"100"` to integer `100`, `"true"` to boolean `true`, and `"3.14"` to float `3.14`.
- [ ] Type coercion failure (e.g., overriding a boolean field with `"not-a-bool"`) halts the admission process and returns a validation error.
- [ ] All successfully applied environment overrides are explicitly listed in the final `EnvOverrideReport` output.
- [ ] Any override that is applied but omitted from the audit trail or the cryptographic configuration witness triggers an immediate admission error.
- [ ] The presence of unprefixed environment variables matching config keys triggers a hard admission failure rather than being silently ignored.

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
  - Inject environment variables that match target fields but omit the prefix (e.g., `DB__PORT` instead of `STAR_DB__PORT`) and assert that the system rejects the configuration.
  - Simulate a situation where an override is applied but manually stripped from the report/witness, and assert that the validator catches and rejects the state.
