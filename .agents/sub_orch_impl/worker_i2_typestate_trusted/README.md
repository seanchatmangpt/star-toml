# Worker Task: Typestate & Trusted Config

## Objective
Implement `Config<S>` typestate lifecycles and `trusted()` builder (Milestone I2) in `src/loader.rs` and `src/lib.rs`.

## Details
1. Implement states for the typestate lifecycle:
   - `Raw` (contains `toml::Value`)
   - `Merged` (contains `toml::Value`)
   - `Deserialized<T>` (contains `T`)
   - `Validated<T>` (contains `T`)
   - `Frozen<T>` (contains `T`)

2. Implement `Config<S>` struct containing the state and the path of the last loaded file. Expose transition methods:
   - `Loader::load_raw() -> Result<Config<Raw>>` - merges layers (without environment overrides) and returns `Config<Raw>`.
   - `Config<Raw>::merge(self, env_prefix: Option<&str>) -> Result<Config<Merged>>` - applies environment overrides.
   - `Config<Merged>::deserialize<T: DeserializeOwned>(self) -> Result<Config<Deserialized<T>>>` - deserializes into `T`.
   - `Config<Deserialized<T>>::validate(self) -> Result<Config<Validated<T>>>` - runs validation checks and fails with `Error::Invalid` if validation fails.
   - `Config<Validated<T>>::freeze(self) -> Config<Frozen<T>>` - freezes the configuration.

3. Implement `star_toml::trusted()` returning a `TrustedLoader` builder.
   - It should have the same builder methods as `Loader` (`layer_str`, `layer_file`, `layer_file_if_exists`, `find_file`, `env_prefix`).
   - `.load::<T: DeserializeOwned + Validate>()` returns `Result<TrustedConfig<T>, Error>`.
   - `TrustedConfig<T>` must contain:
     - `value: T`
     - `source: ConfigSourceReport` (containing the path)
     - `validation: ValidationReport` (containing fitness, checks_run, checks_passed, errors)
     - `digest: ConfigDigest` (containing u64 hash of the merged TOML representation, using FNV-1a).

4. Add tests to verify typestate transition compiling and validation behavior.
5. Verify `cargo check` and `cargo test` pass.
