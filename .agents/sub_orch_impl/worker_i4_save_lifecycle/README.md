# Worker Task: Save & Lifecycle Hooks

## Objective
Implement canonical saving, pretty saving, and `ConfigLifecycle` hooks (Milestone I4).

## Details
1. **Canonical & Pretty Saving**:
   - Expose free function `pub fn save_pretty<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()>` which serializes using pretty-printing (`toml::to_string_pretty`) and writes to the file.
   - Modify `save_file` to use standard serialization (`toml::to_string`) without pretty-printing, and write to the file.
   - Implement `save_canonical(&self, path: impl AsRef<Path>) -> Result<()>` on `Config<Frozen<T>>` and `Config<Validated<T>>`.
     - Standardize/canonicalize: serialize `T` to a `toml::Value`, recursively sort the keys of all tables and nested tables alphabetically, serialize that sorted `toml::Value` using standard `toml::to_string`, and write it to the file.
     - Ensure parent directories are created as needed.

2. **ConfigLifecycle Trait**:
   - Define the trait `ConfigLifecycle` in `src/loader.rs`:
     ```rust
     pub trait ConfigLifecycle {
         fn normalize(&mut self) {}
         fn validate_lifecycle(&self, _v: &mut Validator) {}
     }
     ```
   - Re-export `ConfigLifecycle` in `src/lib.rs`.
   - Update the typestate transitions:
     - `Config<Merged>::deserialize<T: DeserializeOwned + ConfigLifecycle>(self)` must deserialize `T`, call `value.normalize()`, and return `Config<Deserialized<T>>`.
     - `Config<Deserialized<T>>::validate(self)` must run `self.state.value.check()`, run `self.state.value.validate_lifecycle(&mut v)`, collect all errors, and fail with `Error::Invalid` if any errors (with severity >= Error) are found.
   - Update `TrustedLoader::load`:
     - Add `ConfigLifecycle` bound to `T`.
     - Call `value.normalize()` after deserializing.
     - Call `value.validate_lifecycle(&mut v)` during validation, accumulating any errors.

3. **Verify**:
   - Add unit tests for `save_pretty`, `save_file`, and `save_canonical` (including verifying that key sorting works recursively).
   - Add unit tests for `ConfigLifecycle` normalization (e.g. trimming string fields) and post-deserialization validation hooks.
   - Run `cargo check` and `cargo test` to ensure everything compiles and passes cleanly.
