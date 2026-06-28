# Worker Task: E2E Test Run & Safety Validation Hardening

## Objective
Verify that all E2E tests compile and pass, and harden the safety validation rules (path traversal, Kelvin, host safety) if there are any gaps (Milestone I5).

## Details
1. **Run E2E Tests**:
   - Run `cargo test --features e2e_tests` in `/Users/sac/star-toml` to verify that all 89 E2E tests pass successfully.
2. **Verify Safety Rules**:
   - Verify that safety validation rules for path traversal (blocking `..` traversal sequences on all OS platforms), null bytes, and DNS hostnames (length limits, label limits, valid characters) are fully hardened.
3. **Run Clippy / Fmt**:
   - Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`.
4. **Dry-Run Publish**:
   - Run `cargo publish --dry-run` to make sure the package is ready for release.
