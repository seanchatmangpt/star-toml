# Original User Request

## Initial Request — 2026-06-27T22:07:17Z

Harden, test, and release `star-toml` version `26.6.27` as a trusted configuration substrate, ensuring robust layering, environment variable override resolution, path-safe validation, and saving capabilities.

Working directory: `/Users/sac/star-toml`
Integrity mode: development

## Requirements

### R1. Version Bump
Bump the crate version in `Cargo.toml` to `26.6.27`.

### R2. Deterministic Layering & Merging
Support layered loading (Defaults < Files < Env) with explicit precedence, recursive table merging, and environment override resolution (prefix filtering, double-underscore nesting, and scalar coercion).

### R3. Core Validation Engine & Trait
Support validation on loaded TOML configs via both the imperative struct-based `Validate` trait (with nested paths, custom rules) and declarative structure-less schema-based validation. Maintain Pydantic-style multi-error reports with fitness metrics, variant fingerprints, and repair hints.

### R4. Path & Host Safety
Guard against path traversal (reject null bytes, `..`, and normalized backslashes) and invalid domain hostnames (IP validation, length checks, Kelvin sign casing normalization safety).

### R5. Deterministic Save & Paths
Provide deterministic TOML output saving (without comment preservation, clearly documented) and source-relative path resolution.

### R6. Local Release Validation
Validate crate structure, formatting, linting, and verify tests pass:
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all`
- `cargo publish --dry-run`

## Acceptance Criteria

### Verification & Dry Run
- [ ] `cargo test --all` passes successfully with all tests green.
- [ ] `cargo clippy` runs clean with zero warnings or errors.
- [ ] `cargo fmt` checks out successfully.
- [ ] `cargo publish --dry-run` runs successfully without failures.
- [ ] Documentation clearly documents the library scope, layered order, env prefix rules, validate/load_validated paths, non-comment-preserving save_file behavior, and path safety limits.
- [ ] Test coverage includes missing/required file loading, layer precedence, env double underscore nesting, env scalar coercion (bool/int/float/string), nested validate error paths, fitness metric validation, variant ID stability, section grouping, repair hints, traversal checks, null bytes checks, source-relative path resolution, and save file parent directory creation + round-tripping.

## Follow-up — 2026-06-27T22:10:10Z

Important Update: The user has refined the design and release scope for star-toml v26.6.27. Please update the project plan, briefing files, and sub-orchestrators to incorporate these requirements:

1. **Typestate Lifecycle Abstraction**: Implement Config<Raw>, Config<Merged>, Config<Deserialized<T>>, Config<Validated<T>>, Config<Frozen<T>> so that config loading and saving are typestate-safe. Saving requires a frozen or validated config.
2. **Trust Path Convenience**: Expose `star_toml::trusted()` returning `TrustedConfig<T>` containing value, inspectable `ConfigSourceReport`, `ValidationReport`, and digest. Expose environment variable keys applied/seen and coerced types in the source report.
3. **Derive Macro `#[derive(Validate)]`**: Provide validation macro with attribute checks like range, one_of, non_empty, path safety, and eq checks.
4. **Schema Macro**: Implement `schema! { ... }` macro for declarative validation.
5. **Canonical Saving**: Separate save_file, save_pretty, and save_canonical (documented as deterministic but not comment-preserving).
6. **Pure Lifecycle Hooks & Normalization**: Introduce a `ConfigLifecycle<T>` trait (with normalize and validate_lifecycle).
7. **Profile & Policy Helpers**: Custom validation checkers for profiles map checks and policy field values.
8. **Security and Host Safety**: Add traversal tests, null byte tests, and host/domain DNS safety checks.

The final prompt is detailed in prompt_draft.md. Please update your sub-agents and target files to align with this release scope.
