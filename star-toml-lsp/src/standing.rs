//! Workspace-level standing report for the ANDON surface.
//!
//! Reads docs/config_metadata.json, runs validate_docs_integrity, counts
//! diagnostics by severity, and formats a Markdown standing table.
//!
//! Does NOT compute q_config. Does NOT call load_admitted().

use std::path::Path;

pub struct StandingReport {
    pub version: String,
    pub q_docs: bool,
    pub q_json_schema_bridge: bool,
    pub doc_error_count: usize,
    pub doc_warning_count: usize,
    pub sch_json_error_count: usize,
    pub release_readiness: String,
}

impl StandingReport {
    pub fn to_markdown(&self) -> String {
        let pass = |b: bool| if b { "✅ PASS" } else { "❌ FAIL" };
        format!(
            "# star-toml Standing Report — v{}\n\n\
             | Predicate | Status |\n\
             |-----------|--------|\n\
             | q_docs | {} |\n\
             | q_json_schema_bridge | {} |\n\n\
             **DOC diagnostics**: {} errors, {} warnings\n\
             **SCH-JSON diagnostics**: {} errors\n\n\
             > {}\n\n\
             > LSP advisory only — does not grant q_config standing.",
            self.version,
            pass(self.q_docs),
            pass(self.q_json_schema_bridge),
            self.doc_error_count,
            self.doc_warning_count,
            self.sch_json_error_count,
            self.release_readiness,
        )
    }
}

/// Build a standing report from the workspace root.
pub fn build_report(root: &Path) -> StandingReport {
    let version = env!("CARGO_PKG_VERSION").to_owned();

    let metadata_path = root.join("docs/config_metadata.json");
    let (q_docs, doc_error_count, doc_warning_count) = match std::fs::read_to_string(&metadata_path)
        .ok()
        .and_then(|s| serde_json::from_str::<star_toml::ConfigDocumentationModel>(&s).ok())
    {
        Some(model) => {
            let diags = crate::diagnostics::project_doc_diagnostics(&model);
            let errors = diags
                .iter()
                .filter(|d| d.severity == Some(lsp_max::lsp_types_max::DiagnosticSeverity::ERROR))
                .count();
            let warnings = diags
                .iter()
                .filter(|d| d.severity == Some(lsp_max::lsp_types_max::DiagnosticSeverity::WARNING))
                .count();
            (errors == 0, errors, warnings)
        }
        None => (false, 0, 0),
    };

    // SCH-JSON standing: check docs/schema.json if present
    let (q_json_schema_bridge, sch_json_error_count) = {
        let schema_path = root.join("docs/schema.json");
        match std::fs::read_to_string(&schema_path).ok() {
            Some(text) if text.contains("\"$schema\"") => {
                let diags = crate::diagnostics::project_sch_json_diagnostics(&text);
                let errors = diags
                    .iter()
                    .filter(|d| {
                        d.severity == Some(lsp_max::lsp_types_max::DiagnosticSeverity::ERROR)
                    })
                    .count();
                (errors == 0, errors)
            }
            _ => (true, 0), // no schema.json = not applicable = no failure
        }
    };

    let total_errors = doc_error_count + sch_json_error_count;
    let release_readiness = if total_errors == 0 {
        "All checked predicates pass — ADMITTED pending runtime verification.".to_owned()
    } else {
        format!("{total_errors} error(s) would block admission. Fix before releasing.")
    };

    StandingReport {
        version,
        q_docs,
        q_json_schema_bridge,
        doc_error_count,
        doc_warning_count,
        sch_json_error_count,
        release_readiness,
    }
}
