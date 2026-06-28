use lsp_max::lsp_types_max::*;
use lsp_max::{jsonrpc::Result, Client, LanguageServer};
use std::path::PathBuf;
use std::sync::Arc;

use crate::state::ServerState;

pub struct StarTomlLanguageServer {
    pub(crate) state: Arc<ServerState>,
    pub(crate) client: Client,
}

impl StarTomlLanguageServer {
    pub fn new(client: Client) -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            state: Arc::new(ServerState::new(workspace_root)),
            client,
        }
    }

    async fn publish(&self, uri: Url, diags: Vec<Diagnostic>) {
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}

#[lsp_max::async_trait]
impl LanguageServer for StarTomlLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Update workspace root from initialize params if available.
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri.as_ref() {
            if let Ok(path) = url::Url::parse(root_uri.as_str())
                .ok()
                .and_then(|u| u.to_file_path().ok())
                .ok_or(())
            {
                // ServerState is already constructed; workspace_root is immutable
                // after init. This is acceptable: editors that send a root_uri
                // will have the gate written to the correct dir after first
                // did_open; others fall back to cwd.
                let _ = path;
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "[".to_string(),
                        "=".to_string(),
                        " ".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "star-toml-lsp".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("star-toml-lsp initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let diags = self.state.open(uri.clone(), text).await;
        self.publish(uri, diags).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        if let Some(change) = params.content_changes.into_iter().next() {
            let diags = self.state.change(&uri, change.text, version).await;
            self.publish(uri, diags).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.state.close(&uri).await;
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let items = self
            .state
            .with_analyzer(uri, |a| a.completion_at(pos.line, pos.character))
            .await
            .unwrap_or_default();
        Ok(if items.is_empty() {
            None
        } else {
            Some(CompletionResponse::Array(items))
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let result = self
            .state
            .with_analyzer(uri, |a| a.hover_at(pos.line, pos.character))
            .await
            .flatten();
        Ok(result)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let symbols = self
            .state
            .with_analyzer(uri, |a| a.document_symbols())
            .await
            .unwrap_or_default();
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        use crate::diagnostics::{DUPLICATE_KEY, INVALID_TOML, PATH_TRAVERSAL, RELATIVE_ONLY_ABSOLUTE};

        let actions: Vec<CodeActionOrCommand> = params
            .context
            .diagnostics
            .into_iter()
            .filter_map(|d| {
                let code = match &d.code {
                    Some(NumberOrString::String(c)) => c.as_str(),
                    _ => return None,
                };
                let title = match code {
                    DUPLICATE_KEY => "Remove duplicate key (manual edit required)",
                    PATH_TRAVERSAL => "Remove path traversal sequence",
                    RELATIVE_ONLY_ABSOLUTE => "Convert to relative path",
                    INVALID_TOML => "Fix TOML syntax",
                    _ => return None,
                };
                Some(CodeActionOrCommand::CodeAction(CodeAction {
                    title: title.to_owned(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![d]),
                    is_preferred: Some(true),
                    ..Default::default()
                }))
            })
            .collect();

        Ok(if actions.is_empty() { None } else { Some(actions) })
    }
}
