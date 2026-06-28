# star-toml-lsp

Language Server Protocol (LSP) server for `*.toml` files validated by `star-toml`.

This server is designed to run asynchronously over standard I/O (stdio) to provide diagnostics, completions, hovers, document symbols, and formatting directly inside your editor.

## 🚀 Quick Start

Build the binary:
```bash
cargo build --release --bin star-toml-lsp
```

Run the server over stdio:
```bash
target/release/star-toml-lsp
```

## 📖 Detailed Documentation

For architectural design, list of features, and editor integration guides (Helix, Neovim, VSCode), please refer to the main documentation:

👉 **[Star TOML LSP Documentation](../docs/LSP.md)**
