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
  - Comment-preserving writes are not implemented in the core engine; a 'no comment-preservation claim' is maintained to handle the detector check (which triggers `CommentPreservationClaimUnproven` or returns a statically evaluated check success).

### 2. Witness Bounds (`star_toml::witness`)
- **Cryptographic Config Witness (`ConfigWitness`)**:
  - Generate a stable, cryptographic witness representing the complete configuration state and historical loading context.
  - The witness is defined mathematically as:
    $$ConfigWitness = \text{BLAKE3}(SourcePart \mid LayerPart \mid EnvPart \mid FitnessPart \mid CanonicalPart)$$
    where:
    - $SourcePart$ is the comma-joined list of source file BLAKE3 digests sorted by `source_id`.
    - $LayerPart$ is the last `layer_order_digest` from the layer report (or an empty string if empty).
    - $EnvPart$ is the comma-joined list of active and accepted environment variable overrides, sorted alphabetically, each formatted as `"{key}={path}:{value_digest}"`.
    - $FitnessPart$ is the validation fitness score formatted as a string with exactly 6 decimal places (e.g. `1.000000`). Note: Due to typestate boundaries, the validation report details are consumed prior to this transition, and the fitness score is hardcoded to `1.000000` (representing total passing conformance) at witness generation time.
    - $CanonicalPart$ is the BLAKE3 hex digest of the canonical TOML output bytes.
    - $\mid$ denotes the pipe (`|`) character used as a separator.
  - The witness aggregates and hashes the following boundaries and metadata structures:
    - **Sources**: Metadata reports containing paths, load status, and BLAKE3 digests of all loaded configuration files (`witness_missing_source_digest`).
    - **Layers**: The deterministic merge order of configuration layers and their winning-layer trace history.
    - **Environment Overrides**: A deterministic map of active and accepted environment overrides, raw values, and mapped paths (`witness_missing_env_report`).
    - **Validation Reports**: Conformance history represented by the validation fitness score (`witness_missing_validation_report`).
    - **Canonical Output**: The final serialized canonical TOML byte sequence (hashed using BLAKE3).
- **Determinism Guarantee**:
  - The witness hash calculation must be completely deterministic. Given identical inputs, environment state, and validation reports, the resulting `ConfigWitness` must remain identical (`witness_nondeterministic`). All hash inputs must be stably sorted (e.g. environment variable maps sorted by key) before being fed into the hasher.

---

## Counterexamples Covered

- **`nondeterministic_save`**: Saving map/table values without strict recursive key sorting, causing differences in serialization across runs.
- **`comment_preservation_claim_unproven`**: Performing comment-preserving configuration updates without validating structural equivalence via round-trip parsing, leading to silent comment loss.
- **`rewrite_without_validation`**: Allowing unvalidated or Raw configurations to be saved canonical, bypassing semantic and schema validators.
- **`witness_missing_source_digest`**: Omitting the file content hash or source metadata from the cryptographic witness hash.
- **`witness_missing_env_report`**: Omitting active environment variable overrides from the witness, allowing silent environmental injection.
- **`witness_missing_validation_report`**: Omitting the validation fitness score from the witness, allowing invalid configurations to produce the same witness as valid ones.
- **`witness_nondeterministic`**: Producing different witness hashes for identical config operations due to unstable map ordering or non-reproducible values.

---

## Acceptance Criteria

- [x] `save_canonical` outputs TOML files with keys sorted alphabetically at every level of nested tables.
- [x] Attempting to write back a configuration directly from unvalidated typestates (e.g. `Config<Raw>` or `Config<Deserialized<T>>`) is rejected at compile time.
- [x] Comment-preserving writes are explicitly marked as unproven/deferred.
- [x] Comment-preserving writes are explicitly marked as unproven/deferred.
- [x] `ConfigWitness` aggregates and hashes:
  - Source file paths and content digests.
  - Active environment override keys and values.
  - Validation metrics (fitness score).
  - The final canonical output bytes.
- [x] Generating a witness twice on identical configuration structures and environments yields the identical hash value.
- [x] Modifying a source file, modifying an environment variable, or modifying validation results changes the generated `ConfigWitness` hash.

---

## Verification Method

### 1. Unit Tests
- **Key Sorting Test**: Create a TOML document with out-of-order keys and verify that `save_canonical` generates a byte-identical sorted output regardless of the initial insertion order.
- **State Transition Compile Check**: Implement static asserts to confirm that the `save_canonical` method is only implemented for `Config<Validated<T>>` and `Config<Frozen<T>>`.
- **Deterministic Hashing Test**: Run the witness generation multiple times on the same configuration inputs and verify that the output hash is stable and identical.

### 2. Integration Tests
- **Round-Trip Comment Proof**: Verify that comment preservation is explicitly marked as unproven/deferred.
- **Witness Variance Test**: Load a configuration, generate its witness, and assert that:
  - Changing a source file digest changes the witness.
  - Changing an environment override variable changes the witness.
  - Changing validation errors changes the witness.

### 3. Stress & Chaos Tests
- **Precedence & Key Sort Chaos**: Randomly shuffle table keys in memory and verify that `save_canonical` produces identical, stable disk writes.
