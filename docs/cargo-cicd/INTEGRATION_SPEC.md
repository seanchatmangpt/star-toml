# Integration Specification: cargo-cicd & star-toml

This specification details the interface boundaries, control flow, and cryptographic data exchange between the `cargo-cicd` orchestrator and the `star-toml` admission substrate.

## 1. Interface Boundary

`cargo-cicd` relies entirely on `star-toml` to enforce security rules and filesystem sandboxing.

```
                  ┌──────────────────────┐
                  │    cargo-cicd CLI    │
                  └──────────┬───────────┘
                             │
            Uses TrustedLoader to parse config
                             │
                             ▼
                  ┌──────────────────────┐
                  │   star-toml Core     │
                  └──────────┬───────────┘
                             │
             Validates schema and paths
                             │
                             ▼
                  ┌──────────────────────┐
                  │  AdmittedConfig<T>   │
                  └──────────┬───────────┘
                             │
               Extracts admitted config facts
                             │
                             ▼
                  ┌──────────────────────┐
                  │  cargo-cicd Executor │
                  └──────────────────────┘
```

## 2. Control Flow & Error Isolation

- **Config Validation Failure**: If validation fails, `star-toml` accumulates diagnostics and interrupts the execution of the command. `cargo-cicd` converts these diagnostics into formatting outputs and exits with non-zero code.
- **Path Traversal Escape**: Path policy validation occurs during the ingestion stage before parsing. Sandbox violations trigger immediate process abortion with `PathTraversalAccepted` failure codes.
- **OCEL Observability Integration**: `cargo-cicd` intercepts events inside the loader pipeline and maps them directly to the Object-Centric Event Log (`OcelLog`) for recording task durations, command arguments, and outcomes.

## 3. Receipt Verification Loop

To achieve "production impossibility" (Principle 2), `cargo-cicd` executes verification of receipts before starting deployment tasks.

1. **Read Task Receipt**: Locates the cryptographic task receipt from `target/cicd/receipts/`.
2. **Recompute Digests**: Computes BLAKE3 digests of the input environment configuration, source artifacts, and executable code binary.
3. **Verify Binding**: Asserts that the computed digests match the hashes recorded inside the receipt.
4. **Grant Standing**: If signatures and digests align, the task is marked as `ADMITTED` and the deployment proceeds; otherwise, execution is `REFUSED`.
