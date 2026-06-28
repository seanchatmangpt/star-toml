# star-toml E2E Test Infrastructure Specification (Revised)

This document details the comprehensive End-to-End (E2E) test plan for `star-toml`, incorporating the updated design requirements. The plan outlines the test philosophy, architecture, inventory of test cases spanning Tiers 1–4, and validation thresholds.

---

## 1. Test Philosophy

To guarantee the reliability, security, and correctness of the `star-toml` library, the E2E test plan leverages several software testing strategies:

- **Opaque-Box Testing**: Tests interact solely with the public API of `star-toml` (such as the typestate transitions, loaders, validate macros, built-in safety checkers, save functions, hooks, and trusted config loaders) without mocking internal state.
- **Category-Partition Testing**: Input spaces (TOML configuration content, environment variables, paths, and domains) are partitioned into logical categories (e.g., braced vs. bare env vars, valid vs. invalid semver formats, safe paths vs. traversal paths) to ensure all equivalence classes are checked.
- **Boundary Value Analysis (BVA)**: Focuses heavily on boundaries, such as minimum/maximum ranges (e.g., `1..=65535` for ports, integer limits, u64 boundaries for size formats, empty/none inputs, and Kelvin/host limit boundaries).
- **Pairwise Testing**: Combines features (e.g., type coercion combined with layered merging, or environment variable expansion combined with path resolution) to uncover subtle interaction defects.
- **Workload & Stress Testing**: Exercises the library with extremely large configs, deep nesting structures, and diverse UTF-8/control character sets (including null bytes) to confirm panic-safety and resource efficiency.

---

## 2. Feature Inventory

The E2E test suite targets the 7 refined features:

- **F1: Typestate Lifecycle Abstraction**: Transitions `Config<Raw>` -> `Config<Merged>` -> `Config<Deserialized<T>>` -> `Config<Validated<T>>` -> `Config<Frozen<T>>`, verifying compile-time types and runtime safety checks.
- **F2: Layered Loading & Env Overrides**: Merging Defaults, Files, Env prefix mapping (dotted paths), Type coercion, and Env var expansion `$VAR`/`${VAR}`.
- **F3: Validation Interfaces & Macros**: Procedural macro `#[derive(Validate)]`, declarative `schema!` macro, `Validate` trait implementation, and custom profile/policy validators.
- **F4: Built-in Safety & Domain Checkers**: Path traversal guards, null bytes rejection, Kelvin/host safety, semver check, range checks, and size format parsing.
- **F5: Save Functions & Serialization**: File write-back functions (`save_file`, `save_canonical`, `save_pretty`) and path resolution (`ConfigFile::resolve`).
- **F6: Lifecycle Hooks**: Custom normalization and post-deserialization validation hooks via the `ConfigLifecycle<T>` trait (`normalize`, `validate_lifecycle`).
- **F7: Trusted Loader & Analytics**: The `star_toml::trusted()` builder yielding `TrustedConfig<T>`, conformance fitness scores, FNV-1a variant fingerprints, and section grouping.

---

## 3. Test Architecture

The E2E test suite is located in the integration test file:
```text
tests/e2e_tests.rs
```
It is run using:
```bash
cargo test --test e2e_tests
```

This ensures isolation from unit tests and focuses on the composition of typestate transitions, layered configuration loading, and validation pipelines.

---

## 4. Feature Coverage Matrix (Tiers 1–3)

Below is the inventory of **84** test cases mapped across Tier 1 (Opaque-Box & BVA), Tier 2 (Edge cases & Error handling), and Tier 3 (System & API Integration).

### Tier 1: Opaque-Box & Boundary Value Analysis (BVA) (38 Cases)

| Test ID | Feature | Description |
|---|---|---|
| `T1_01` | F1 | BVA: Compile-time check verification for Typestate transitions (`Config<Raw>` must not allow deserialization before merging, etc.) and runtime invariants. |
| `T1_02` | F1 | Typestate transition: Successful sequence `Raw` -> `Merged` -> `Deserialized` -> `Validated` -> `Frozen`. |
| `T1_03` | F1 | Typestate state checks: Verifying that `Config<Frozen<T>>` is indeed immutable and guarantees no further changes. |
| `T1_04` | F2 | Category-Partition: Layered loading precedence with 4 layers (Defaults + File 1 + File 2 + Env Override). |
| `T1_05` | F2 | Category-Partition: Env prefix handling with special characters (dotted path mapping like `APP_A__B` -> `a.b`). |
| `T1_06` | F2 | Type coercion: Boolean string representations ("true", "false", "True", "FALSE"). |
| `T1_07` | F2 | Type coercion: Integer string representations ("0", "-123", "9223372036854775807"). |
| `T1_08` | F2 | Type coercion: Float string representations ("0.0", "-3.14", "1e10", "NaN", "inf"). |
| `T1_09` | F2 | Env var expansion: Empty or missing env variable expansion (`$EMPTY` / `${EMPTY}`). |
| `T1_10` | F2 | Env var expansion: Expansion of multiple sequential variables (`$A$B$C`) and brace syntax. |
| `T1_11` | F2 | Env var expansion: UTF-8 preservation during expansion with non-ASCII surrounding content. |
| `T1_12` | F3 | `#[derive(Validate)]` macro basic case: verifying generated `Validate` trait on simple struct. |
| `T1_13` | F3 | Declarative `schema!` macro: declarative validation of a basic flat TOML payload. |
| `T1_14` | F3 | `Validate` trait: direct manual implementation verification. |
| `T1_15` | F3 | Custom profile validators: profile-based conditional validation (e.g. `dev` profile allows HTTP, `prod` requires HTTPS). |
| `T1_16` | F3 | Custom policy validators: policy rules (e.g. maximum resource caps) validated via custom closures. |
| `T1_17` | F4 | Path traversal guards: check rejection of relative traversal paths escaping the root (`../../etc`). |
| `T1_18` | F4 | Null bytes rejection: rejection of any config string/value containing null bytes. |
| `T1_19` | F4 | Host safety: validating domain labels length boundary (<= 63 characters). |
| `T1_20` | F4 | Host safety: validating overall domain length boundary (<= 253 characters). |
| `T1_21` | F4 | Semver check: validating standard semver format x.y.z (0.0.0 to u32 max boundaries). |
| `T1_22` | F4 | Range check: boundary checks for port or integer ranges (off-by-one). |
| `T1_23` | F4 | Size format check: validating size string suffixes ("MB", "GB", "KB") and u64 limits. |
| `T1_24` | F5 | `save_file`: basic file serialization to disk. |
| `T1_25` | F5 | `save_canonical`: saving config in a standardized canonical key-sorted TOML format. |
| `T1_26` | F5 | `save_pretty`: saving config with pretty formatting (indentation, spacing). |
| `T1_27` | F5 | `ConfigFile::resolve`: resolving relative path when base path is absolute vs relative. |
| `T1_28` | F6 | `ConfigLifecycle::normalize`: field normalization like trimming whitespace on string fields. |
| `T1_29` | F6 | `ConfigLifecycle::validate_lifecycle`: post-deserialization lifecycle validation hook logic. |
| `T1_30` | F7 | `star_toml::trusted()`: trusted loader returns valid `TrustedConfig<T>`. |
| `T1_31` | F7 | Conformance fitness score: fitness calculation for empty checks, 0%, 50%, and 100% passed. |
| `T1_32` | F7 | Variant fingerprint: hashing sorted errors to produce stable fingerprint. |
| `T1_33` | F7 | Section grouping: `ValidationErrors::by_section` grouping for flat and nested configs. |
| `T1_34` | F4 | Host safety: Kelvin/host safety specific check (Kelvin temperature range bounds check). |
| `T1_35` | F3 | Declarative `schema!` macro: validation of nested table sections. |
| `T1_36` | F3 | Procedural `#[derive(Validate)]` macro: nested struct validate traversal. |
| `T1_37` | F7 | Conformance fitness: score when no checks are run (should be 1.0). |
| `T1_38` | F7 | Variant fingerprint: stability when error messages vary but codes/locations match. |

### Tier 2: Edge Cases & Error Handling (38 Cases)

| Test ID | Feature | Description |
|---|---|---|
| `T2_01` | F1 | Typestate: Attempting to serialize/save raw/merged configs before validation (blocked at compile time/runtime). |
| `T2_02` | F1 | Typestate: Handling validation failures during transition to `Validated`. |
| `T2_03` | F1 | Typestate: Attempting to mutate configuration after transitioning to `Config<Frozen<T>>` (runtime/compile-time enforcement). |
| `T2_04` | F2 | Layered Loading: Missing file in `layer_file` returns `Error::FileNotFound`. |
| `T2_05` | F2 | Layered Loading: Missing file in `layer_file_if_exists` is silently ignored. |
| `T2_06` | F2 | Env prefix override with nested tables (`APP_A__B__C=1` -> `a.b.c = 1`) and conflicting types. |
| `T2_07` | F2 | Type coercion: Fallback to string for unparseable scalars (like "1.2.3.4", "10GB"). |
| `T2_08` | F2 | Env var expansion: Unclosed brace in env variable expansion (`${UNCLOSED`). |
| `T2_09` | F2 | Env var expansion: Nested variable lookups (`${VAR_${SUB}}`). |
| `T2_10` | F2 | Env var expansion: Extremely long env variable values (stress test). |
| `T2_11` | F3 | `#[derive(Validate)]`: Handling structural validation on complex structures with `Option` fields. |
| `T2_12` | F3 | Declarative `schema!`: Syntax errors in schema definition or duplicate constraints on same field. |
| `T2_13` | F3 | Custom profile: Missing or unspecified profile uses fallback defaults. |
| `T2_14` | F3 | Custom policy: Multiple policies chained where intermediate policy fails. |
| `T2_15` | F4 | Path traversal: Tricky traversals that seem safe but escape via symlinks or absolute paths. |
| `T2_16` | F4 | Null bytes: Multi-byte characters containing a null byte sequence in string overrides. |
| `T2_17` | F4 | Host safety: Invalid hyphens in hostnames (`-example.com`, `example-.com`). |
| `T2_18` | F4 | Host safety: Invalid dots in hostnames (`domain..com`, `.domain.com`). |
| `T2_19` | F4 | Kelvin/host safety: Kelvin safety lower limit (0 Kelvin) boundary check. |
| `T2_20` | F4 | Semver: Prerelease tags and build metadata checking (e.g. rejection of alpha/beta if format restricts). |
| `T2_21` | F4 | Range: Extreme integer boundaries (u64 limits, negative limits). |
| `T2_22` | F4 | Size format: Invalid size format suffixes or floating numbers (e.g., `1.5GB`). |
| `T2_23` | F5 | Save functions: Saving to a path where parent directories cannot be created (permission denied). |
| `T2_24` | F5 | Save functions: Serializing complex structs that contain maps or arrays. |
| `T2_25` | F5 | `ConfigFile::resolve`: Resolving paths with multiple relative segments (`../../`). |
| `T2_26` | F6 | Lifecycle Hooks: Normalization resulting in empty string when empty check is active. |
| `T2_27` | F6 | Lifecycle Hooks: Normalization loops or conflicts between different hooks. |
| `T2_28` | F6 | Lifecycle Hooks: `validate_lifecycle` returning multiple errors. |
| `T2_29` | F7 | Trusted Loader: Loader rejects untrusted config input or invalid signature/report. |
| `T2_30` | F7 | Fitness score: Multi-error scenario verification. |
| `T2_31` | F7 | Variant fingerprint: Handling of complex nested location path hashing. |
| `T2_32` | F7 | Section grouping: Grouping of errors with no segments (root level errors mapped to `(root)`). |
| `T2_33` | F2 | Env Overrides: Conflicting keys in environment variables (case sensitivity overrides). |
| `T2_34` | F3 | Validation Macros: Macro validation failing on enum variants. |
| `T2_35` | F4 | Host safety: Domain names with non-ASCII or IDNA punycode characters. |
| `T2_36` | F5 | Save functions: Overwriting existing read-only files. |
| `T2_37` | F6 | Lifecycle Hooks: Modifying fields during `normalize` that violate validation ranges. |
| `T2_38` | F7 | Trusted Loader: Validating the `ConfigDigest` generation stability. |

### Tier 3: System-level & API Integration (8 Cases)

| Test ID | Feature | Description |
|---|---|---|
| `T3_01` | F1–F7 | Full lifecycle path: raw TOML -> merged layers -> deserialized to struct -> validated with macro -> normalized & lifecycle-hooked -> frozen -> saved to disk -> loaded via trusted loader. |
| `T3_02` | F1, F2 | Transition of typestate states in presence of complex env overrides and multi-file merging. |
| `T3_03` | F3, F6 | Procedural macro `#[derive(Validate)]` validation coexisting with `ConfigLifecycle` hooks on the same struct. |
| `T3_04` | F4, F7 | Checking how built-in safety validators (null bytes, path traversal) affect the conformance fitness and section grouping analytics of `TrustedConfig`. |
| `T3_05` | F1, F5 | Verifying save functions (`save_canonical`, `save_pretty`) enforce typestate restrictions dynamically. |
| `T3_06` | F3, F7 | Comparing results of declarative `schema!` validation and `#[derive(Validate)]` on identical inputs to ensure identical variant fingerprints. |
| `T3_07` | F2, F4 | System-level error propagation pipeline: syntax error -> `Error::Parse`, traversal safety violation -> `Error::Invalid`, missing file -> `Error::FileNotFound`. |
| `T3_08` | F1, F7 | Concurrency test: Multiple threads executing Typestate Transitions and loader checks on shared read-only resources. |

---

## 5. Real-World Application Scenarios (Tier 4)

These scenarios represent full end-to-end setups where multiple features are combined to model real-world service requirements.

### `T4_01`: Microservice Web Server Configuration
- **Scenario**: Configures database host/port, TLS, worker pool, and cache size.
- **Features Used**: F1, F2, F3, F4, F5.
- **Workflow / Checks**:
  - The configuration undergoes the full Typestate transition: loaded as `Raw`, merged with environment overrides for port (`APP_DATABASE__PORT`), deserialized into a Rust struct derived with `#[derive(Validate)]`.
  - Safety checks are executed: database host is verified via host safety domain checkers, port is validated against range `1..=65535`, TLS certificate paths are checked for path traversal and null bytes, and cache size format is validated.
  - A custom policy validator ensures that if TLS is enabled, certificate paths are non-empty.
  - The validated configuration is transitioned to `Frozen`, and then saved to disk via `save_canonical`.

### `T4_02`: CI/CD Pipeline Runner Config
- **Scenario**: Configures pipeline runner with workspace paths, execution timeout, allowed environments, engine version, and custom lifecycle hooks.
- **Features Used**: F1, F2, F3, F4, F6, F7.
- **Workflow / Checks**:
  - Raw configuration is loaded from file.
  - Env variables expand engine version and paths.
  - Undergoes Typestate transition to `Deserialized`.
  - Executes `ConfigLifecycle::normalize` to trim whitespace and standardize workspace path formats.
  - Runs procedural macro `#[derive(Validate)]` check which validates workspace path does not escape via traversal, timeout is within `check_range`, environment is `check_one_of` ("docker", "kubernetes", "local"), and engine version matches `check_semver`.
  - `star_toml::trusted()` loader yields a `TrustedConfig` checking conformance fitness score is 1.0.

### `T4_03`: Distributed Database Cluster Configuration
- **Scenario**: Configures cluster seed nodes, node roles (primary vs replica), heartbeat intervals, and security profiles.
- **Features Used**: F1, F2, F3, F4, F6.
- **Workflow / Checks**:
  - Typestate lifecycle transitions from `Raw` to `Merged`. Dotted env override maps node IP/hostnames.
  - Deserialized into a struct with custom profile validators.
  - `ConfigLifecycle::validate_lifecycle` hook is run post-deserialization.
  - Cross-field validation ensures that if the node role is "replica", seed nodes list must be non-empty and all nodes must pass host safety checks.
  - Transitioned to `Frozen` state for thread-safe access.

### `T4_04`: Data Ingestion Agent Configuration
- **Scenario**: Configures folder monitoring, log levels, file size limits, archiving paths, and error analytics.
- **Features Used**: F1, F3, F4, F5, F7.
- **Workflow / Checks**:
  - TOML payload containing invalid inputs (e.g. paths with null bytes and invalid size formats) is validated.
  - Procedural macro validation and safety checkers reject path traversals and invalid file sizes.
  - The errors are analyzed: `ValidationErrors::by_section` groups the failures by the ingest/archive sections, and a variant fingerprint (using FNV-1a) is generated to identify the error pattern.
  - Once corrected, the config is saved using `save_pretty`, and paths are resolved using `ConfigFile::resolve`.

### `T4_05`: API Gateway Config with Rate Limiter
- **Scenario**: Configures rate limits, endpoint paths, default headers, and target backend hostnames.
- **Features Used**: F1, F2, F3, F4, F7.
- **Workflow / Checks**:
  - Layered configuration merges default settings, a config file, and env overrides.
  - Target backend hostnames are validated against domain/IP guards.
  - The configuration loader uses `star_toml::trusted()` to return a `TrustedConfig` that exposes the `ConfigSourceReport`, `ValidationReport`, and `ConfigDigest` to guarantee integrity before launching the gateway.

---

## 6. Coverage Thresholds

To pass the test verification suite, the actual test executions must satisfy the following case counts:

- **Tier 1 (Opaque-Box & BVA)**: $\ge 35$ cases
- **Tier 2 (Edge cases & Error handling)**: $\ge 35$ cases
- **Tier 3 (System & API integration)**: $\ge 7$ cases
- **Tier 4 (Real-World Application Scenarios)**: $\ge 5$ scenarios
- **Total Cases**: $\ge 82$ cases
