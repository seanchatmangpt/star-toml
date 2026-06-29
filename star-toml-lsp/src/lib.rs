// stdout is the LSP frame channel — diagnostics only, no q_config authority.
#![deny(clippy::print_stdout)]

pub mod analyzer;
pub mod commands;
pub mod diagnostics;
pub mod gate;
pub mod server;
pub mod standing;
pub mod state;
pub mod telemetry;

use lsp_max::{LspService, Server};
pub use server::StarTomlLanguageServer;
pub use state::ServerState;

/// Run the language server over stdio (the transport editors and Claude Code use).
pub async fn run_stdio() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(StarTomlLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await?;
    Ok(())
}
