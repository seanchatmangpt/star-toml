//! ANDON gate surface — writes `.star-toml/lsp.gate` on every diagnostic publish.
//!
//! Gate file is EDITOR-TIME ONLY. It does not grant q_config standing.

use lsp_max::lsp_types_max::{Diagnostic, DiagnosticSeverity};
use std::path::{Path, PathBuf};

pub struct GateState {
    pub andon: bool,
    pub diagnostics: usize,
    pub errors: usize,
    pub warnings: usize,
}

impl GateState {
    pub fn from_diagnostics(diags: &[Diagnostic]) -> Self {
        let errors = diags
            .iter()
            .filter(|d| matches!(d.severity, Some(DiagnosticSeverity::ERROR)))
            .count();
        let warnings = diags
            .iter()
            .filter(|d| matches!(d.severity, Some(DiagnosticSeverity::WARNING)))
            .count();
        Self {
            andon: errors > 0,
            diagnostics: diags.len(),
            errors,
            warnings,
        }
    }
}

/// Write the gate file. `workspace_root` should be the project root directory.
/// Creates `.star-toml/` directory if it does not exist.
pub fn write_gate(workspace_root: &Path, state: &GateState) {
    let dir = workspace_root.join(".star-toml");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("lsp.gate");
    let content = format!(
        "ANDON={}\ndiagnostics={}\nerrors={}\nwarnings={}\n",
        if state.andon { 1 } else { 0 },
        state.diagnostics,
        state.errors,
        state.warnings,
    );
    let _ = std::fs::write(path, content);
}

/// Resolve the workspace root from a file URI path, walking upward until we
/// find `Cargo.toml` / `.git` / `.star-toml`, or give up at fs root.
pub fn resolve_workspace_root(file_path: &Path) -> PathBuf {
    let mut dir = if file_path.is_file() {
        file_path.parent().unwrap_or(file_path).to_owned()
    } else {
        file_path.to_owned()
    };
    loop {
        if dir.join("Cargo.toml").exists()
            || dir.join(".git").exists()
            || dir.join(".star-toml").exists()
        {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_owned(),
            None => return file_path.parent().unwrap_or(file_path).to_owned(),
        }
    }
}
