# Verification and Testing Plan

This plan details the steps to test adversarial edge cases, verify coverage, and document any identified gaps/bugs in the `star-toml` library.

## Step 1: Baseline Verification
Run the existing test suite with `--all-features` to ensure everything is currently passing.
- Command: `cargo test --all-features`

## Step 2: Implement Adversarial Tests
Append the following adversarial tests to `tests/adversarial.rs`:
1. `test_schema_missing_nested_section_adversarial`: Tests the missing nested section bug in `Schema::report_section_missing`.
2. `test_schema_range_f64_nan_discrepancy`: Tests the f64 range validation discrepancy with `NaN` float values.
3. `test_env_prefix_consecutive_trailing_leading_dots`: Tests edge cases of environment variable prefix overrides containing trailing/consecutive double underscores.

## Step 3: Run Tests and Verify Results
Run the test suite again to verify that all tests, including the new ones, pass cleanly.
- Command: `cargo test --all-features`

## Step 4: Analyze and Document Gaps
Document the observed behaviors, logic chains, and findings in `handoff.md`.
