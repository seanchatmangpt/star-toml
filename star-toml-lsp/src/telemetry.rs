//! Minimal OCEL-compatible telemetry hooks.
//!
//! Emits lifecycle events to `.star-toml/ocel.jsonl`.
//! This is LSP editor-time telemetry — it does NOT compute q_config.
//! OCEL = process history. Standing = ConfigWitness + failset_cardinality = 0.

use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    DocumentOpened,
    DocumentChanged,
    DocumentClosed,
    DiagnosticRaised,
    DiagnosticCleared,
}

impl EventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::DocumentOpened => "DocumentOpened",
            Self::DocumentChanged => "DocumentChanged",
            Self::DocumentClosed => "DocumentClosed",
            Self::DiagnosticRaised => "DiagnosticRaised",
            Self::DiagnosticCleared => "DiagnosticCleared",
        }
    }
}

#[derive(Clone)]
pub struct OcelEmitter {
    path: PathBuf,
    inner: Arc<Mutex<()>>,
}

impl OcelEmitter {
    pub fn new(workspace_root: &Path) -> Self {
        let dir = workspace_root.join(".star-toml");
        let _ = std::fs::create_dir_all(&dir);
        Self {
            path: dir.join("ocel.jsonl"),
            inner: Arc::new(Mutex::new(())),
        }
    }

    /// Emit one telemetry event. Non-fatal: silently drops on I/O error.
    pub fn emit(&self, kind: EventKind, uri: &str, attrs: Value) {
        let event = json!({
            "ocel:type": kind.as_str(),
            "ocel:uri": uri,
            "ocel:authority": "lsp_advisory_only",
            // Explicit boundary statement: the LSP is not a q_config authority.
            "ocel:note": "OCEL here is process/lifecycle history. q_config requires ConfigWitness + failset_cardinality=0.",
            "attrs": attrs,
        });
        let line = match serde_json::to_string(&event) {
            Ok(s) => s,
            Err(_) => return,
        };
        // Serialise writes with the mutex.
        let _guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(f, "{line}");
        }
    }

    pub fn document_opened(&self, uri: &str) {
        self.emit(EventKind::DocumentOpened, uri, json!({}));
    }

    pub fn document_changed(&self, uri: &str, version: i32) {
        self.emit(EventKind::DocumentChanged, uri, json!({ "version": version }));
    }

    pub fn document_closed(&self, uri: &str) {
        self.emit(EventKind::DocumentClosed, uri, json!({}));
    }

    pub fn diagnostics_raised(&self, uri: &str, error_count: usize, warning_count: usize) {
        self.emit(
            EventKind::DiagnosticRaised,
            uri,
            json!({ "errors": error_count, "warnings": warning_count }),
        );
    }

    pub fn diagnostics_cleared(&self, uri: &str) {
        self.emit(EventKind::DiagnosticCleared, uri, json!({}));
    }
}
