use crate::analyzer::StarTomlDocumentAnalyzer;
use crate::gate::{write_gate, GateState};
use crate::telemetry::OcelEmitter;
use lsp_max::lsp_types_max::{Diagnostic, Url};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServerState {
    pub documents: Arc<Mutex<HashMap<Url, String>>>,
    pub analyzers: Arc<Mutex<HashMap<Url, StarTomlDocumentAnalyzer>>>,
    pub workspace_root: PathBuf,
    pub telemetry: OcelEmitter,
}

impl ServerState {
    pub fn new(workspace_root: PathBuf) -> Self {
        let telemetry = OcelEmitter::new(&workspace_root);
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            analyzers: Arc::new(Mutex::new(HashMap::new())),
            workspace_root,
            telemetry,
        }
    }

    pub async fn open(&self, uri: Url, content: String) -> Vec<Diagnostic> {
        let analyzer = StarTomlDocumentAnalyzer::new(uri.clone(), content.clone());
        let diags = analyzer.diagnostics();
        self.documents.lock().await.insert(uri.clone(), content);
        self.analyzers.lock().await.insert(uri.clone(), analyzer);

        self.telemetry.document_opened(uri.as_str());
        self.update_gate_and_telemetry(uri.as_str(), &diags);
        diags
    }

    pub async fn change(&self, uri: &Url, content: String, version: i32) -> Vec<Diagnostic> {
        self.documents.lock().await.insert(uri.clone(), content.clone());
        let diags = {
            let mut guard = self.analyzers.lock().await;
            if let Some(a) = guard.get_mut(uri) {
                a.update(content);
                a.diagnostics()
            } else {
                let a = StarTomlDocumentAnalyzer::new(uri.clone(), content);
                let d = a.diagnostics();
                guard.insert(uri.clone(), a);
                d
            }
        };
        self.telemetry.document_changed(uri.as_str(), version);
        self.update_gate_and_telemetry(uri.as_str(), &diags);
        diags
    }

    pub async fn close(&self, uri: &Url) {
        self.documents.lock().await.remove(uri);
        self.analyzers.lock().await.remove(uri);
        self.telemetry.document_closed(uri.as_str());
        self.telemetry.diagnostics_cleared(uri.as_str());
        // Write a zero gate on close (no open doc = no errors from this doc)
        write_gate(&self.workspace_root, &GateState::from_diagnostics(&[]));
    }

    pub async fn with_analyzer<F, R>(&self, uri: &Url, f: F) -> Option<R>
    where
        F: FnOnce(&StarTomlDocumentAnalyzer) -> R,
    {
        let guard = self.analyzers.lock().await;
        guard.get(uri).map(f)
    }

    fn update_gate_and_telemetry(&self, uri: &str, diags: &[Diagnostic]) {
        let state = GateState::from_diagnostics(diags);
        write_gate(&self.workspace_root, &state);
        if diags.is_empty() {
            self.telemetry.diagnostics_cleared(uri);
        } else {
            self.telemetry.diagnostics_raised(uri, state.errors, state.warnings);
        }
    }
}
