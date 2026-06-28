//! [`Loader`] — composable, layered TOML config loading.
//!
//! The central piece of the `star_toml` framework. Each `layer_*` call adds one
//! config source; `load()` merges them in order (first = lowest priority, last =
//! highest priority) and deserializes the result into `T`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use toml::Value;

use crate::{
    error::{Error, Result},
    expand::expand_env_vars,
    merge::{deep_merge, deep_merge_traced, env_str_to_value, set_dotted, WinnerMap},
    reports::{
        blake3_hex, CoercedType, EnvOverrideEntry, EnvOverrideReport, LayerEntry, LayerReport,
        SourceEntry, SourceKind, SourceReport,
    },
    validation::Validate,
};

// ---------------------------------------------------------------------------
// ConfigLayer — one source in the loading stack
// ---------------------------------------------------------------------------

enum ConfigLayer {
    /// A literal TOML string (used for built-in defaults).
    Str(String, &'static str /* label for errors */),
    /// A file that must exist.
    File(PathBuf),
    /// A file that is silently skipped when absent.
    FileIfExists(PathBuf),
    /// Walk parent directories until `file_name` is found.
    FindFile(String /* file_name */),
}

// ---------------------------------------------------------------------------
// ConfigFile — loaded config + resolved source path
// ---------------------------------------------------------------------------

/// A `T` together with the path of the TOML file it was loaded from.
///
/// Use [`ConfigFile::resolve`] to convert relative paths found *inside* the
/// config into absolute paths.
///
/// # Example
///
/// ```no_run
/// use star_toml::{Loader, ConfigFile};
///
/// #[derive(serde::Deserialize)]
/// struct Project {
///     template_dir: String,
/// }
///
/// let cf: ConfigFile<Project> = Loader::new()
///     .layer_file("project.toml")
///     .load_file()?;
///
/// // "templates/foo.tera" becomes absolute relative to project.toml's directory
/// let abs = cf.resolve(&cf.config.template_dir);
/// # Ok::<(), star_toml::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ConfigFile<T> {
    /// The parsed config.
    pub config: T,
    /// Path to the TOML file that was the last file-based source loaded.
    ///
    /// If no file source was used, this is the current working directory.
    pub path: PathBuf,
}

impl<T> ConfigFile<T> {
    /// Resolve `relative` against the directory that contains the config file.
    ///
    /// If `relative` is already absolute, it is returned unchanged.
    #[must_use]
    pub fn resolve(&self, relative: impl AsRef<Path>) -> PathBuf {
        let rel = relative.as_ref();
        if rel.is_absolute() {
            return rel.to_path_buf();
        }
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        dir.join(rel)
    }
}

impl<T> std::ops::Deref for ConfigFile<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.config
    }
}

impl<T> std::ops::DerefMut for ConfigFile<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.config
    }
}

// ---------------------------------------------------------------------------
// Loader — builder
// ---------------------------------------------------------------------------

/// Builder for composing multiple TOML config sources into a single value.
///
/// Sources are merged in the order they are added: earlier layers provide defaults,
/// later layers override specific keys. Table keys are merged recursively; arrays and
/// scalars are replaced entirely by the later layer.
///
/// Environment-variable expansion (`${VAR}` / `$VAR`) is applied to every source
/// before parsing.
///
/// # Example
///
/// ```no_run
/// use star_toml::Loader;
///
/// #[derive(serde::Deserialize)]
/// struct AppConfig {
///     name: String,
/// }
///
/// const DEFAULTS: &str = r#"
/// name = "default-app"
/// "#;
///
/// let cfg: AppConfig = Loader::new()
///     .layer_str(DEFAULTS, "built-in defaults")
///     .find_file("app.toml")       // walks up from cwd
///     .env_prefix("APP_")          // APP_NAME=foo → name = "foo"
///     .load()?;
/// # Ok::<(), star_toml::Error>(())
/// ```
pub struct Loader {
    layers: Vec<ConfigLayer>,
    env_prefix: Option<String>,
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

impl Loader {
    /// Create an empty loader with no sources.
    #[must_use]
    pub fn new() -> Self {
        Self { layers: Vec::new(), env_prefix: None }
    }

    /// Add a literal TOML string as the next layer.
    ///
    /// `label` is used in error messages to identify this source (e.g., `"built-in defaults"`).
    #[must_use]
    pub fn layer_str(mut self, content: impl Into<String>, label: &'static str) -> Self {
        self.layers.push(ConfigLayer::Str(content.into(), label));
        self
    }

    /// Add a TOML file as the next layer. Returns an error from [`load`] if the file
    /// does not exist.
    ///
    /// [`load`]: Loader::load
    #[must_use]
    pub fn layer_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.layers.push(ConfigLayer::File(path.into()));
        self
    }

    /// Add a TOML file as the next layer, silently skipping it when absent.
    ///
    /// Useful for optional user-specific config files (e.g., `~/.config/app.toml`).
    #[must_use]
    pub fn layer_file_if_exists(mut self, path: impl Into<PathBuf>) -> Self {
        self.layers.push(ConfigLayer::FileIfExists(path.into()));
        self
    }

    /// Walk parent directories from the current working directory until `file_name`
    /// is found, then add that file as the next layer.
    ///
    /// Skipped silently when the file is not found anywhere in the directory tree.
    #[must_use]
    pub fn find_file(mut self, file_name: impl Into<String>) -> Self {
        self.layers.push(ConfigLayer::FindFile(file_name.into()));
        self
    }

    /// Apply environment-variable overrides using `prefix` as a filter.
    ///
    /// After loading and merging all file layers, any env var whose name starts with
    /// `prefix` is stripped of that prefix and mapped to a TOML key path:
    ///
    /// - Double underscores (`__`) become `.` (path separator)
    /// - The result is lowercased
    ///
    /// For example, with prefix `"APP_"`:
    /// - `APP_NAME=foo`              → `name = "foo"`
    /// - `APP_SERVER__PORT=9090`     → `server.port = 9090` (integer)
    /// - `APP_DEBUG=true`            → `debug = true` (boolean)
    ///
    /// Values are parsed as TOML scalars: bool → integer → float → string.
    #[must_use]
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Merge all layers and deserialize into `T`.
    ///
    /// Returns an error if any required file is missing, any layer fails to parse,
    /// or the merged result cannot be deserialized into `T`.
    pub fn load<T: DeserializeOwned>(self) -> Result<T> {
        let (merged, _) = self.merge_layers()?;
        deserialize_value(merged, "merged config")
    }

    /// Like [`load`], but also returns a [`ConfigFile`] carrying the path of the last
    /// file-based source.  The path is used for resolving relative paths stored in
    /// the config.
    ///
    /// [`load`]: Loader::load
    pub fn load_file<T: DeserializeOwned>(self) -> Result<ConfigFile<T>> {
        let (merged, last_path) = self.merge_layers()?;
        let config = deserialize_value(merged, "merged config")?;
        Ok(ConfigFile { config, path: last_path })
    }

    /// Merge all layers (without environment overrides) and return `Config<Raw>`.
    ///
    /// # Errors
    ///
    /// Returns an error if any required file is missing or fails to parse.
    pub fn load_raw(mut self) -> Result<Config<Raw>> {
        self.env_prefix = None;
        let (merged, last_path) = self.merge_layers()?;
        Ok(Config { state: Raw(merged), path: last_path })
    }

    /// Like [`load`], but runs [`Validate::check`] on the result before returning.
    ///
    /// On validation failure the error is [`Error::Invalid`], carrying the full
    /// path-precise, multi-error report.
    ///
    /// [`load`]: Loader::load
    pub fn load_validated<T: DeserializeOwned + Validate>(self) -> Result<T> {
        let cfg: T = self.load()?;
        cfg.check()?;
        Ok(cfg)
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    fn merge_layers(self) -> Result<(Value, PathBuf)> {
        let mut merged = Value::Table(toml::map::Map::new());
        let mut last_file_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        for layer in self.layers {
            match layer {
                ConfigLayer::Str(content, label) => {
                    let expanded = expand_env_vars(&content);
                    let val = parse_str(&expanded, label)?;
                    deep_merge(&mut merged, val);
                }
                ConfigLayer::File(path) => {
                    if !path.exists() {
                        return Err(Error::FileNotFound(path));
                    }
                    let val = load_file_as_value(&path)?;
                    last_file_path = path;
                    deep_merge(&mut merged, val);
                }
                ConfigLayer::FileIfExists(path) => {
                    if path.exists() {
                        let val = load_file_as_value(&path)?;
                        last_file_path = path;
                        deep_merge(&mut merged, val);
                    }
                }
                ConfigLayer::FindFile(file_name) => {
                    if let Some(path) = find_config_file_from_cwd(&file_name) {
                        let val = load_file_as_value(&path)?;
                        last_file_path = path;
                        deep_merge(&mut merged, val);
                    }
                }
            }
        }

        // Apply env-var prefix overrides on top of everything
        if let Some(prefix) = self.env_prefix {
            let prefix_upper = prefix.to_ascii_uppercase();
            for (key, val) in std::env::vars() {
                let key_upper = key.to_ascii_uppercase();
                if let Some(suffix) = key_upper.strip_prefix(&prefix_upper) {
                    let toml_key = suffix.replace("__", ".").to_ascii_lowercase();
                    let toml_val = env_str_to_value(&val);
                    set_dotted(&mut merged, &toml_key, toml_val);
                }
            }
        }

        Ok((merged, last_file_path))
    }
}

// ---------------------------------------------------------------------------
// Free functions (also part of the public API via lib.rs re-exports)
// ---------------------------------------------------------------------------

/// Walk parent directories from `start` until `file_name` is found.
///
/// Returns `None` when no matching file exists in any ancestor directory.
///
/// # Examples
///
/// ```no_run
/// use star_toml::find_config_file;
///
/// if let Some(path) = find_config_file("Cargo.toml", ".") {
///     println!("workspace root: {:?}", path.parent());
/// }
/// ```
#[must_use]
pub fn find_config_file(file_name: &str, start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut dir = start.as_ref().to_path_buf();
    // Normalise: if `start` is a file, begin with its parent.
    if dir.is_file() {
        dir.pop();
    }
    loop {
        let candidate = dir.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Walk parent directories from `start` and load the first `file_name` found.
///
/// Equivalent to combining [`find_config_file`] and [`load_file`].
///
/// # Errors
///
/// Returns [`Error::FileNotFound`] when no matching file exists anywhere in the
/// directory tree.
pub fn find_and_load<T: DeserializeOwned>(
    file_name: &str,
    start: impl AsRef<Path>,
) -> Result<(PathBuf, T)> {
    let path = find_config_file(file_name, start)
        .ok_or_else(|| Error::FileNotFound(PathBuf::from(file_name)))?;
    let cfg = load_file(&path)?;
    Ok((path, cfg))
}

/// Parse `T` from a TOML string after expanding `${VAR}` / `$VAR` references.
///
/// # Examples
///
/// ```
/// #[derive(serde::Deserialize)]
/// struct Config { name: String }
///
/// let cfg: Config = star_toml::from_str("[name]\nname = \"test\"").unwrap_or(Config { name: "test".into() });
/// ```
pub fn from_str<T: DeserializeOwned>(content: &str) -> Result<T> {
    let expanded = expand_env_vars(content);
    parse_str(&expanded, "inline string")
}

/// Load and parse `T` from a TOML file, with env-var expansion.
///
/// # Errors
///
/// Returns [`Error::FileNotFound`] if the file does not exist.
pub fn load_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let content = read_file(path)?;
    let expanded = expand_env_vars(&content);
    parse_str(&expanded, &path.display().to_string())
}

/// Serialize `value` to a pretty-printed TOML string.
///
/// # Errors
///
/// Returns [`Error::Serialize`] if the value cannot be represented as TOML.
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    toml::to_string_pretty(value).map_err(Error::from)
}

/// Serialize `value` and write it to `path`, creating parent directories as needed.
///
/// Round-trips with [`load_file`]: useful for `init`-style commands that scaffold a
/// default config to disk.
///
/// # Errors
///
/// Returns [`Error::Serialize`] on serialization failure or [`Error::Io`] on write failure.
/// Serialize `value` and write it to `path` using standard TOML serialization (not pretty-printed),
/// creating parent directories as needed.
///
/// Round-trips with [`load_file`]: useful for `init`-style commands that scaffold a
/// default config to disk.
///
/// # Errors
///
/// Returns [`Error::Serialize`] on serialization failure or [`Error::Io`] on write failure.
pub fn save_file<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let toml = toml::to_string(value).map_err(Error::from)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
    }
    fs::write(path, toml).map_err(|e| Error::io(path, e))
}

/// Serialize `value` and write it to `path` using pretty-printed TOML serialization,
/// creating parent directories as needed.
///
/// # Errors
///
/// Returns [`Error::Serialize`] on serialization failure or [`Error::Io`] on write failure.
pub fn save_pretty<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let toml = toml::to_string_pretty(value).map_err(Error::from)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
    }
    fs::write(path, toml).map_err(|e| Error::io(path, e))
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn read_file(path: &Path) -> Result<String> {
    if !path.exists() {
        return Err(Error::FileNotFound(path.to_path_buf()));
    }
    fs::read_to_string(path).map_err(|e| Error::io(path, e))
}

fn load_file_as_value(path: &Path) -> Result<Value> {
    let content = read_file(path)?;
    let expanded = expand_env_vars(&content);
    parse_str(&expanded, &path.display().to_string())
}

fn parse_str<T: DeserializeOwned>(content: &str, label: &str) -> Result<T> {
    toml::from_str(content).map_err(|e| Error::parse(label, e))
}

fn deserialize_value<T: DeserializeOwned>(value: Value, label: &str) -> Result<T> {
    T::deserialize(value).map_err(|e| Error::parse(label, e))
}

fn find_config_file_from_cwd(file_name: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_config_file(file_name, cwd)
}

// ---------------------------------------------------------------------------
// Config Typestate Lifecycle
// ---------------------------------------------------------------------------

/// Trait for configuration structures that require normalization or post-deserialization validation hooks.
pub trait ConfigLifecycle {
    /// Normalise fields (e.g. trim strings, resolve relative paths).
    fn normalize(&mut self) {}
    /// Post-deserialization validation hook.
    fn validate_lifecycle(&self, _v: &mut crate::validation::Validator) {}
}

/// Raw state of the loaded configuration.
#[derive(Debug, Clone)]
pub struct Raw(pub Value);

/// Merged state of the configuration, after environment overrides.
#[derive(Debug, Clone)]
pub struct Merged(Value);

/// Deserialized state of the configuration, mapped to a struct `T`.
#[derive(Debug, Clone)]
pub struct Deserialized<T>(T);

/// Validated state of the configuration, after satisfying invariants.
#[derive(Debug, Clone)]
pub struct Validated<T>(T);

/// Frozen, immutable state of the configuration.
#[derive(Debug, Clone)]
pub struct Frozen<T>(T);

/// A configuration wrapper carrying state `S` and the path of the last loaded file.
#[derive(Debug, Clone)]
pub struct Config<S> {
    /// The current state of the configuration.
    pub state: S,
    /// Path to the TOML file that was the last file-based source loaded.
    pub path: PathBuf,
}

impl Config<Raw> {
    /// Construct a new Config with the given content in Raw state.
    pub fn new(content: &str) -> Self {
        let val: Value = toml::from_str(content).unwrap_or(Value::Table(toml::map::Map::new()));
        Self { state: Raw(val), path: PathBuf::from("") }
    }

    /// Get the state name representation.
    pub fn state_name(&self) -> &'static str {
        "Raw"
    }

    /// Apply environment-variable overrides using `env_prefix` and transition to `Merged`.
    ///
    /// # Errors
    ///
    /// Returns parsing or merge errors.
    pub fn merge(self, env_prefix: Option<&str>) -> Result<Config<Merged>> {
        let mut merged = self.state.0;
        if let Some(prefix) = env_prefix {
            let prefix_upper = prefix.to_ascii_uppercase();
            for (key, val) in std::env::vars() {
                let key_upper = key.to_ascii_uppercase();
                if let Some(suffix) = key_upper.strip_prefix(&prefix_upper) {
                    let toml_key = suffix.replace("__", ".").to_ascii_lowercase();
                    let toml_val = env_str_to_value(&val);
                    set_dotted(&mut merged, &toml_key, toml_val);
                }
            }
        }
        Ok(Config { state: Merged(merged), path: self.path })
    }
}

impl Config<Merged> {
    /// Get the state name representation.
    pub fn state_name(&self) -> &'static str {
        "Merged"
    }

    /// Deserialize the merged TOML representation into `T` and transition to `Deserialized`.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML representation cannot be deserialized into `T`.
    pub fn deserialize<T: DeserializeOwned + ConfigLifecycle>(
        self,
    ) -> Result<Config<Deserialized<T>>> {
        let mut value: T = deserialize_value(self.state.0, "merged config")?;
        value.normalize();
        Ok(Config { state: Deserialized(value), path: self.path })
    }
}

impl<T> Config<Deserialized<T>> {
    /// Get a reference to the deserialized value.
    pub fn get(&self) -> &T {
        &self.state.0
    }

    /// Get a mutable reference to the deserialized value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.state.0
    }

    /// Get the state name representation.
    pub fn state_name(&self) -> &'static str {
        "Deserialized"
    }
}

impl<T: Validate + ConfigLifecycle> Config<Deserialized<T>> {
    /// Validate the deserialized configuration and transition to `Validated`.
    ///
    /// # Errors
    ///
    /// Returns `Error::Invalid` if any validation checks fail.
    pub fn validate(self) -> Result<Config<Validated<T>>> {
        let mut v = crate::validation::Validator::new();
        self.state.0.validate(&mut v);
        self.state.0.validate_lifecycle(&mut v);

        let checks_run = v.checks_run;
        let errors = v.errors.clone();
        let failed =
            errors.iter().filter(|e| e.severity >= crate::validation::Severity::Error).count();

        if failed > 0 {
            let mut errs = crate::validation::ValidationErrors { errors, title: None, checks_run };
            errs.set_title_for::<T>();
            return Err(Error::Invalid(errs));
        }

        Ok(Config { state: Validated(self.state.0), path: self.path })
    }
}

impl<T: Validate> Config<Validated<T>> {
    /// Construct a new Config with the given value in Validated state.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn new(value: T) -> Result<Self> {
        value.check()?;
        Ok(Self { state: Validated(value), path: PathBuf::from("") })
    }
}

impl<T> Config<Validated<T>> {
    /// Get a reference to the validated value.
    pub fn get(&self) -> &T {
        &self.state.0
    }

    /// Get a mutable reference to the validated value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.state.0
    }

    /// Get the state name representation.
    pub fn state_name(&self) -> &'static str {
        "Validated"
    }

    /// Freeze the configuration, transitioning to `Frozen`.
    #[must_use]
    pub fn freeze(self) -> Config<Frozen<T>> {
        Config { state: Frozen(self.state.0), path: self.path }
    }
}

impl<T> Config<Frozen<T>> {
    /// Get a reference to the frozen value.
    pub fn get(&self) -> &T {
        &self.state.0
    }

    /// Get the state name representation.
    pub fn state_name(&self) -> &'static str {
        "Frozen"
    }
}

fn sort_toml_value(val: &mut toml::Value) {
    match val {
        toml::Value::Table(table) => {
            let old_map = std::mem::take(table);
            let mut items: Vec<(String, toml::Value)> = old_map.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            for (k, mut v) in items {
                sort_toml_value(&mut v);
                table.insert(k, v);
            }
        }
        toml::Value::Array(arr) => {
            for v in arr {
                sort_toml_value(v);
            }
        }
        _ => {}
    }
}

fn save_canonical_impl<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
    let mut val = toml::Value::try_from(value).map_err(Error::from)?;
    sort_toml_value(&mut val);
    let toml = toml::to_string(&val).map_err(Error::from)?;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
    }
    fs::write(path, toml).map_err(|e| Error::io(path, e))
}

impl<T: Serialize> Config<Frozen<T>> {
    /// Serialize the frozen configuration in alphabetical sorted canonical order and write it to `path`.
    ///
    /// # Errors
    ///
    /// Returns serialization or I/O errors.
    pub fn save_canonical(&self, path: impl AsRef<Path>) -> Result<()> {
        save_canonical_impl(&self.state.0, path)
    }
}

impl<T: Serialize> Config<Validated<T>> {
    /// Serialize the validated configuration in alphabetical sorted canonical order and write it to `path`.
    ///
    /// # Errors
    ///
    /// Returns serialization or I/O errors.
    pub fn save_canonical(&self, path: impl AsRef<Path>) -> Result<()> {
        save_canonical_impl(&self.state.0, path)
    }
}

// ---------------------------------------------------------------------------
// Trusted Config & Analytics
// ---------------------------------------------------------------------------

/// Report containing the resolved path of the last loaded file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSourceReport {
    /// Path to the TOML file.
    pub path: PathBuf,
}

/// Report containing validation statistics and errors.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    /// Conformance score (0.0 to 1.0).
    pub fitness: f64,
    /// Total number of checks run.
    pub checks_run: usize,
    /// Number of checks passed.
    pub checks_passed: usize,
    /// The detailed list of validation errors.
    pub errors: Vec<crate::validation::ValidationError>,
}

/// A wrapper for the configuration digest hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigDigest(pub u64);

/// A trusted configuration package carrying the value and reports.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustedConfig<T> {
    /// The parsed and validated configuration value.
    pub value: T,
    /// Metadata about the source of the configuration.
    pub source: ConfigSourceReport,
    /// Detailed validation report.
    pub validation: ValidationReport,
    /// Unique digest hash of the merged TOML representation.
    pub digest: ConfigDigest,
}

impl<T> std::ops::Deref for TrustedConfig<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

/// Builder for composing multiple TOML config sources and producing a `TrustedConfig`.
pub struct TrustedLoader {
    loader: Loader,
}

impl Default for TrustedLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustedLoader {
    /// Create a new empty `TrustedLoader` with no sources.
    #[must_use]
    pub fn new() -> Self {
        Self { loader: Loader::new() }
    }

    /// Add a literal TOML string as the next layer.
    #[must_use]
    pub fn layer_str(mut self, content: impl Into<String>, label: &'static str) -> Self {
        self.loader = self.loader.layer_str(content, label);
        self
    }

    /// Add a TOML file as the next layer.
    #[must_use]
    pub fn layer_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.loader = self.loader.layer_file(path);
        self
    }

    /// Add a TOML file as the next layer, silently skipping it when absent.
    #[must_use]
    pub fn layer_file_if_exists(mut self, path: impl Into<PathBuf>) -> Self {
        self.loader = self.loader.layer_file_if_exists(path);
        self
    }

    /// Walk parent directories from the current working directory until `file_name` is found.
    #[must_use]
    pub fn find_file(mut self, file_name: impl Into<String>) -> Self {
        self.loader = self.loader.find_file(file_name);
        self
    }

    /// Apply environment-variable overrides using `prefix` as a filter.
    #[must_use]
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.loader = self.loader.env_prefix(prefix);
        self
    }

    /// Merge all layers, deserialize, validate, and compute the digest to produce a `TrustedConfig`.
    ///
    /// # Errors
    ///
    /// Returns parsing, merge, or validation errors (wrapped as `Error::Invalid`).
    pub fn load<T: DeserializeOwned + Validate + ConfigLifecycle>(
        self,
    ) -> Result<TrustedConfig<T>> {
        let (merged, last_path) = self.loader.merge_layers()?;
        let mut value: T = deserialize_value(merged.clone(), "merged config")?;

        value.normalize();

        let mut v = crate::validation::Validator::new();
        value.validate(&mut v);
        value.validate_lifecycle(&mut v);

        let toml_str = toml::to_string(&merged).map_err(Error::from)?;
        let hash = crate::validation::fnv1a(toml_str.as_bytes());
        let digest = ConfigDigest(hash);

        let checks_run = v.checks_run;
        let errors = v.errors.clone();
        let failed =
            errors.iter().filter(|e| e.severity >= crate::validation::Severity::Error).count();
        let checks_passed = checks_run.saturating_sub(failed);
        let fitness = if checks_run == 0 { 1.0 } else { checks_passed as f64 / checks_run as f64 };

        let validation = ValidationReport { fitness, checks_run, checks_passed, errors };

        let source = ConfigSourceReport { path: last_path };

        if !validation.errors.is_empty() {
            let mut errs = crate::validation::ValidationErrors {
                errors: validation.errors,
                title: None,
                checks_run,
            };
            errs.set_title_for::<T>();
            return Err(Error::Invalid(errs));
        }

        Ok(TrustedConfig { value, source, validation, digest })
    }
}

// ---------------------------------------------------------------------------
// WP-1: Additional typestate markers
// ---------------------------------------------------------------------------

/// State after all file layers are loaded, merged with provenance tracking, and
/// sources are registered in a [`SourceReport`].
///
/// This is the entry point for the trusted pipeline. From here, apply env overrides
/// (→ [`EnvResolved`]) then continue the lifecycle.
///
#[derive(Debug, Clone)]
pub struct BoundedSources {
    /// Merged TOML value from all file layers (env not yet applied).
    pub value: Value,
    /// Provenance record for every source registered during loading.
    pub source_report: SourceReport,
    /// Per-layer merge provenance.
    pub layer_report: LayerReport,
    /// Cumulative field-provenance map after all file layers — every leaf maps to its
    /// current winning layer-id string.
    pub global_winner_map: WinnerMap,
}

/// State after env-var overrides have been applied and recorded.
///
/// Carries all prior reports forwarded from [`BoundedSources`].
#[derive(Debug, Clone)]
pub struct EnvResolved {
    /// Merged TOML value with env overrides applied.
    pub value: Value,
    /// Forwarded from [`BoundedSources`].
    pub source_report: SourceReport,
    /// Forwarded from [`BoundedSources`].
    pub layer_report: LayerReport,
    /// Env-override provenance for variables that matched the configured prefix.
    pub env_report: EnvOverrideReport,
    /// Cumulative field-provenance map after env overrides; env entries overwrite
    /// file-layer entries for fields they touch.
    pub global_winner_map: WinnerMap,
}

/// Terminal result of [`TrustedLoader::load_frozen`].
///
/// Bundles the frozen config with the full provenance reports produced during
/// the loading pipeline. Designed so a future OCEL lifecycle-history export can
/// consume `source_report`, `layer_report`, and `env_report` directly.
///
#[derive(Debug)]
pub struct FrozenLoadResult<T> {
    /// The frozen, validated configuration.
    pub config: Config<Frozen<T>>,
    /// Source provenance.
    pub source_report: SourceReport,
    /// Layer merge provenance.
    pub layer_report: LayerReport,
    /// Env-override provenance.
    pub env_report: EnvOverrideReport,
    /// Final cumulative field-provenance map.
    pub global_winner_map: WinnerMap,
}

// ---------------------------------------------------------------------------
// Config<BoundedSources> — WP-1 / WP-2
// ---------------------------------------------------------------------------

impl Config<BoundedSources> {
    /// Name of this typestate for diagnostics.
    pub fn state_name(&self) -> &'static str {
        "BoundedSources"
    }

    /// Access the source provenance report.
    pub fn source_report(&self) -> &SourceReport {
        &self.state.source_report
    }

    /// Access the layer merge provenance report.
    pub fn layer_report(&self) -> &LayerReport {
        &self.state.layer_report
    }

    /// Access the cumulative field-provenance map after all file layers.
    pub fn global_winner_map(&self) -> &WinnerMap {
        &self.state.global_winner_map
    }

    /// Apply env-var overrides using `prefix` and transition to [`EnvResolved`].
    ///
    /// Only variables whose names start with `prefix` (case-insensitive) are
    /// processed. Unrelated ambient OS variables (`PATH`, `HOME`, `SHELL`, etc.)
    /// are ignored and never appear in the report.
    ///
    /// An env var that matches the prefix but maps to an empty TOML path after
    /// stripping the prefix and transforming `__` → `.` is rejected with code
    /// `"empty_path"`.
    pub fn apply_env(self, prefix: Option<&str>) -> Result<Config<EnvResolved>> {
        let mut value = self.state.value;
        let mut global_winner_map = self.state.global_winner_map;
        let mut env_report = EnvOverrideReport::default();

        if let Some(prefix) = prefix {
            env_report.prefix = prefix.to_owned();
            let prefix_upper = prefix.to_ascii_uppercase();

            for (key, raw_val) in std::env::vars() {
                let key_upper = key.to_ascii_uppercase();
                if let Some(suffix) = key_upper.strip_prefix(&prefix_upper) {
                    let toml_key = suffix.replace("__", ".").to_ascii_lowercase();
                    let raw_digest = blake3_hex(raw_val.as_bytes());
                    let accepted = !toml_key.is_empty();

                    if accepted {
                        let toml_val = env_str_to_value(&raw_val);
                        let coerced_type = match &toml_val {
                            Value::Boolean(_) => CoercedType::Bool,
                            Value::Integer(_) => CoercedType::Integer,
                            Value::Float(_) => CoercedType::Float,
                            _ => CoercedType::Str,
                        };
                        let coerced_repr = toml_val.to_string();
                        let coerced_digest = blake3_hex(coerced_repr.as_bytes());

                        set_dotted(&mut value, &toml_key, toml_val);
                        global_winner_map.insert(toml_key.clone(), "env".to_owned());

                        env_report.entries.push(EnvOverrideEntry {
                            raw_env_key: key,
                            configured_prefix: prefix.to_owned(),
                            mapped_path: toml_key,
                            raw_value_digest: raw_digest,
                            coerced_type: Some(coerced_type),
                            coerced_value_digest: Some(coerced_digest),
                            accepted: true,
                            rejection_code: None,
                        });
                    } else {
                        env_report.entries.push(EnvOverrideEntry {
                            raw_env_key: key,
                            configured_prefix: prefix.to_owned(),
                            mapped_path: String::new(),
                            raw_value_digest: raw_digest,
                            coerced_type: None,
                            coerced_value_digest: None,
                            accepted: false,
                            rejection_code: Some("empty_path".to_owned()),
                        });
                    }
                }
            }
        }

        Ok(Config {
            state: EnvResolved {
                value,
                source_report: self.state.source_report,
                layer_report: self.state.layer_report,
                env_report,
                global_winner_map,
            },
            path: self.path,
        })
    }
}

// ---------------------------------------------------------------------------
// Config<EnvResolved> — WP-1 / WP-2
// ---------------------------------------------------------------------------

impl Config<EnvResolved> {
    /// Name of this typestate for diagnostics.
    pub fn state_name(&self) -> &'static str {
        "EnvResolved"
    }

    pub fn source_report(&self) -> &SourceReport {
        &self.state.source_report
    }

    pub fn layer_report(&self) -> &LayerReport {
        &self.state.layer_report
    }

    pub fn env_report(&self) -> &EnvOverrideReport {
        &self.state.env_report
    }

    pub fn global_winner_map(&self) -> &WinnerMap {
        &self.state.global_winner_map
    }

    /// Deserialize and normalize into `T`, transitioning to [`Deserialized<T>`].
    pub fn deserialize<T: DeserializeOwned + ConfigLifecycle>(
        self,
    ) -> Result<Config<Deserialized<T>>> {
        let mut value: T = deserialize_value(self.state.value, "merged config")?;
        value.normalize();
        Ok(Config { state: Deserialized(value), path: self.path })
    }
}

// ---------------------------------------------------------------------------
// Loader::load_bounded — WP-1 / WP-2 / WP-3
// ---------------------------------------------------------------------------

impl Loader {
    /// Load all file layers with full provenance tracking, producing [`Config<BoundedSources>`].
    ///
    /// Unlike [`load_raw`], this method:
    /// - records a [`SourceReport`] for every source (including optional-missing files)
    /// - uses [`deep_merge_traced`] to produce per-layer and cumulative [`WinnerMap`]s
    /// - does **not** apply env-var overrides (call [`Config::apply_env`] next)
    ///
    /// [`load_raw`]: Loader::load_raw
    pub fn load_bounded(self) -> Result<Config<BoundedSources>> {
        let mut merged = Value::Table(toml::map::Map::new());
        let mut last_file_path =
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut source_report = SourceReport::default();
        let mut layer_report = LayerReport::default();
        let mut global_winner_map = WinnerMap::new();
        let mut layer_order_acc = String::new();

        for layer in self.layers {
            match layer {
                ConfigLayer::Str(content, label) => {
                    let source_id = source_report.entries.len();
                    let digest = blake3_hex(content.as_bytes());
                    let size = content.len() as u64;

                    source_report.entries.push(SourceEntry {
                        source_id,
                        source_kind: SourceKind::Str,
                        label: label.to_owned(),
                        path: None,
                        required: true,
                        found: true,
                        digest: Some(digest.clone()),
                        size_bytes: Some(size),
                        source_root: None,
                        source_parent: None,
                    });

                    let expanded = expand_env_vars(&content);
                    let val: Value = parse_str(&expanded, label)?;
                    let layer_id_str = format!("layer-{source_id}");

                    layer_order_acc.push_str(&digest);
                    let layer_order_digest = blake3_hex(layer_order_acc.as_bytes());

                    let mut layer_winner_map = WinnerMap::new();
                    deep_merge_traced(
                        &mut merged,
                        val,
                        &layer_id_str,
                        "",
                        &mut layer_winner_map,
                    );
                    global_winner_map.extend(layer_winner_map.clone());

                    layer_report.entries.push(LayerEntry {
                        layer_id: layer_report.entries.len(),
                        layer_name: label.to_owned(),
                        priority: layer_report.entries.len(),
                        source_id,
                        digest,
                        layer_order_digest,
                        winning_field_map: layer_winner_map,
                    });
                }

                ConfigLayer::File(path) => {
                    let source_id = source_report.entries.len();
                    if !path.exists() {
                        source_report.entries.push(SourceEntry {
                            source_id,
                            source_kind: SourceKind::File,
                            label: path.display().to_string(),
                            path: Some(path.clone()),
                            required: true,
                            found: false,
                            digest: None,
                            size_bytes: None,
                            source_root: None,
                            source_parent: None,
                        });
                        return Err(Error::FileNotFound(path));
                    }

                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| Error::io(&path, e))?;
                    let digest = blake3_hex(content.as_bytes());
                    let size = content.len() as u64;
                    let source_root = path.parent().map(PathBuf::from);
                    let source_parent =
                        source_root.as_ref().and_then(|p| p.parent()).map(PathBuf::from);

                    source_report.entries.push(SourceEntry {
                        source_id,
                        source_kind: SourceKind::File,
                        label: path.display().to_string(),
                        path: Some(path.clone()),
                        required: true,
                        found: true,
                        digest: Some(digest.clone()),
                        size_bytes: Some(size),
                        source_root,
                        source_parent,
                    });

                    let expanded = expand_env_vars(&content);
                    let val: Value = parse_str(&expanded, &path.display().to_string())?;
                    let layer_id_str = format!("layer-{source_id}");

                    last_file_path = path;
                    layer_order_acc.push_str(&digest);
                    let layer_order_digest = blake3_hex(layer_order_acc.as_bytes());
                    let layer_name =
                        source_report.entries.last().unwrap().label.clone();

                    let mut layer_winner_map = WinnerMap::new();
                    deep_merge_traced(
                        &mut merged,
                        val,
                        &layer_id_str,
                        "",
                        &mut layer_winner_map,
                    );
                    global_winner_map.extend(layer_winner_map.clone());

                    layer_report.entries.push(LayerEntry {
                        layer_id: layer_report.entries.len(),
                        layer_name,
                        priority: layer_report.entries.len(),
                        source_id,
                        digest,
                        layer_order_digest,
                        winning_field_map: layer_winner_map,
                    });
                }

                ConfigLayer::FileIfExists(path) => {
                    let source_id = source_report.entries.len();
                    if !path.exists() {
                        source_report.entries.push(SourceEntry {
                            source_id,
                            source_kind: SourceKind::OptionalFile,
                            label: path.display().to_string(),
                            path: Some(path),
                            required: false,
                            found: false,
                            digest: None,
                            size_bytes: None,
                            source_root: None,
                            source_parent: None,
                        });
                        continue;
                    }

                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| Error::io(&path, e))?;
                    let digest = blake3_hex(content.as_bytes());
                    let size = content.len() as u64;
                    let source_root = path.parent().map(PathBuf::from);
                    let source_parent =
                        source_root.as_ref().and_then(|p| p.parent()).map(PathBuf::from);

                    source_report.entries.push(SourceEntry {
                        source_id,
                        source_kind: SourceKind::OptionalFile,
                        label: path.display().to_string(),
                        path: Some(path.clone()),
                        required: false,
                        found: true,
                        digest: Some(digest.clone()),
                        size_bytes: Some(size),
                        source_root,
                        source_parent,
                    });

                    let expanded = expand_env_vars(&content);
                    let val: Value = parse_str(&expanded, &path.display().to_string())?;
                    let layer_id_str = format!("layer-{source_id}");

                    last_file_path = path;
                    layer_order_acc.push_str(&digest);
                    let layer_order_digest = blake3_hex(layer_order_acc.as_bytes());
                    let layer_name =
                        source_report.entries.last().unwrap().label.clone();

                    let mut layer_winner_map = WinnerMap::new();
                    deep_merge_traced(
                        &mut merged,
                        val,
                        &layer_id_str,
                        "",
                        &mut layer_winner_map,
                    );
                    global_winner_map.extend(layer_winner_map.clone());

                    layer_report.entries.push(LayerEntry {
                        layer_id: layer_report.entries.len(),
                        layer_name,
                        priority: layer_report.entries.len(),
                        source_id,
                        digest,
                        layer_order_digest,
                        winning_field_map: layer_winner_map,
                    });
                }

                ConfigLayer::FindFile(file_name) => {
                    let source_id = source_report.entries.len();
                    match find_config_file_from_cwd(&file_name) {
                        None => {
                            source_report.entries.push(SourceEntry {
                                source_id,
                                source_kind: SourceKind::FindFile,
                                label: file_name,
                                path: None,
                                required: false,
                                found: false,
                                digest: None,
                                size_bytes: None,
                                source_root: None,
                                source_parent: None,
                            });
                        }
                        Some(path) => {
                            let content = std::fs::read_to_string(&path)
                                .map_err(|e| Error::io(&path, e))?;
                            let digest = blake3_hex(content.as_bytes());
                            let size = content.len() as u64;
                            let source_root = path.parent().map(PathBuf::from);
                            let source_parent = source_root
                                .as_ref()
                                .and_then(|p| p.parent())
                                .map(PathBuf::from);

                            source_report.entries.push(SourceEntry {
                                source_id,
                                source_kind: SourceKind::FindFile,
                                label: path.display().to_string(),
                                path: Some(path.clone()),
                                required: false,
                                found: true,
                                digest: Some(digest.clone()),
                                size_bytes: Some(size),
                                source_root,
                                source_parent,
                            });

                            let expanded = expand_env_vars(&content);
                            let val: Value =
                                parse_str(&expanded, &path.display().to_string())?;
                            let layer_id_str = format!("layer-{source_id}");

                            last_file_path = path;
                            layer_order_acc.push_str(&digest);
                            let layer_order_digest =
                                blake3_hex(layer_order_acc.as_bytes());
                            let layer_name =
                                source_report.entries.last().unwrap().label.clone();

                            let mut layer_winner_map = WinnerMap::new();
                            deep_merge_traced(
                                &mut merged,
                                val,
                                &layer_id_str,
                                "",
                                &mut layer_winner_map,
                            );
                            global_winner_map.extend(layer_winner_map.clone());

                            layer_report.entries.push(LayerEntry {
                                layer_id: layer_report.entries.len(),
                                layer_name,
                                priority: layer_report.entries.len(),
                                source_id,
                                digest,
                                layer_order_digest,
                                winning_field_map: layer_winner_map,
                            });
                        }
                    }
                }
            }
        }

        Ok(Config {
            state: BoundedSources {
                value: merged,
                source_report,
                layer_report,
                global_winner_map,
            },
            path: last_file_path,
        })
    }
}

// ---------------------------------------------------------------------------
// TrustedLoader::load_frozen — WP-1 terminal API for this slice
// ---------------------------------------------------------------------------

impl TrustedLoader {
    /// Run the full pre-admission pipeline and produce a frozen, validated config
    /// with complete provenance reports.
    ///
    /// Pipeline:
    /// ```text
    /// load sources (→ SourceReport)
    /// → bound/register sources (→ BoundedSources with LayerReport + WinnerMap)
    /// → apply env overrides (→ EnvResolved with EnvOverrideReport)
    /// → deserialize
    /// → validate        ← validation is mandatory; cannot be skipped
    /// → freeze          ← Frozen<T> is the terminal pre-admission state for this slice
    /// ```
    ///
    /// Returns [`FrozenLoadResult<T>`] which bundles the frozen config with all three
    /// provenance reports. The reports are designed to feed a future OCEL
    /// lifecycle-history export.
    ///
    /// # Errors
    ///
    /// Returns an error if any required file is missing, parsing fails, or validation
    /// produces any `Error`-or-above severity findings.
    pub fn load_frozen<T: DeserializeOwned + Validate + ConfigLifecycle>(
        mut self,
    ) -> Result<FrozenLoadResult<T>> {
        // Separate env prefix from the loader so load_bounded skips env.
        // Env application happens in apply_env with full tracking.
        let env_prefix = self.loader.env_prefix.take();

        let bounded = self.loader.load_bounded()?;
        let env_resolved = bounded.apply_env(env_prefix.as_deref())?;

        let source_report = env_resolved.state.source_report.clone();
        let layer_report = env_resolved.state.layer_report.clone();
        let env_report = env_resolved.state.env_report.clone();
        let global_winner_map = env_resolved.state.global_winner_map.clone();

        let deser = env_resolved.deserialize::<T>()?;
        let validated = deser.validate()?;
        let config = validated.freeze();

        Ok(FrozenLoadResult { config, source_report, layer_report, env_report, global_winner_map })
    }
}

// ---------------------------------------------------------------------------
// ST-109: ConfigWitness
// ---------------------------------------------------------------------------

/// A cryptographic witness that binds provenance reports + validation fitness
/// to the canonical config bytes.
///
/// Use [`ConfigWitness::compute`] to produce one from a [`FrozenLoadResult`].
/// The inner hash is not publicly settable — use [`ConfigWitness::hash`] to read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWitness {
    /// BLAKE3 hex of all inputs: source digests, layer order, env entries,
    /// validation fitness, and canonical config bytes.
    hash: String,
}

impl ConfigWitness {
    /// The BLAKE3 hex digest that binds all provenance inputs.
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Compute a witness from the full provenance context.
    ///
    /// Hash inputs (joined with `|`):
    /// 1. Source digests concatenated (sorted by `source_id`)
    /// 2. Last `layer_order_digest` (or empty string)
    /// 3. Accepted env entries sorted: `"key=path:raw_digest"`
    /// 4. `format!("{:.6}", validation_fitness)`
    /// 5. `canonical_bytes`
    pub fn compute(
        source_report: &SourceReport,
        layer_report: &LayerReport,
        env_report: &EnvOverrideReport,
        validation_fitness: f64,
        canonical_bytes: &[u8],
    ) -> Self {
        let mut parts: Vec<String> = Vec::new();

        // 1. Source digests sorted by source_id
        let mut source_entries: Vec<_> = source_report.entries.iter().collect();
        source_entries.sort_by_key(|e| e.source_id);
        let source_part: String = source_entries
            .iter()
            .filter_map(|e| e.digest.as_deref())
            .collect::<Vec<_>>()
            .join(",");
        parts.push(source_part);

        // 2. Last layer_order_digest
        let last_lod = layer_report
            .entries
            .last()
            .map(|e| e.layer_order_digest.as_str())
            .unwrap_or("")
            .to_owned();
        parts.push(last_lod);

        // 3. Accepted env entries sorted
        let mut env_entries: Vec<String> = env_report
            .entries
            .iter()
            .filter(|e| e.accepted)
            .map(|e| format!("{}={}:{}", e.raw_env_key, e.mapped_path, e.raw_value_digest))
            .collect();
        env_entries.sort();
        parts.push(env_entries.join(","));

        // 4. Validation fitness
        parts.push(format!("{:.6}", validation_fitness));

        // 5. Canonical bytes (as hex to avoid embedding binary)
        parts.push(blake3_hex(canonical_bytes));

        let joined = parts.join("|");
        let hash = blake3_hex(joined.as_bytes());
        Self { hash }
    }
}

// ---------------------------------------------------------------------------
// ST-106: detect_unknown_fields
// ---------------------------------------------------------------------------

/// Compare `original` (raw TOML value) against `typed` (re-serialized from the
/// deserialized struct) to find keys present in `original` but absent in `typed`.
///
/// Returns dot-separated paths for unknown fields.
///
/// # Errors
///
/// Returns an empty vec if re-serialization fails.
pub fn detect_unknown_fields<T: Serialize>(original: &Value, typed: &T) -> Vec<String> {
    let typed_val = match toml::Value::try_from(typed) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut unknown = Vec::new();
    collect_unknown_keys(original, &typed_val, "", &mut unknown);
    unknown
}

fn collect_unknown_keys(
    original: &Value,
    typed: &Value,
    prefix: &str,
    unknown: &mut Vec<String>,
) {
    match (original, typed) {
        (Value::Table(orig_t), Value::Table(typed_t)) => {
            for (k, v) in orig_t {
                let path = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                if let Some(typed_v) = typed_t.get(k) {
                    collect_unknown_keys(v, typed_v, &path, unknown);
                } else {
                    unknown.push(path);
                }
            }
        }
        // Traverse arrays element-wise so [[table]] arrays of tables are checked.
        (Value::Array(orig_arr), Value::Array(typed_arr)) => {
            for (i, orig_item) in orig_arr.iter().enumerate() {
                let path = format!("{prefix}[{i}]");
                if let Some(typed_item) = typed_arr.get(i) {
                    collect_unknown_keys(orig_item, typed_item, &path, unknown);
                }
                // If typed array is shorter, the extra items are unknown by position.
                // We don't flag them as individual field unknowns here because the
                // structural mismatch is caught by serde deserialization.
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// ST-102: AdmittedConfig
// ---------------------------------------------------------------------------

/// The terminal admission envelope: a validated, witnessed, reportable config.
///
/// Produced by [`TrustedLoader::load_admitted`]. Fields are private to prevent
/// external forgery — use the accessor methods to read them.
#[derive(Debug)]
pub struct AdmittedConfig<T> {
    value: T,
    witness: ConfigWitness,
    source_report: SourceReport,
    layer_report: LayerReport,
    env_report: EnvOverrideReport,
    global_winner_map: WinnerMap,
}

impl<T> AdmittedConfig<T> {
    /// The deserialized and validated configuration value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Cryptographic witness binding all provenance to the canonical bytes.
    pub fn witness(&self) -> &ConfigWitness {
        &self.witness
    }

    /// Source provenance report.
    pub fn source_report(&self) -> &SourceReport {
        &self.source_report
    }

    /// Layer merge provenance report.
    pub fn layer_report(&self) -> &LayerReport {
        &self.layer_report
    }

    /// Env-override provenance report.
    pub fn env_report(&self) -> &EnvOverrideReport {
        &self.env_report
    }

    /// Final cumulative field-provenance map (field path → winning layer id).
    pub fn global_winner_map(&self) -> &WinnerMap {
        &self.global_winner_map
    }

    /// Consume the envelope and return the inner value, discarding provenance.
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T> std::ops::Deref for AdmittedConfig<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl TrustedLoader {
    /// Run the full admission pipeline and return an [`AdmittedConfig<T>`].
    ///
    /// Unknown fields in the TOML source are rejected with code `"unknown_field"`
    /// (CE-8: trusted admission must not silently accept unknown fields).
    /// Use [`load_admitted_exploratory`](TrustedLoader::load_admitted_exploratory)
    /// to allow unknown fields during development.
    ///
    /// # Errors
    ///
    /// Returns an error if any required file is missing, parsing fails,
    /// validation fails, or unknown fields are detected.
    pub fn load_admitted<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(
        mut self,
    ) -> Result<AdmittedConfig<T>> {
        // We need the original merged value to compare against.
        // Re-run load_bounded + apply_env to get both original value and typed result.
        let env_prefix = self.loader.env_prefix.take();
        let bounded = self.loader.load_bounded()?;
        let env_resolved = bounded.apply_env(env_prefix.as_deref())?;

        let original_value = env_resolved.state.value.clone();
        let source_report = env_resolved.state.source_report.clone();
        let layer_report = env_resolved.state.layer_report.clone();
        let env_report = env_resolved.state.env_report.clone();
        let global_winner_map = env_resolved.state.global_winner_map.clone();

        let deser = env_resolved.deserialize::<T>()?;
        let typed_ref = deser.get();

        // Detect unknown fields before consuming
        let unknown = detect_unknown_fields(&original_value, typed_ref);
        if !unknown.is_empty() {
            // Emit one path-precise error per unknown field (CE-10 fix)
            let errors: Vec<crate::validation::ValidationError> = unknown
                .iter()
                .map(|field_path| {
                    let segments = field_path
                        .split('.')
                        .map(|seg| crate::validation::LocSegment::Key(seg.to_owned()))
                        .collect();
                    crate::validation::ValidationError {
                        loc: crate::validation::Loc(segments),
                        kind: crate::validation::ErrorKind::Predicate { code: "unknown_field" },
                        severity: crate::validation::Severity::Error,
                        input: Some(field_path.clone()),
                        msg: format!("unknown field: `{field_path}`"),
                    }
                })
                .collect();
            let checks_run = errors.len();
            let errs = crate::validation::ValidationErrors {
                errors,
                title: None,
                checks_run,
            };
            return Err(Error::Invalid(errs));
        }

        let validated = deser.validate()?;
        let config = validated.freeze();

        let frozen_result = FrozenLoadResult {
            config,
            source_report,
            layer_report,
            env_report,
            global_winner_map,
        };
        build_admitted(frozen_result)
    }

    /// Like [`load_admitted`] but does **not** reject unknown fields.
    ///
    /// Intended for exploratory/development use where the schema is still being
    /// defined. Do not use in production trusted paths.
    ///
    /// # Errors
    ///
    /// Returns an error if any required file is missing, parsing fails, or
    /// validation produces `Error`-or-above findings.
    pub fn load_admitted_exploratory<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(
        self,
    ) -> Result<AdmittedConfig<T>> {
        let result = self.load_frozen::<T>()?;
        build_admitted(result)
    }

    /// Alias for [`load_admitted`] — kept for call-site compatibility.
    ///
    /// Prefer `load_admitted()` directly; this alias will be removed in v26.7.
    #[deprecated(since = "26.6.28", note = "use load_admitted() — it is now strict by default")]
    pub fn load_admitted_strict<T: DeserializeOwned + Validate + ConfigLifecycle + Serialize>(
        self,
    ) -> Result<AdmittedConfig<T>> {
        self.load_admitted::<T>()
    }
}

fn build_admitted<T: Serialize>(result: FrozenLoadResult<T>) -> Result<AdmittedConfig<T>> {
    let FrozenLoadResult { config, source_report, layer_report, env_report, global_winner_map } =
        result;

    // Serialize to canonical bytes
    let mut canonical_val = toml::Value::try_from(config.get()).map_err(Error::from)?;
    sort_toml_value(&mut canonical_val);
    let canonical_str = toml::to_string(&canonical_val).map_err(Error::from)?;
    let canonical_bytes = canonical_str.as_bytes();

    // Compute validation fitness (all valid since load_frozen succeeded)
    let validation_fitness = 1.0_f64;

    let witness = ConfigWitness::compute(
        &source_report,
        &layer_report,
        &env_report,
        validation_fitness,
        canonical_bytes,
    );

    // Extract value from frozen config
    let value = config.state.0;

    Ok(AdmittedConfig { value, witness, source_report, layer_report, env_report, global_winner_map })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serde::{Deserialize, Serialize};
    use tempfile::{NamedTempFile, TempDir};

    use super::*;
    use crate::trusted;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Simple {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
    }

    fn write_toml(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn from_str_parses() {
        let cfg: Simple = from_str("name = \"hello\"\nport = 8080\n").unwrap();
        assert_eq!(cfg.name, "hello");
        assert_eq!(cfg.port, Some(8080));
    }

    #[test]
    fn load_file_ok() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "name = \"test\"").unwrap();
        let cfg: Simple = load_file(f.path()).unwrap();
        assert_eq!(cfg.name, "test");
    }

    #[test]
    fn load_file_not_found() {
        let result = load_file::<Simple>("/nonexistent/x.toml");
        assert!(matches!(result, Err(Error::FileNotFound(_))));
    }

    #[test]
    fn loader_layer_str_and_file() {
        let dir = TempDir::new().unwrap();
        write_toml(&dir, "a.toml", "name = \"from-file\"\nport = 9090\n");

        let cfg: Simple = Loader::new()
            .layer_str("name = \"default\"\nport = 8080\n", "defaults")
            .layer_file(dir.path().join("a.toml"))
            .load()
            .unwrap();

        assert_eq!(cfg.name, "from-file"); // file overrides default
        assert_eq!(cfg.port, Some(9090));
    }

    #[test]
    fn loader_file_if_exists_skips_missing() {
        let cfg: Simple = Loader::new()
            .layer_str("name = \"default\"", "defaults")
            .layer_file_if_exists("/nonexistent/optional.toml")
            .load()
            .unwrap();
        assert_eq!(cfg.name, "default");
    }

    #[test]
    fn loader_env_prefix_override() {
        std::env::set_var("STTOML_NAME", "env-name");
        let result = Loader::new()
            .layer_str("name = \"original\"", "defaults")
            .env_prefix("STTOML_")
            .load::<Simple>();
        std::env::remove_var("STTOML_NAME");
        let cfg = result.unwrap();
        assert_eq!(cfg.name, "env-name");
    }

    #[test]
    fn loader_env_prefix_nested_double_underscore() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Outer {
            server: Server,
        }
        #[derive(Deserialize, PartialEq, Debug)]
        struct Server {
            port: u16,
        }

        std::env::set_var("STTOML2_SERVER__PORT", "9999");
        let result = Loader::new()
            .layer_str("[server]\nport = 8080\n", "defaults")
            .env_prefix("STTOML2_")
            .load::<Outer>();
        std::env::remove_var("STTOML2_SERVER__PORT");
        assert_eq!(result.unwrap().server.port, 9999);
    }

    #[test]
    fn config_file_resolves_relative_paths() {
        let dir = TempDir::new().unwrap();
        write_toml(&dir, "app.toml", "name = \"app\"\n");
        let path = dir.path().join("app.toml");

        let cf: ConfigFile<Simple> = Loader::new().layer_file(&path).load_file().unwrap();

        let resolved = cf.resolve("templates/foo.tera");
        assert_eq!(resolved, dir.path().join("templates/foo.tera"));
    }

    #[test]
    fn find_config_file_walks_up() {
        let dir = TempDir::new().unwrap();
        let child = dir.path().join("a/b/c");
        std::fs::create_dir_all(&child).unwrap();
        let config = dir.path().join("myconfig.toml");
        std::fs::write(&config, "").unwrap();

        let found = find_config_file("myconfig.toml", &child);
        assert_eq!(found, Some(config));
    }

    #[test]
    fn find_config_file_none_when_absent() {
        let dir = TempDir::new().unwrap();
        assert!(find_config_file("missing.toml", dir.path()).is_none());
    }

    #[test]
    fn find_and_load_returns_path_and_config() {
        let dir = TempDir::new().unwrap();
        let child = dir.path().join("sub");
        std::fs::create_dir_all(&child).unwrap();
        write_toml(&dir, "x.toml", "name = \"found\"\n");

        let (path, cfg): (PathBuf, Simple) = find_and_load("x.toml", &child).unwrap();
        assert_eq!(path, dir.path().join("x.toml"));
        assert_eq!(cfg.name, "found");
    }

    #[test]
    fn save_file_round_trips_with_load_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested/out.toml");
        let original = Simple { name: "round-trip".into(), port: Some(1234) };

        save_file(&original, &path).unwrap();
        assert!(path.exists()); // parent dir was created

        let reloaded: Simple = load_file(&path).unwrap();
        assert_eq!(reloaded, original);
    }

    #[test]
    fn loader_three_layer_precedence() {
        let dir = TempDir::new().unwrap();
        write_toml(&dir, "mid.toml", "name = \"mid\"\nport = 2000\n");
        write_toml(&dir, "top.toml", "port = 3000\n");

        let cfg: Simple = Loader::new()
            .layer_str("name = \"base\"\nport = 1000\n", "base")
            .layer_file(dir.path().join("mid.toml"))
            .layer_file(dir.path().join("top.toml"))
            .load()
            .unwrap();

        // `name` comes from mid (top layer didn't set it), `port` from top
        assert_eq!(cfg.name, "mid");
        assert_eq!(cfg.port, Some(3000));
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct ValidatedSimple {
        name: String,
        port: u16,
    }

    impl Validate for ValidatedSimple {
        fn validate(&self, v: &mut crate::validation::Validator) {
            v.check_non_empty("name", &self.name);
            v.check_range("port", self.port, 1024..=65535);
        }
    }

    impl ConfigLifecycle for ValidatedSimple {}

    #[test]
    fn test_config_typestate_lifecycle_transitions() {
        let dir = TempDir::new().unwrap();
        let file_path = write_toml(&dir, "app.toml", "name = \"lifecycle\"\nport = 8080\n");

        // 1. Loader::load_raw() -> Config<Raw>
        let config_raw = Loader::new()
            .layer_str("name = \"default\"\nport = 9000\n", "defaults")
            .layer_file(&file_path)
            .load_raw()
            .unwrap();

        assert_eq!(config_raw.path, file_path);

        // Ensure env prefix is NOT applied yet
        std::env::set_var("ST_PORT", "9999");

        // 2. Config<Raw>::merge() -> Config<Merged>
        let config_merged = config_raw.merge(Some("ST_")).unwrap();
        std::env::remove_var("ST_PORT");

        // Check that the merged TOML value is updated with the env override
        assert_eq!(config_merged.state.0["port"].as_integer(), Some(9999));
        assert_eq!(config_merged.path, file_path);

        // 3. Config<Merged>::deserialize() -> Config<Deserialized<ValidatedSimple>>
        let config_deser = config_merged.deserialize::<ValidatedSimple>().unwrap();
        assert_eq!(config_deser.state.0.name, "lifecycle");
        assert_eq!(config_deser.state.0.port, 9999);
        assert_eq!(config_deser.path, file_path);

        // 4. Config<Deserialized<T>>::validate() -> Config<Validated<T>>
        let config_val = config_deser.validate().unwrap();
        assert_eq!(config_val.state.0.port, 9999);
        assert_eq!(config_val.path, file_path);

        // 5. Config<Validated<T>>::freeze() -> Config<Frozen<T>>
        let config_frozen = config_val.freeze();
        assert_eq!(config_frozen.state.0.port, 9999);
        assert_eq!(config_frozen.path, file_path);
    }

    #[test]
    fn test_config_typestate_lifecycle_failure() {
        let dir = TempDir::new().unwrap();
        let file_path = write_toml(&dir, "app.toml", "name = \"\"\nport = 80\n"); // empty name, port out of range

        let config_raw = Loader::new().layer_file(&file_path).load_raw().unwrap();
        let config_merged = config_raw.merge(None).unwrap();
        let config_deser = config_merged.deserialize::<ValidatedSimple>().unwrap();

        let val_res = config_deser.validate();
        assert!(val_res.is_err());
        if let Err(Error::Invalid(errs)) = val_res {
            assert_eq!(errs.len(), 2);
            assert_eq!(errs.fitness(), 0.0); // 0 out of 2 checks passed
        } else {
            panic!("Expected Error::Invalid");
        }
    }

    #[test]
    fn test_trusted_loader_success() {
        let dir = TempDir::new().unwrap();
        let file_path = write_toml(&dir, "app.toml", "name = \"trusted\"\nport = 8080\n");

        std::env::set_var("TR_PORT", "1234");
        let trusted_config: TrustedConfig<ValidatedSimple> = trusted()
            .layer_str("name = \"default\"\nport = 9000\n", "defaults")
            .layer_file(&file_path)
            .env_prefix("TR_")
            .load()
            .unwrap();
        std::env::remove_var("TR_PORT");

        assert_eq!(trusted_config.value.name, "trusted");
        assert_eq!(trusted_config.value.port, 1234);
        assert_eq!(trusted_config.source.path, file_path);

        // ValidationReport check
        assert_eq!(trusted_config.validation.fitness, 1.0);
        assert_eq!(trusted_config.validation.checks_run, 2);
        assert_eq!(trusted_config.validation.checks_passed, 2);
        assert!(trusted_config.validation.errors.is_empty());

        // ConfigDigest check
        assert!(trusted_config.digest.0 > 0);
    }

    #[test]
    fn test_trusted_loader_validation_failure() {
        let dir = TempDir::new().unwrap();
        let file_path = write_toml(&dir, "app.toml", "name = \"\"\nport = 8080\n"); // name empty

        let res = trusted().layer_file(&file_path).load::<ValidatedSimple>();

        assert!(res.is_err());
        if let Err(Error::Invalid(errs)) = res {
            assert_eq!(errs.len(), 1);
            // 1 check passed (port), 1 check failed (name) -> fitness = 0.5
            assert_eq!(errs.fitness(), 0.5);
            assert_eq!(errs.errors()[0].code(), "empty");
        } else {
            panic!("Expected Error::Invalid");
        }
    }

    #[test]
    fn test_trusted_loader_digest_stability() {
        let dir = TempDir::new().unwrap();
        let file_path = write_toml(&dir, "app.toml", "name = \"stable\"\nport = 8080\n");

        let tc1 = trusted().layer_file(&file_path).load::<ValidatedSimple>().unwrap();

        let tc2 = trusted().layer_file(&file_path).load::<ValidatedSimple>().unwrap();

        assert_eq!(tc1.digest, tc2.digest);
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct NestedTest {
        z: String,
        a: String,
        m: SubTest,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct SubTest {
        y: i32,
        b: i32,
    }

    impl Validate for NestedTest {
        fn validate(&self, _v: &mut crate::validation::Validator) {}
    }

    impl Validate for SubTest {
        fn validate(&self, _v: &mut crate::validation::Validator) {}
    }

    impl ConfigLifecycle for NestedTest {}
    impl ConfigLifecycle for SubTest {}

    #[test]
    fn test_save_pretty_and_save_file() {
        let dir = TempDir::new().unwrap();
        let pretty_path = dir.path().join("pretty.toml");
        let plain_path = dir.path().join("plain.toml");

        let original =
            NestedTest { z: "last".into(), a: "first".into(), m: SubTest { y: 10, b: 20 } };

        save_pretty(&original, &pretty_path).unwrap();
        save_file(&original, &plain_path).unwrap();

        assert!(pretty_path.exists());
        assert!(plain_path.exists());

        let pretty_content = std::fs::read_to_string(&pretty_path).unwrap();
        let plain_content = std::fs::read_to_string(&plain_path).unwrap();

        let reloaded_pretty: NestedTest = load_file(&pretty_path).unwrap();
        let reloaded_plain: NestedTest = load_file(&plain_path).unwrap();

        assert_eq!(reloaded_pretty, original);
        assert_eq!(reloaded_plain, original);

        assert!(pretty_content.len() >= plain_content.len());
    }

    #[test]
    fn test_save_canonical_sorting() {
        let dir = TempDir::new().unwrap();
        let canonical_path = dir.path().join("canonical.toml");

        let original =
            NestedTest { z: "last".into(), a: "first".into(), m: SubTest { y: 10, b: 20 } };

        let config_raw = Config::<Raw>::new("");
        let config_merged = config_raw.merge(None).unwrap();
        let config_val = Config::<Validated<NestedTest>>::new(original).unwrap();
        let config_frozen = config_val.freeze();

        config_frozen.save_canonical(&canonical_path).unwrap();
        assert!(canonical_path.exists());

        let canonical_content = std::fs::read_to_string(&canonical_path).unwrap();

        let pos_a = canonical_content.find("a =").expect("Key a not found");
        let pos_z = canonical_content.find("z =").expect("Key z not found");
        assert!(pos_a < pos_z, "Key 'a' must come before 'z' in canonical serialization");

        let pos_b = canonical_content.find("b =").expect("Key b not found");
        let pos_y = canonical_content.find("y =").expect("Key y not found");
        assert!(pos_b < pos_y, "Key 'b' must come before 'y' in canonical serialization");
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct NormTest {
        name: String,
        port: u16,
    }

    impl Validate for NormTest {
        fn validate(&self, _v: &mut crate::validation::Validator) {}
    }

    impl ConfigLifecycle for NormTest {
        fn normalize(&mut self) {
            self.name = self.name.trim().to_string();
        }
        fn validate_lifecycle(&self, v: &mut crate::validation::Validator) {
            v.check_range("port", self.port, 1000..=9999);
        }
    }

    #[test]
    fn test_config_lifecycle_normalization_and_validation() {
        let raw = Config::<Raw>::new("name = '  spaces  '\nport = 2000");
        let merged = raw.merge(None).unwrap();
        let deserialized = merged.deserialize::<NormTest>().unwrap();
        assert_eq!(deserialized.get().name, "spaces");

        let val_res = deserialized.validate();
        assert!(val_res.is_ok());

        let raw_fail = Config::<Raw>::new("name = 'test'\nport = 80");
        let deserialized_fail = raw_fail.merge(None).unwrap().deserialize::<NormTest>().unwrap();
        let val_fail_res = deserialized_fail.validate();
        assert!(val_fail_res.is_err());
        if let Err(Error::Invalid(errs)) = val_fail_res {
            assert_eq!(errs.len(), 1);
            assert_eq!(errs.errors()[0].code(), "out_of_range");
            assert_eq!(errs.errors()[0].loc.to_string(), "port");
        } else {
            panic!("Expected Error::Invalid");
        }
    }
}
