# How to Verify Release Readiness

This guide outlines steps to confirm the codebase satisfies the release criteria before tagging and publishing.

## Goal

Run the global release verifier and verify that the failset of active caveats is empty.

## Implementation Steps

### 1. Compile & Format Check

Format files and run general targets:

```bash
cargo fmt --check
cargo check --all-targets
```

### 2. Run Test Suites

Run standard test blocks:

```bash
cargo test
```

### 3. Run Release Verifier

Execute the verifier gate tool:

```bash
cargo run --bin release_verifier
```

To run during development with uncommitted or dirty git statuses, specify the dev mode flag:

```bash
cargo run --bin release_verifier -- --dev
```

Ensure the exit code is `0` and output ends with `Final: ADMITTED`.
