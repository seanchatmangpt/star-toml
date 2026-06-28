//! Per-document TOML analyzer.
//!
//! Produces LSP diagnostics from:
//!   - TOML parse errors
//!   - Duplicate key detection
//!   - Path traversal / RelativeOnly violations (static scan)
//!   - OCEL/q_config authority boundary notes
//!
//! Does NOT compute q_config. Does NOT construct AdmittedConfig<T> or ConfigWitness.

use lsp_max::lsp_types_max::{
    CompletionItem, CompletionItemKind, Diagnostic, DocumentSymbol, Hover, HoverContents,
    MarkupContent, MarkupKind, Position, Range, SymbolKind, Url,
};
use std::collections::HashMap;
use toml::Value;

use crate::diagnostics::{
    self, duplicate_key, find_authority_boundary_diagnostics, find_key_range_in_text,
    find_path_traversal_diagnostics, invalid_toml, line_start_offsets, offset_to_position,
};

fn pos(line: u32, col: u32) -> Position {
    Position { line, character: col }
}

/// Cached per-document analyzer.
#[derive(Debug, Clone)]
pub struct StarTomlDocumentAnalyzer {
    pub uri: Url,
    pub text: String,
    /// Parsed TOML value; None if parse failed.
    pub parsed: Option<Value>,
    diagnostics: Vec<Diagnostic>,
}

impl StarTomlDocumentAnalyzer {
    pub fn new(uri: Url, text: String) -> Self {
        let mut a = Self {
            uri,
            text,
            parsed: None,
            diagnostics: Vec::new(),
        };
        a.analyze();
        a
    }

    pub fn update(&mut self, text: String) {
        self.text = text;
        self.parsed = None;
        self.diagnostics.clear();
        self.analyze();
    }

    /// Diagnostics from the last analysis pass.
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    fn analyze(&mut self) {
        // 1. TOML parse
        match toml::from_str::<Value>(&self.text) {
            Ok(val) => {
                self.parsed = Some(val);
            }
            Err(e) => {
                let range = e
                    .span()
                    .map(|span| {
                        let starts = line_start_offsets(&self.text);
                        let start = offset_to_position(span.start, &starts);
                        let end = offset_to_position(span.end, &starts);
                        Range { start, end }
                    })
                    .unwrap_or_else(|| diagnostics::document_start());
                self.diagnostics.push(invalid_toml(range, &e.to_string()));
                return;
            }
        }

        // 2. Duplicate keys (raw scan)
        self.diagnostics.extend(detect_duplicate_keys(&self.text));

        // 3. Path traversal / absolute-path warnings (static scan)
        self.diagnostics.extend(find_path_traversal_diagnostics(&self.text));

        // 4. OCEL/q_config authority boundary notes
        self.diagnostics.extend(find_authority_boundary_diagnostics(&self.text));
    }

    /// Merge validation errors from a caller-supplied `ValidationErrors`.
    /// Use this after deserializing into a typed struct to surface schema violations.
    pub fn merge_validation_errors(&mut self, errors: &star_toml::ValidationErrors) {
        for e in errors.errors() {
            self.diagnostics
                .push(diagnostics::from_validation_error(e, &self.text));
        }
    }

    // ---------------------------------------------------------------------------
    // Hover
    // ---------------------------------------------------------------------------

    pub fn hover_at(&self, line: u32, character: u32) -> Option<Hover> {
        let val = self.parsed.as_ref()?;
        let key = token_at(&self.text, line, character)?;
        let table_path = current_table_path(&self.text, line);
        let table = navigate_to(val, &table_path)?;
        let Value::Table(map) = table else { return None };
        let child = map.get(&key)?;

        let kind_str = toml_kind_str(child);
        let preview = truncate(&child.to_string(), 120);
        let scope = if table_path.is_empty() {
            "document root".to_owned()
        } else {
            format!("[{}]", table_path.join("."))
        };

        let md = format!(
            "**`{key}`** — *{kind_str}* in `{scope}`\n\n\
             ```toml\n{key} = {preview}\n```\n\n\
             > star-toml-lsp: diagnostics only — does not grant q_config standing."
        );

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: None,
        })
    }

    // ---------------------------------------------------------------------------
    // Completions
    // ---------------------------------------------------------------------------

    pub fn completion_at(&self, line: u32, _character: u32) -> Vec<CompletionItem> {
        let Some(val) = self.parsed.as_ref() else { return Vec::new() };
        let table_path = current_table_path(&self.text, line);
        let Some(Value::Table(map)) = navigate_to(val, &table_path) else {
            return Vec::new();
        };
        map.keys()
            .map(|k| CompletionItem {
                label: k.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("key in [{}]", table_path.join("."))),
                ..Default::default()
            })
            .collect()
    }

    // ---------------------------------------------------------------------------
    // Document symbols
    // ---------------------------------------------------------------------------

    pub fn document_symbols(&self) -> Vec<DocumentSymbol> {
        let Some(val) = self.parsed.as_ref() else { return Vec::new() };
        let line_starts = line_start_offsets(&self.text);
        let mut out = Vec::new();
        collect_symbols(val, "", &self.text, &line_starts, &mut out);
        out
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn detect_duplicate_keys(text: &str) -> Vec<Diagnostic> {
    let mut seen: HashMap<String, u32> = HashMap::new();
    let mut diags = Vec::new();
    let mut current_table: Option<String> = None;

    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") {
            // Array-of-tables: each element starts a fresh scope for its keys.
            let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            current_table = Some(inner.to_owned());
            seen.clear();
            continue;
        }
        if trimmed.starts_with('[') {
            let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            current_table = Some(inner.to_owned());
            seen.clear();
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim();
            let qualified = match &current_table {
                Some(t) => format!("{t}.{key}"),
                None => key.to_owned(),
            };
            if let Some(&prev) = seen.get(&qualified) {
                let col = line.find(key).unwrap_or(0) as u32;
                let range = Range {
                    start: pos(line_no as u32, col),
                    end: pos(line_no as u32, col + key.len() as u32),
                };
                diags.push(duplicate_key(range, key, prev));
            } else {
                seen.insert(qualified, line_no as u32);
            }
        }
    }
    diags
}

fn current_table_path(text: &str, cursor_line: u32) -> Vec<String> {
    let preceding: Vec<&str> = text.lines().take(cursor_line as usize).collect();
    for line in preceding.into_iter().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            return inner.split('.').map(str::to_owned).collect();
        }
    }
    Vec::new()
}

fn navigate_to<'v>(val: &'v Value, path: &[String]) -> Option<&'v Value> {
    let mut cur = val;
    for key in path {
        cur = cur.as_table()?.get(key)?;
    }
    Some(cur)
}

fn token_at(text: &str, line: u32, character: u32) -> Option<String> {
    let src_line = text.lines().nth(line as usize)?;
    let col = character as usize;
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    let start = src_line[..col].rfind(|c: char| !is_ident(c)).map_or(0, |p| p + 1);
    let end = src_line[col..]
        .find(|c: char| !is_ident(c))
        .map_or(src_line.len(), |p| col + p);
    let tok = src_line[start..end].trim();
    if tok.is_empty() { None } else { Some(tok.to_owned()) }
}

fn toml_kind_str(val: &Value) -> &'static str {
    match val {
        Value::String(_) => "string",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::Boolean(_) => "boolean",
        Value::Datetime(_) => "datetime",
        Value::Array(_) => "array",
        Value::Table(_) => "table",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_owned() } else { format!("{}…", &s[..max]) }
}

fn collect_symbols(
    val: &Value,
    prefix: &str,
    text: &str,
    _line_starts: &[usize],
    out: &mut Vec<DocumentSymbol>,
) {
    let Value::Table(map) = val else { return };
    for (key, child) in map {
        let full_key = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };

        let (kind, detail) = match child {
            Value::Table(_) => (SymbolKind::OBJECT, "table"),
            Value::Array(_) => (SymbolKind::ARRAY, "array"),
            Value::String(_) => (SymbolKind::STRING, "string"),
            Value::Integer(_) => (SymbolKind::NUMBER, "integer"),
            Value::Float(_) => (SymbolKind::NUMBER, "float"),
            Value::Boolean(_) => (SymbolKind::BOOLEAN, "boolean"),
            Value::Datetime(_) => (SymbolKind::PROPERTY, "datetime"),
        };

        let range = find_key_range_in_text(text, &full_key)
            .unwrap_or(diagnostics::document_start());

        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: key.clone(),
            detail: Some(detail.to_owned()),
            kind,
            range,
            selection_range: range,
            children: {
                let mut ch = Vec::new();
                collect_symbols(child, &full_key, text, _line_starts, &mut ch);
                if ch.is_empty() { None } else { Some(ch) }
            },
            tags: None,
            deprecated: None,
        });
    }
}
