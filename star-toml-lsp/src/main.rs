// stdout is the LSP frame channel — forbid print!/println! so log text
// can never interleave ahead of the Content-Length header and corrupt framing.
#![deny(clippy::print_stdout)]

use star_toml_lsp::run_stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All logging goes to stderr — stdout carries the JSON-RPC protocol.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    run_stdio().await
}
