//! Diagnostic codes and conversion from star-toml errors to LSP diagnostics.
//!
//! This module defines the poka-yoke diagnostic surface. It does NOT grant
//! q_config standing, construct AdmittedConfig<T>, or create ConfigWitness.
//! All diagnostics are advisory/ANDON only.

use lsp_max::lsp_types_max::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

// ---------------------------------------------------------------------------
// Diagnostic code constants
// ---------------------------------------------------------------------------

pub const INVALID_TOML: &str = "invalid_toml";
pub const UNKNOWN_FIELD: &str = "unknown_field";
pub const PATH_TRAVERSAL: &str = "path_traversal_detected";
pub const RELATIVE_ONLY_ABSOLUTE: &str = "relative_only_escape:absolute_path_forbidden";
pub const DUPLICATE_KEY: &str = "duplicate_key";
pub const OCEL_NOT_STANDING: &str = "ocel_history_not_standing";
pub const Q_CONFIG_AUTHORITY: &str = "q_config_requires_witness_and_failset_zero";
pub const ENV_OVERRIDE_REPORTED: &str = "env_override_reported";

// ---------------------------------------------------------------------------
// Builder helpers
// ---------------------------------------------------------------------------

fn pos(line: u32, col: u32) -> Position {
    Position { line, character: col }
}

pub fn point_range(line: u32, col: u32, len: u32) -> Range {
    Range {
        start: pos(line, col),
        end: pos(line, col + len),
    }
}

pub fn full_line_range(line: u32, text: &str) -> Range {
    let line_len = text.lines().nth(line as usize).map_or(0, |l| l.len()) as u32;
    Range { start: pos(line, 0), end: pos(line, line_len) }
}

pub fn document_start() -> Range {
    Range { start: pos(0, 0), end: pos(0, 1) }
}

fn make(
    range: Range,
    severity: DiagnosticSeverity,
    code: &str,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(code.to_owned())),
        source: Some("star-toml-lsp".to_owned()),
        message: message.into(),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Public constructors
// ---------------------------------------------------------------------------

/// Diagnostic for a TOML parse failure.
pub fn invalid_toml(range: Range, detail: &str) -> Diagnostic {
    make(range, DiagnosticSeverity::ERROR, INVALID_TOML,
        format!("TOML parse error: {detail}"))
}

/// Diagnostic for a key present in the TOML file but absent from the typed schema.
pub fn unknown_field(range: Range, field_path: &str) -> Diagnostic {
    make(range, DiagnosticSeverity::ERROR, UNKNOWN_FIELD,
        format!("unknown field `{field_path}` — not present in schema"))
}

/// Diagnostic for a duplicate key.
pub fn duplicate_key(range: Range, key: &str, first_line: u32) -> Diagnostic {
    make(range, DiagnosticSeverity::ERROR, DUPLICATE_KEY,
        format!("duplicate key `{key}` (first defined on line {})", first_line + 1))
}

/// Diagnostic for a path containing `..` or backslash traversal sequences.
pub fn path_traversal(range: Range, raw: &str) -> Diagnostic {
    make(range, DiagnosticSeverity::ERROR, PATH_TRAVERSAL,
        format!("path traversal detected in `{raw}` — `..` or `\\..` sequences are forbidden"))
}

/// Diagnostic for an absolute path where only relative paths are allowed.
pub fn relative_only_absolute(range: Range, raw: &str) -> Diagnostic {
    make(range, DiagnosticSeverity::ERROR, RELATIVE_ONLY_ABSOLUTE,
        format!("absolute path `{raw}` is forbidden under RelativeOnly policy"))
}

/// Advisory info when the document appears to reference OCEL/lifecycle export.
/// Clarifies that OCEL is process history, not q_config standing.
pub fn ocel_not_standing(range: Range) -> Diagnostic {
    make(range, DiagnosticSeverity::INFORMATION, OCEL_NOT_STANDING,
        "OCEL tracks lifecycle history — it does not grant q_config standing. \
         Standing requires ConfigWitness + failset_cardinality = 0.")
}

/// Advisory info when the document appears to reference q_config or admission.
pub fn q_config_authority_note(range: Range) -> Diagnostic {
    make(range, DiagnosticSeverity::INFORMATION, Q_CONFIG_AUTHORITY,
        "q_config = 1 iff BoundSatisfied ∧ TransformLawful ∧ WitnessComplete ∧ CounterexamplesAbsent. \
         The LSP cannot grant this — use TrustedLoader::load_admitted() at runtime.")
}

// ---------------------------------------------------------------------------
// Convert star-toml ValidationError → LSP Diagnostic
// ---------------------------------------------------------------------------

/// Convert a `star_toml::ValidationError` to an LSP `Diagnostic`.
/// Range falls back to document start if the loc is empty/root.
pub fn from_validation_error(
    err: &star_toml::ValidationError,
    text: &str,
) -> Diagnostic {
    let severity = match err.severity {
        star_toml::Severity::Fatal | star_toml::Severity::Error => DiagnosticSeverity::ERROR,
        star_toml::Severity::Warning => DiagnosticSeverity::WARNING,
        star_toml::Severity::Advisory => DiagnosticSeverity::INFORMATION,
    };

    let loc_path = err.loc.to_string();
    let range = if loc_path.is_empty() || loc_path == "<root>" {
        document_start()
    } else {
        find_key_range_in_text(text, &loc_path).unwrap_or(document_start())
    };

    let hint = err.repair_hint();
    let message = if hint.is_empty() {
        format!("[{}] {}", err.code(), err.msg)
    } else {
        format!("[{}] {} — hint: {}", err.code(), err.msg, hint)
    };

    make(range, severity, err.code(), message)
}

// ---------------------------------------------------------------------------
// Span helpers
// ---------------------------------------------------------------------------

/// Find the range of the last dotted segment of `dotted_path` in `text`.
/// Returns None if not found (caller falls back to document_start).
pub fn find_key_range_in_text(text: &str, dotted_path: &str) -> Option<Range> {
    let last_key = dotted_path.split('.').next_back().unwrap_or(dotted_path);
    let line_starts = line_start_offsets(text);

    let needles = [
        format!("{last_key} ="),
        format!("{last_key}="),
        format!("\"{last_key}\" ="),
        format!("[{last_key}]"),
    ];
    for needle in &needles {
        if let Some(offset) = text.find(needle.as_str()) {
            let start = offset_to_position(offset, &line_starts);
            let end = Position {
                line: start.line,
                character: start.character + last_key.len() as u32,
            };
            return Some(Range { start, end });
        }
    }
    None
}

/// Find the byte offset of each line start (0-based) in `text`.
pub fn line_start_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Map a byte offset to an LSP Position.
pub fn offset_to_position(offset: usize, line_starts: &[usize]) -> Position {
    let line = line_starts.partition_point(|&s| s <= offset).saturating_sub(1);
    let col = offset - line_starts[line];
    pos(line as u32, col as u32)
}

/// Detect path values in the raw TOML string that contain traversal sequences.
/// Returns `(range, raw_value)` pairs for any suspicious string values.
pub fn find_path_traversal_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        if let Some(eq) = line.find('=') {
            let value_part = line[eq + 1..].trim();
            // Strip surrounding quotes from string values
            let raw = if (value_part.starts_with('"') && value_part.ends_with('"'))
                || (value_part.starts_with('\'') && value_part.ends_with('\''))
            {
                &value_part[1..value_part.len() - 1]
            } else {
                continue; // not a string value
            };

            // Normalise backslashes (CE-12 pattern)
            let normalised = raw.replace('\\', "/");

            let has_traversal = normalised.split('/').any(|seg| seg == "..")
                || normalised.contains("/..")
                || normalised.starts_with("..");

            let is_absolute = normalised.starts_with('/');

            if has_traversal {
                let col = line.find(raw).unwrap_or(eq + 1) as u32;
                let range = Range {
                    start: pos(line_no as u32, col),
                    end: pos(line_no as u32, col + raw.len() as u32),
                };
                out.push(path_traversal(range, raw));
            } else if is_absolute {
                // Advisory — only an error when policy is RelativeOnly, which we
                // cannot know without the schema. Surface as WARNING.
                let col = line.find(raw).unwrap_or(eq + 1) as u32;
                let range = Range {
                    start: pos(line_no as u32, col),
                    end: pos(line_no as u32, col + raw.len() as u32),
                };
                out.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(RELATIVE_ONLY_ABSOLUTE.to_owned())),
                    source: Some("star-toml-lsp".to_owned()),
                    message: format!(
                        "absolute path `{raw}` may be forbidden if RelativeOnly policy is active"
                    ),
                    ..Default::default()
                });
            }
        }
    }
    out
}

/// Scan for string values that look like OCEL/q_config references.
pub fn find_authority_boundary_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let lo = line.to_ascii_lowercase();
        if lo.contains("ocel") && (lo.contains("standing") || lo.contains("q_config")) {
            let range = full_line_range(line_no as u32, text);
            out.push(ocel_not_standing(range));
        }
        if lo.contains("q_config") && !lo.starts_with('#') {
            let range = full_line_range(line_no as u32, text);
            out.push(q_config_authority_note(range));
        }
    }
    out
}
