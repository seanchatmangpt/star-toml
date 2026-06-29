//! Declarative `Schema` builder — validate any `toml::Value` without Rust structs.
//!
//! The Pydantic + Van der Aalst engines in [`validation`](crate::validation) require
//! implementing [`Validate`](crate::Validate) for each config type. That's powerful for
//! typed configs — but sometimes you want to validate a raw TOML file without writing a
//! struct at all.
//!
//! `Schema` is the 1,700× version: declare constraints once, validate any
//! `toml::Value` or TOML string directly.
//!
//! # Phase-shift comparison
//!
//! **Before (imperative `Validate`):**
//! ```
//! use star_toml::{Validate, Validator};
//!
//! struct ServerConfig { host: String, port: u16 }
//!
//! impl Validate for ServerConfig {
//!     fn validate(&self, v: &mut Validator) {
//!         v.check_non_empty("host", &self.host);
//!         v.check_range("port", self.port, 1..=65535);
//!     }
//! }
//!
//! // Must deserialize to the struct first
//! let s = ServerConfig { host: String::new(), port: 0 };
//! let errs = s.check().unwrap_err();
//! assert_eq!(errs.len(), 2);
//! ```
//!
//! **After (declarative `Schema`):**
//! ```
//! use star_toml::Schema;
//!
//! let schema = Schema::new()
//!     .field("host").non_empty().done()
//!     .field("port").range_i64(1, 65535).done();
//!
//! let errs = schema.validate_str("[server]\nhost=''\nport=0").unwrap_err();
//! assert_eq!(errs.len(), 2);
//! ```
//!
//! Each `.field()` call opens a [`FieldBuilder`]; call `.done()` to return to the
//! parent `Schema` before opening the next field.
//!
//! # Nesting
//!
//! ```
//! use star_toml::Schema;
//!
//! let schema = Schema::new()
//!     .field("name").non_empty().done()
//!     .field("workers").range_i64(1, 1024).done()
//!     .section("server", Schema::new()
//!         .field("host").non_empty().done()
//!         .field("port").range_i64(1, 65535).done());
//!
//! let toml = r#"
//! name = "demo"
//! workers = 8
//! [server]
//! host = "localhost"
//! port = 8080
//! "#;
//! assert!(schema.validate_str(toml).is_ok());
//! ```

#[macro_export]
#[doc(hidden)]
macro_rules! schema_key_to_str {
    ($key:ident) => {
        stringify!($key)
    };
    ($key:literal) => {
        $key
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! schema_field_constraint {
    ($fb:expr, non_empty) => {
        $fb.non_empty()
    };
    ($fb:expr, required) => {
        $fb.required()
    };
    ($fb:expr, range($lo:expr, $hi:expr)) => {
        $fb.range_i64($lo, $hi)
    };
    ($fb:expr, range_f64($lo:expr, $hi:expr)) => {
        $fb.range_f64($lo, $hi)
    };
    ($fb:expr, one_of($($allowed:expr),* $(,)?)) => {
        $fb.one_of(&[$($allowed),*])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! schema_field_constraints {
    ($fb:expr, ) => {
        $fb
    };
    ($fb:expr, $cname:ident ($($cargs:tt)*) , $($rest:tt)*) => {
        {
            let fb = $crate::schema_field_constraint!($fb, $cname ($($cargs)*));
            $crate::schema_field_constraints!(fb, $($rest)*)
        }
    };
    ($fb:expr, $cname:ident ($($cargs:tt)*)) => {
        $crate::schema_field_constraint!($fb, $cname ($($cargs)*))
    };
    ($fb:expr, $cname:ident , $($rest:tt)*) => {
        {
            let fb = $crate::schema_field_constraint!($fb, $cname);
            $crate::schema_field_constraints!(fb, $($rest)*)
        }
    };
    ($fb:expr, $cname:ident) => {
        $crate::schema_field_constraint!($fb, $cname)
    };
}

/// Declarative schema builder macro.
///
/// Supports fields with constraints and nested sections.
#[macro_export]
macro_rules! schema {
    // Nested sections: key: { ... }
    ( @entry $s:ident, $key:tt, { $($nested:tt)* } ) => {
        let key_str = $crate::schema_key_to_str!($key);
        let nested_schema = $crate::schema! { $($nested)* };
        $s = $s.section(key_str, nested_schema);
    };

    // Bracketed constraints: key: [required, range(1, 10)]
    ( @entry $s:ident, $key:tt, [ $($constraints:tt)* ] ) => {
        let key_str = $crate::schema_key_to_str!($key);
        let fb = $s.field(key_str);
        let fb = $crate::schema_field_constraints!(fb, $($constraints)*);
        $s = fb.done();
    };

    // Single constraint with args: key: range(1, 10)
    ( @entry $s:ident, $key:tt, $cname:ident ($($cargs:tt)*) ) => {
        let key_str = $crate::schema_key_to_str!($key);
        let fb = $s.field(key_str);
        let fb = $crate::schema_field_constraint!(fb, $cname ($($cargs)*));
        $s = fb.done();
    };

    // Single constraint without args: key: non_empty
    ( @entry $s:ident, $key:tt, $cname:ident ) => {
        let key_str = $crate::schema_key_to_str!($key);
        let fb = $s.field(key_str);
        let fb = $crate::schema_field_constraint!(fb, $cname);
        $s = fb.done();
    };

    // Entry point: comma-separated list of entries
    ( $($key:tt : $val:tt $(($($args:tt)*))? ),* $(,)? ) => {
        {
            let mut s = $crate::Schema::new();
            $(
                $crate::schema!(@entry s, $key, $val $(($($args)*))?);
            )*
            s
        }
    };
}

use toml::Value;

use crate::validation::{ErrorKind, Loc, LocSegment, Severity, ValidationError, ValidationErrors};

// ---------------------------------------------------------------------------
// Constraint — one atomic rule on a TOML value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) enum Constraint {
    NonEmpty,
    RangeI64 { lo: i64, hi: i64 },
    RangeF64 { lo: f64, hi: f64 },
    OneOf { allowed: Vec<String> },
    Required,
    Predicate { code: &'static str, msg: String, test: fn(&Value) -> bool },
}

impl Constraint {
    fn check(
        &self,
        field: &str,
        value: Option<&Value>,
        checks_run: &mut usize,
    ) -> Option<ValidationError> {
        *checks_run += 1;
        match self {
            Self::Required => {
                if value.is_none() {
                    return Some(make_err(field, ErrorKind::Missing, None, "field is required"));
                }
                None
            }
            Self::NonEmpty => {
                let s = value.and_then(Value::as_str).unwrap_or("");
                if s.is_empty() {
                    return Some(make_err(
                        field,
                        ErrorKind::Empty,
                        Some("\"\"".into()),
                        "must not be empty",
                    ));
                }
                None
            }
            Self::RangeI64 { lo, hi } => {
                let n = value.and_then(Value::as_integer).unwrap_or(0);
                if n < *lo || n > *hi {
                    let msg = format!("input must be in range {lo}..={hi}");
                    return Some(make_err(
                        field,
                        ErrorKind::OutOfRange {
                            lower: Some(lo.to_string()),
                            upper: Some(hi.to_string()),
                        },
                        Some(n.to_string()),
                        msg,
                    ));
                }
                None
            }
            Self::RangeF64 { lo, hi } => {
                let n = value.and_then(Value::as_float).unwrap_or(0.0);
                if n.is_nan() || n < *lo || n > *hi {
                    let msg = format!("input must be in range {lo}..={hi}");
                    return Some(make_err(
                        field,
                        ErrorKind::OutOfRange {
                            lower: Some(lo.to_string()),
                            upper: Some(hi.to_string()),
                        },
                        Some(n.to_string()),
                        msg,
                    ));
                }
                None
            }
            Self::OneOf { allowed } => {
                let s = value.and_then(Value::as_str).unwrap_or("");
                if !allowed.iter().any(|a| a == s) {
                    let msg = format!("must be one of: {}", allowed.join(", "));
                    return Some(make_err(
                        field,
                        ErrorKind::NotOneOf { allowed: allowed.clone() },
                        Some(s.to_string()),
                        msg,
                    ));
                }
                None
            }
            Self::Predicate { code, msg, test } => {
                if !test(value.unwrap_or(&Value::Boolean(false))) {
                    return Some(make_err(field, ErrorKind::Predicate { code }, None, msg.clone()));
                }
                None
            }
        }
    }
}

fn make_err(
    field: &str,
    kind: ErrorKind,
    input: Option<String>,
    msg: impl Into<String>,
) -> ValidationError {
    ValidationError {
        loc: Loc(vec![LocSegment::Key(field.to_string())]),
        kind,
        severity: Severity::Error,
        input,
        msg: msg.into(),
    }
}

// ---------------------------------------------------------------------------
// FieldBuilder — fluent constraint attachment
// ---------------------------------------------------------------------------

/// Fluent builder returned by [`Schema::field`].
///
/// Attach constraints with method chaining; call [`done`](FieldBuilder::done)
/// to return to the parent schema.
pub struct FieldBuilder<'a> {
    schema: &'a mut Schema,
    name: String,
}

impl<'a> FieldBuilder<'a> {
    /// Fail with `empty` if the value is an empty string.
    pub fn non_empty(self) -> Self {
        self.add(Constraint::NonEmpty)
    }

    /// Fail with `out_of_range` if the integer value is outside `[lo, hi]`.
    pub fn range_i64(self, lo: i64, hi: i64) -> Self {
        self.add(Constraint::RangeI64 { lo, hi })
    }

    /// Fail with `out_of_range` if the float value is outside `[lo, hi]`.
    pub fn range_f64(self, lo: f64, hi: f64) -> Self {
        self.add(Constraint::RangeF64 { lo, hi })
    }

    /// Fail with `not_one_of` if the string value is not in `allowed`.
    pub fn one_of(self, allowed: &[&str]) -> Self {
        let allowed = allowed.iter().map(|s| (*s).to_string()).collect();
        self.add(Constraint::OneOf { allowed })
    }

    /// Fail with `missing` if the field is absent from the config.
    pub fn required(self) -> Self {
        self.add(Constraint::Required)
    }

    /// Fail with `code` when `test(value)` returns false.
    pub fn predicate(
        self,
        code: &'static str,
        msg: impl Into<String>,
        test: fn(&Value) -> bool,
    ) -> Self {
        self.add(Constraint::Predicate { code, msg: msg.into(), test })
    }

    pub fn description(self, desc: &str) -> Self {
        self.schema.field_descriptions.insert(self.name.clone(), desc.to_string());
        self
    }

    pub fn default(self, val: &str) -> Self {
        self.schema.field_defaults.insert(self.name.clone(), val.to_string());
        self
    }

    pub fn allowed_values(self, vals: &[&str]) -> Self {
        let v = vals.iter().map(|s| (*s).to_string()).collect();
        self.schema.field_allowed.insert(self.name.clone(), v);
        self
    }

    pub fn forbidden_values(self, vals: &[&str]) -> Self {
        let v = vals.iter().map(|s| (*s).to_string()).collect();
        self.schema.field_forbidden.insert(self.name.clone(), v);
        self
    }

    pub fn stability(self, level: &str) -> Self {
        self.schema.field_stabilities.insert(self.name.clone(), level.to_string());
        self
    }

    fn add(self, c: Constraint) -> Self {
        let idx = self.schema.fields.iter().position(|(n, _)| n == &self.name);
        match idx {
            Some(i) => self.schema.fields[i].1.push(c),
            None => self.schema.fields.push((self.name.clone(), vec![c])),
        }
        self
    }

    /// Finish field configuration and return to the parent [`Schema`].
    pub fn done(self) -> Schema {
        Schema {
            fields: self.schema.fields.clone(),
            sections: self.schema.sections.clone(),
            domain_id: self.schema.domain_id.clone(),
            domain_name: self.schema.domain_name.clone(),
            domain_description: self.schema.domain_description.clone(),
            field_descriptions: self.schema.field_descriptions.clone(),
            field_defaults: self.schema.field_defaults.clone(),
            field_allowed: self.schema.field_allowed.clone(),
            field_forbidden: self.schema.field_forbidden.clone(),
            field_stabilities: self.schema.field_stabilities.clone(),
            authority_boundary: self.schema.authority_boundary.clone(),
            override_policy: self.schema.override_policy.clone(),
            stability: self.schema.stability.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Schema — the declarative model
// ---------------------------------------------------------------------------

/// Declarative validation schema for raw TOML values.
///
/// Build with [`Schema::new`] and the fluent [`field`](Schema::field) /
/// [`section`](Schema::section) methods, then call [`validate_value`](Schema::validate_value),
/// [`validate_str`](Schema::validate_str), or use as a [`Validate`](crate::Validate)
/// implementor via [`into_validator`](Schema::into_validator).
///
/// ```
/// use star_toml::Schema;
///
/// let schema = Schema::new()
///     .field("name").non_empty().done()
///     .field("port").range_i64(1, 65535).done();
///
/// let ok = schema.validate_str("name = \"app\"\nport = 8080");
/// assert!(ok.is_ok());
///
/// let errs = schema.validate_str("name = \"\"\nport = 0").unwrap_err();
/// assert_eq!(errs.len(), 2);
/// assert_eq!(errs.fitness(), 0.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub(crate) fields: Vec<(String, Vec<Constraint>)>,
    pub(crate) sections: Vec<(String, Schema)>,
    pub(crate) domain_id: Option<String>,
    pub(crate) domain_name: Option<String>,
    pub(crate) domain_description: Option<String>,
    pub(crate) field_descriptions: std::collections::HashMap<String, String>,
    pub(crate) field_defaults: std::collections::HashMap<String, String>,
    pub(crate) field_allowed: std::collections::HashMap<String, Vec<String>>,
    pub(crate) field_forbidden: std::collections::HashMap<String, Vec<String>>,
    pub(crate) field_stabilities: std::collections::HashMap<String, String>,
    pub(crate) authority_boundary: Option<crate::docs::AuthorityBoundary>,
    pub(crate) override_policy: Option<crate::docs::OverridePolicy>,
    pub(crate) stability: Option<String>,
}

impl Schema {
    /// Create an empty schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start adding constraints for `name`.
    ///
    /// Returns a [`FieldBuilder`]; call `.done()` to return to the schema.
    pub fn field(&mut self, name: &str) -> FieldBuilder<'_> {
        FieldBuilder { schema: self, name: name.to_string() }
    }

    /// Add a nested sub-schema for table key `name`.
    ///
    /// Errors inside the sub-schema are prefixed with `name.`.
    ///
    /// ```
    /// use star_toml::Schema;
    ///
    /// let schema = Schema::new()
    ///     .section("db", Schema::new()
    ///         .field("host").non_empty().done()
    ///         .field("port").range_i64(1, 65535).done());
    ///
    /// let errs = schema.validate_str("[db]\nhost=''\nport=0").unwrap_err();
    /// assert_eq!(errs.errors()[0].loc.to_string(), "db.host");
    /// ```
    pub fn section(mut self, name: &str, sub: Schema) -> Self {
        self.sections.push((name.to_string(), sub));
        self
    }

    pub fn domain_metadata(mut self, id: &str, name: &str, description: &str) -> Self {
        self.domain_id = Some(id.to_string());
        self.domain_name = Some(name.to_string());
        self.domain_description = Some(description.to_string());
        self
    }

    pub fn authority_boundary(mut self, boundary: crate::docs::AuthorityBoundary) -> Self {
        self.authority_boundary = Some(boundary);
        self
    }

    pub fn override_policy(mut self, policy: crate::docs::OverridePolicy) -> Self {
        self.override_policy = Some(policy);
        self
    }

    pub fn stability(mut self, level: &str) -> Self {
        self.stability = Some(level.to_string());
        self
    }

    /// Validate a `toml::Value` (typically the root table after parsing).
    ///
    /// # Errors
    ///
    /// Returns [`ValidationErrors`] if any constraint is violated.
    pub fn validate_value(&self, value: &Value) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();
        let mut checks_run = 0usize;
        self.check_value(value, &[], &mut errors, &mut checks_run);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors { errors, title: None, checks_run })
        }
    }

    /// Parse `toml_str`, then validate the resulting value.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValidationErrors)` when parsing fails (single synthetic error)
    /// or when any constraint is violated.
    pub fn validate_str(&self, toml_str: &str) -> Result<(), ValidationErrors> {
        let value: Value = match toml::from_str(toml_str) {
            Ok(v) => v,
            Err(e) => {
                let err = ValidationError {
                    loc: Loc::default(),
                    kind: ErrorKind::Predicate { code: "parse_error" },
                    severity: Severity::Fatal,
                    input: None,
                    msg: e.to_string(),
                };
                return Err(ValidationErrors {
                    errors: vec![err],
                    title: Some("TOML".into()),
                    checks_run: 0,
                });
            }
        };
        self.validate_value(&value)
    }

    /// Total number of fields (direct + nested) that have constraints.
    ///
    /// Useful for computing expected check counts in tests.
    #[must_use]
    pub fn constraint_count(&self) -> usize {
        let direct: usize = self.fields.iter().map(|(_, cs)| cs.len()).sum();
        let nested: usize = self.sections.iter().map(|(_, s)| s.constraint_count()).sum();
        direct + nested
    }

    // -- internal ----------------------------------------------------------

    fn check_value(
        &self,
        value: &Value,
        prefix: &[LocSegment],
        errors: &mut Vec<ValidationError>,
        checks_run: &mut usize,
    ) {
        for (name, constraints) in &self.fields {
            let child = value.get(name.as_str());
            for c in constraints {
                if let Some(mut e) = c.check(name, child, checks_run) {
                    let mut loc_segs = prefix.to_vec();
                    loc_segs.extend(e.loc.0.drain(..));
                    e.loc = Loc(loc_segs);
                    errors.push(e);
                }
            }
        }
        for (section_name, sub_schema) in &self.sections {
            let sub_value = value.get(section_name.as_str());
            let mut sub_prefix = prefix.to_vec();
            sub_prefix.push(LocSegment::Key(section_name.clone()));
            match sub_value {
                Some(v) => sub_schema.check_value(v, &sub_prefix, errors, checks_run),
                None => {
                    // If the section is absent, each required field in it is missing
                    sub_schema.report_section_missing(
                        section_name,
                        &sub_prefix,
                        errors,
                        checks_run,
                    );
                }
            }
        }
    }

    fn report_section_missing(
        &self,
        _section: &str,
        prefix: &[LocSegment],
        errors: &mut Vec<ValidationError>,
        checks_run: &mut usize,
    ) {
        for (name, constraints) in &self.fields {
            for c in constraints {
                if let Some(mut e) = c.check(name, None, checks_run) {
                    let mut loc_segs = prefix.to_vec();
                    loc_segs.extend(e.loc.0.drain(..));
                    e.loc = Loc(loc_segs);
                    errors.push(e);
                }
            }
        }
        for (sub_section_name, sub_schema) in &self.sections {
            let mut sub_prefix = prefix.to_vec();
            sub_prefix.push(LocSegment::Key(sub_section_name.clone()));
            sub_schema.report_section_missing(sub_section_name, &sub_prefix, errors, checks_run);
        }
    }

    pub fn get_domain(&self) -> crate::docs::ConfigDomain {
        let domain_id = self.domain_id.clone().unwrap_or_else(|| "anonymous_domain".to_string());
        let domain_name =
            self.domain_name.clone().unwrap_or_else(|| "Anonymous Domain".to_string());
        let domain_description = self
            .domain_description
            .clone()
            .unwrap_or_else(|| "No description provided.".to_string());

        let mut fields = Vec::new();
        let mut validators = Vec::new();
        let mut counterexamples = Vec::new();

        for (field_name, constraints) in &self.fields {
            let mut field_type = "String".to_string();
            let mut required = false;
            let mut val_refs = Vec::new();
            let mut ce_refs = Vec::new();

            for c in constraints {
                let val_name = match c {
                    Constraint::NonEmpty => "non_empty".to_string(),
                    Constraint::Required => {
                        required = true;
                        "required".to_string()
                    }
                    Constraint::RangeI64 { .. } => {
                        field_type = "integer".to_string();
                        "range_i64".to_string()
                    }
                    Constraint::RangeF64 { .. } => {
                        field_type = "float".to_string();
                        "range_f64".to_string()
                    }
                    Constraint::OneOf { .. } => "one_of".to_string(),
                    Constraint::Predicate { code, .. } => code.to_string(),
                };

                let val_id = format!("{}_{}_{}", domain_id, field_name, val_name);
                val_refs.push(val_id.clone());

                let ce_code = match c {
                    Constraint::NonEmpty => "empty".to_string(),
                    Constraint::Required => "missing".to_string(),
                    Constraint::RangeI64 { .. } | Constraint::RangeF64 { .. } => {
                        "out_of_range".to_string()
                    }
                    Constraint::OneOf { .. } => "not_one_of".to_string(),
                    Constraint::Predicate { code, .. } => code.to_string(),
                };
                ce_refs.push(ce_code.clone());

                let val_desc = match c {
                    Constraint::NonEmpty => "must not be empty".to_string(),
                    Constraint::Required => "field is required".to_string(),
                    Constraint::RangeI64 { lo, hi } => format!("must be in range {}..={}", lo, hi),
                    Constraint::RangeF64 { lo, hi } => format!("must be in range {}..={}", lo, hi),
                    Constraint::OneOf { allowed } => {
                        format!("must be one of: {}", allowed.join(", "))
                    }
                    Constraint::Predicate { msg, .. } => msg.clone(),
                };

                let ce_desc = match c {
                    Constraint::NonEmpty => "value is empty".to_string(),
                    Constraint::Required => "required field is missing".to_string(),
                    Constraint::RangeI64 { .. } | Constraint::RangeF64 { .. } => {
                        "value is out of range".to_string()
                    }
                    Constraint::OneOf { allowed } => {
                        format!("must be one of: {}", allowed.join(", "))
                    }
                    Constraint::Predicate { msg, .. } => msg.clone(),
                };

                let ce_hint = match c {
                    Constraint::NonEmpty => "provide a non-empty string".to_string(),
                    Constraint::Required => "provide the required field".to_string(),
                    Constraint::RangeI64 { lo, hi } => {
                        format!("provide a value between {} and {}", lo, hi)
                    }
                    Constraint::RangeF64 { lo, hi } => {
                        format!("provide a value between {} and {}", lo, hi)
                    }
                    Constraint::OneOf { allowed } => {
                        format!("choose one of: {}", allowed.join(", "))
                    }
                    Constraint::Predicate { .. } => "correct the value".to_string(),
                };

                validators.push(crate::docs::ConfigValidator {
                    id: val_id,
                    name: val_name,
                    predicate: "is_valid".to_string(),
                    description: val_desc,
                    applies_to: vec![field_name.clone()],
                    failure_counterexamples: vec![ce_code.clone()],
                    severity: "Error".to_string(),
                });

                if !counterexamples
                    .iter()
                    .any(|existing: &crate::docs::Counterexample| existing.code == ce_code)
                {
                    counterexamples.push(crate::docs::Counterexample {
                        code: ce_code,
                        name: field_name.clone(),
                        description: ce_desc,
                        severity: "Error".to_string(),
                        detection_rule: "validation_fails".to_string(),
                        repair_hint: ce_hint,
                    });
                }
            }

            let field_desc = self.field_descriptions.get(field_name).cloned().unwrap_or_default();
            let default_val = self.field_defaults.get(field_name).cloned();
            let allowed_vals = self.field_allowed.get(field_name).cloned().unwrap_or_default();
            let forbidden_vals = self.field_forbidden.get(field_name).cloned().unwrap_or_default();
            let field_stability = self
                .field_stabilities
                .get(field_name)
                .cloned()
                .unwrap_or_else(|| "stable".to_string());

            fields.push(crate::docs::ConfigField {
                id: format!("{}.{}", domain_id, field_name),
                domain_id: domain_id.clone(),
                name: field_name.clone(),
                r#type: field_type,
                required,
                default: default_val,
                allowed_values: allowed_vals,
                forbidden_values: forbidden_vals,
                description: field_desc,
                validator_refs: val_refs,
                counterexample_refs: ce_refs,
                override_policy_ref: None,
                source_binding: "schema".to_string(),
                stability: field_stability,
            });
        }

        // Recursively handle nested sections
        for (sec_name, sec_schema) in &self.sections {
            let mut sec_domain = sec_schema.get_domain();
            sec_domain.id = format!("{}.{}", domain_id, sec_name);
            fields.extend(sec_domain.fields.drain(..).map(|mut f| {
                f.domain_id = domain_id.clone();
                f
            }));
            validators.extend(sec_domain.validators.drain(..));
            counterexamples.extend(sec_domain.counterexamples.drain(..));
        }

        let authority_boundary =
            self.authority_boundary.clone().unwrap_or_else(|| crate::docs::AuthorityBoundary {
                id: format!("{}_boundary", domain_id),
                domain: domain_id.clone(),
                may_control: vec![],
                must_not_claim: vec![],
                defer_to: None,
                escalation_rule: None,
            });

        let override_policy =
            self.override_policy.clone().unwrap_or_else(|| crate::docs::OverridePolicy {
                id: format!("{}_override", domain_id),
                allowed: true,
                required_reason: false,
                required_scope: vec![],
                required_expiration: None,
                required_tracking: None,
                required_replacement_plan: false,
                refusal_conditions: vec![],
            });

        let stability_str = self.stability.clone().unwrap_or_else(|| "stable".to_string());

        crate::docs::ConfigDomain {
            id: domain_id,
            name: domain_name,
            description: domain_description,
            authority_boundary,
            fields,
            validators,
            counterexamples,
            override_policy,
            receipt_binding: None,
            stability: stability_str,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn server_schema() -> Schema {
        Schema::new().field("host").non_empty().done().field("port").range_i64(1, 65535).done()
    }

    fn app_schema() -> Schema {
        Schema::new()
            .field("name")
            .non_empty()
            .done()
            .field("workers")
            .range_i64(1, 1024)
            .done()
            .field("log_level")
            .one_of(&["trace", "debug", "info", "warn", "error"])
            .done()
            .section("server", server_schema())
    }

    #[test]
    fn valid_config_passes() {
        let toml = r#"
name = "demo"
workers = 8
log_level = "info"
[server]
host = "localhost"
port = 8080
"#;
        assert!(app_schema().validate_str(toml).is_ok());
    }

    #[test]
    fn collects_all_errors() {
        let toml = r#"
name = ""
workers = 0
log_level = "verbose"
[server]
host = ""
port = 0
"#;
        let errs = app_schema().validate_str(toml).unwrap_err();
        assert_eq!(errs.len(), 5);
    }

    #[test]
    fn nested_section_paths_are_prefixed() {
        let toml = "name = \"ok\"\nworkers = 4\nlog_level = \"info\"\n[server]\nhost = \"\"\nport = 8080\n";
        let errs = app_schema().validate_str(toml).unwrap_err();
        let locs: Vec<String> = errs.errors().iter().map(|e| e.loc.to_string()).collect();
        assert!(locs.contains(&"server.host".to_string()));
    }

    #[test]
    fn fitness_reflects_partial_pass() {
        let toml = "name = \"ok\"\nworkers = 0\nlog_level = \"info\"\n[server]\nhost = \"h\"\nport = 8080\n";
        let errs = app_schema().validate_str(toml).unwrap_err();
        assert!(errs.fitness() > 0.0 && errs.fitness() < 1.0);
    }

    #[test]
    fn variant_id_stable_across_equal_error_patterns() {
        let toml =
            "name = \"\"\nworkers = 1\nlog_level = \"info\"\n[server]\nhost = \"h\"\nport = 80\n";
        let id1 = app_schema().validate_str(toml).unwrap_err().variant_id();
        let id2 = app_schema().validate_str(toml).unwrap_err().variant_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn parse_error_produces_fatal_error() {
        let errs = Schema::new().validate_str("not valid toml :::").unwrap_err();
        assert!(errs.errors()[0].is_fatal());
        assert_eq!(errs.errors()[0].code(), "parse_error");
    }

    #[test]
    fn one_of_constraint() {
        let schema = Schema::new().field("level").one_of(&["info", "warn", "error"]).done();
        assert!(schema.validate_str("level = \"info\"").is_ok());
        let errs = schema.validate_str("level = \"verbose\"").unwrap_err();
        assert_eq!(errs.errors()[0].code(), "not_one_of");
    }

    #[test]
    fn range_f64_constraint() {
        let schema = Schema::new().field("ratio").range_f64(0.0, 1.0).done();
        assert!(schema.validate_str("ratio = 0.5").is_ok());
        let errs = schema.validate_str("ratio = 2.0").unwrap_err();
        assert_eq!(errs.errors()[0].code(), "out_of_range");
    }

    #[test]
    fn predicate_constraint() {
        let schema = Schema::new()
            .field("port")
            .predicate("no_well_known", "prefer ports above 1024", |v| {
                v.as_integer().map_or(true, |n| n > 1024)
            })
            .done();
        assert!(schema.validate_str("port = 8080").is_ok());
        let errs = schema.validate_str("port = 80").unwrap_err();
        assert_eq!(errs.errors()[0].code(), "no_well_known");
    }

    #[test]
    fn by_section_grouping_works_on_schema_errors() {
        let toml =
            "name = \"\"\nworkers = 0\nlog_level = \"info\"\n[server]\nhost = \"\"\nport = 8080\n";
        let errs = app_schema().validate_str(toml).unwrap_err();
        let by_sec = errs.by_section();
        assert!(by_sec.contains_key("name"));
        assert!(by_sec.contains_key("workers"));
        assert!(by_sec.contains_key("server"));
    }
}
