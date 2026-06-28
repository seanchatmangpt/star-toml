# Worker Task: Validation Macros

## Objective
Implement `#[derive(Validate)]` procedural macro (in a new `star-toml-derive` sub-crate) and the declarative `schema!` macro in `src/schema.rs` (Milestone I3).

## Details
1. **Procedural macro `#[derive(Validate)]`**:
   - Create a sub-crate directory `star-toml-derive` at the project root.
   - Configure `star-toml-derive/Cargo.toml` as a `proc-macro = true` library, with dependencies `syn = { version = "2.0", features = ["full"] }` and `quote = "1.0"`.
   - Implement `#[derive(Validate)]` which:
     - Inspects each field of the struct.
     - If a field is annotated with `#[validate]`, it generates descent validation code:
       - If the field's type path contains `Option`, it wraps it in `if let Some(ref inner) = self.field { v.field("field", |v| inner.validate(v)); }`.
       - If the field's type path contains `Vec`, it wraps it in `v.field("field", |v| { for (i, x) in self.field.iter().enumerate() { v.index(i, |v| x.validate(v)); } })`.
       - Otherwise, it generates `v.field("field", |v| self.field.validate(v))`.
   - Update `Cargo.toml` at the project root to depend on `star-toml-derive`.
   - Update `src/lib.rs` to re-export `star-toml-derive::Validate` so that users can use `#[derive(Validate)]` directly.

2. **Declarative `schema!` macro**:
   - Implement the `schema!` macro in `src/schema.rs` (and export it from `src/lib.rs`).
   - The macro should allow declaring a `Schema` with fields, constraints, and nested sections:
     ```rust
     let s = schema! {
         name: non_empty,
         port: [required, range(1, 65535)],
         server: {
             host: non_empty,
         }
     };
     ```
   - Support constraints: `required`, `non_empty`, `range(lo, hi)`, `range_f64(lo, hi)`, and `one_of(...)`.
   - Support both identifier keys (like `port`) and string literal keys (like `"port"`).

3. **Profile and Policy Validator Helpers**:
   - In `src/validation.rs`, implement helper methods on `Validator`:
     - `check_profile(&mut self, field: &str, active_profile: &str, target_profile: &str, condition: bool, code: &'static str, msg: impl Into<String>)`
     - `check_policy<F>(&mut self, field: &str, policy_closure: F, code: &'static str, msg: impl Into<String>) where F: FnOnce() -> bool`

4. **Verify**:
   - Add unit tests verifying `#[derive(Validate)]` traversal, `schema!` validation, and the profile/policy validators.
   - Run `cargo check` and `cargo test` to ensure everything compiles and passes.
