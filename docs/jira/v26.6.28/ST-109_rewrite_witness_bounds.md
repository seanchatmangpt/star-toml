# ST-109: Rewrite and Witness Bounds

## Description
Implements the Rewrite and Witness bounds of the configuration admission lifecycle. This story ensures that configurations are serialized deterministically, comment-preserving modifications are verified via round-trip proofs, and cryptographic witnesses are generated to verify the integrity and provenance of all configuration layers, environments, validation reports, and canonical outputs.

## Key Requirements

### 1. Rewrite Bounds (`star_toml::rewrite`)
- **Deterministic Canonical Writes (`save_canonical`)**:
  - Any configuration written via `save_canonical` must guarantee identical byte outputs for identical inputs.
  - Implement recursive alphabetical sorting of all keys in all tables and nested structures.
  - Reject writes of raw or unvalidated configurations. The save interfaces must strictly consume `Config<Validated<T>>` or `Config<Frozen<T>>` to prevent unvalidated changes from being persisted (`rewrite_without_validation`).
- **Comment-Preserving Edits**:
  - The framework must support modifying or updating specific configuration fields while preserving existing developer comments.
  - Any comment-preserving edit operation must perform a **parser round-trip proof**. If the proof cannot be verified, the operation must fail with a `CommentPreservationClaimUnproven` error.
  - The round-trip verification procedure is defined as:
    1. Parse the edited document content back into an AST / memory representation.
    2. Strip all comments from both the original modified memory state and the newly parsed state.
    3. Compare the semantic config values of the two stripped structures to verify they are equivalent.
    4. Verify that comments are mapped correctly to their original keys, and no comments have been silently dropped or displaced.

### 2. Witness Bounds (`star_toml::witness`)
- **Cryptographic Config Witness (`ConfigWitness`)**:
  - Generate a stable, cryptographic witness representing the complete configuration state and historical loading context.
  - The witness is defined mathematically as:
    $$ConfigWitness = H(B_{config}, Sources, Layers, Env, Validation, CanonicalOutput)$$
    where $H$ is a stable cryptographic hashing function (such as SHA-256).
  - The witness must aggregate and hash the following boundaries and metadata structures:
    - **Boundaries ($B_{config}$)**: Hashing of schemas, allowed sources, path restrictions, and environment prefixes.
    - **Sources**: Metadata reports containing paths, load status, and SHA-256 digests of all loaded configuration files (`witness_missing_source_digest`).
    - **Layers**: The deterministic merge order of configuration layers and their winning-layer trace history.
    - **Environment Overrides**: A deterministic map of active environment overrides, their raw values, and coerced target types (`witness_missing_env_report`).
    - **Validation Reports**: Conformance history, checks run, fitness score, and any validation errors (`witness_missing_validation_report`).
    - **Canonical Output**: The final serialized canonical TOML byte sequence.
- **Determinism Guarantee**:
  - The witness hash calculation must be completely deterministic. Given identical inputs, environment state, and validation reports, the resulting `ConfigWitness` must remain identical (`witness_nondeterministic`). All hash inputs must be stably sorted (e.g. environment variable maps sorted by key) before being fed into the hasher.

---

## Counterexamples Covered

- **`nondeterministic_save`**: Saving map/table values without strict recursive key sorting, causing differences in serialization across runs.
- **`comment_preservation_claim_unproven`**: Performing comment-preserving configuration updates without validating structural equivalence via round-trip parsing, leading to silent comment loss.
- **`rewrite_without_validation`**: Allowing unvalidated or Raw configurations to be saved canonical, bypassing semantic and schema validators.
- **`witness_missing_source_digest`**: Omitting the file content hash or source metadata from the cryptographic witness hash.
- **`witness_missing_env_report`**: Omitting active environment variable overrides from the witness, allowing silent environmental injection.
- **`witness_missing_validation_report`**: Omitting the validation errors or conformance records from the witness, allowing invalid configurations to produce the same witness as valid ones.
- **`witness_nondeterministic`**: Producing different witness hashes for identical config operations due to unstable map ordering or non-reproducible values.

---

## Acceptance Criteria

- [ ] `save_canonical` outputs TOML files with keys sorted alphabetically at every level of nested tables.
- [ ] Attempting to write back a configuration directly from unvalidated typestates (e.g. `Config<Raw>` or `Config<Deserialized<T>>`) is rejected at compile time.
- [ ] Comment-preserving writes check for semantic equivalence by stripping comments and verifying that the structure remains unchanged.
- [ ] If comments are lost or modified unexpectedly, the edit action fails with `CommentPreservationClaimUnproven`.
- [ ] `ConfigWitness` aggregates and hashes:
  - Source file paths and content digests.
  - Active environment override keys and values.
  - Validation metrics (fitness, checks run) and error logs.
  - The final canonical output bytes.
- [ ] Generating a witness twice on identical configuration structures and environments yields the identical hash value.
- [ ] Modifying a source file, modifying an environment variable, or modifying validation results changes the generated `ConfigWitness` hash.

---

## Verification Method

### 1. Unit Tests
- **Key Sorting Test**: Create a TOML document with out-of-order keys and verify that `save_canonical` generates a byte-identical sorted output regardless of the initial insertion order.
- **State Transition Compile Check**: Implement static asserts to confirm that the `save_canonical` method is only implemented for `Config<Validated<T>>` and `Config<Frozen<T>>`.
- **Deterministic Hashing Test**: Run the witness generation multiple times on the same configuration inputs and verify that the output hash is stable and identical.

### 2. Integration Tests
- **Round-Trip Comment Proof**: Perform a comment-preserving edit on a sample configuration, verify it succeeds, and verify that injecting an invalid comment edit that alters values or drops comments triggers `CommentPreservationClaimUnproven`.
- **Witness Variance Test**: Load a configuration, generate its witness, and assert that:
  - Changing a source file digest changes the witness.
  - Changing an environment override variable changes the witness.
  - Changing validation errors changes the witness.

### 3. Stress & Chaos Tests
- **Precedence & Key Sort Chaos**: Randomly shuffle table keys in memory and verify that `save_canonical` produces identical, stable disk writes.
