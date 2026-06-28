# Worker Task: Safety & Release Hardening (Bug Fixes)

## Objective
Fix the three bugs identified by the challenger and update `tests/adversarial.rs` to assert the correct, bug-free behavior.

## Details
1. **Bug 1: Missing recursion in `Schema::report_section_missing`**
   - In `src/schema.rs`, update `report_section_missing` to recursively call `report_section_missing` on all sub-sections in `self.sections`.
   - Update `test_schema_missing_nested_section_adversarial` in `tests/adversarial.rs` to assert that `locs1` contains `server.tls.client_cert`.

2. **Bug 2: RangeF64 NaN validation discrepancy**
   - In `src/schema.rs`, in `Constraint::RangeF64` check, ensure that `n.is_nan()` triggers a range check failure (e.g. `n.is_nan() || n < *lo || n > *hi`).
   - Update `test_schema_range_f64_nan_discrepancy` in `tests/adversarial.rs` to assert that `schema.validate_str("ratio = nan")` is `Err`.

3. **Bug 3: Dot handling anomalies in environment prefix mapping**
   - In `src/merge.rs`, rewrite `set_dotted` to split path on dots, filter out empty segments (leading, trailing, consecutive), and recursively build the path using segments.
   - Update `test_env_prefix_consecutive_trailing_leading_dots` in `tests/adversarial.rs` to assert that loading succeeds and sets the correct values for both cases.

4. **Verify**:
   - Run `cargo check --all-targets --all-features` and `cargo test --all-targets --all-features` to verify that everything compiles and passes cleanly.
