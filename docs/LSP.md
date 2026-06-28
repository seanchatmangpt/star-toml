# star-toml Language Server Protocol (LSP)

The `star-toml-lsp` language server provides rich IDE features for `*.toml` files validated by the `star-toml` config admission engine.

---

## 🏛️ Architectural Separation of Concerns

`star-toml` enforces a strict separation between the runtime admission engine and the editor tooling:
1. **Clean Dependency Boundary**: The core `star-toml` library depends on `wasm4pm-compat` for OCEL lifecycle types and process-evidence compatibility, but has **zero dependencies** on LSP crates (`lsp-types`, `jsonrpc-core`, `tower-lsp`, etc.). The dependency law is: `star-toml → wasm4pm-compat`; `star-toml-lsp → star-toml`; `wasm4pm → star-toml`; `star-toml ↛ wasm4pm`; `star-toml-lsp ↛ tower-lsp`.
2. **Unidirectional Dependency**: The `star-toml-lsp` language server depends on `star-toml` to run validation checks and parse schemas, but the core engine remains completely unaware of the LSP.
3. **Async / Sync Separation**:
   * **Runtime**: Runs synchronously and deterministically.
   * **LSP**: Runs asynchronously as a long-running JSON-RPC daemon over standard IO.

---

## 🚀 Key IDE Features

### 1. Real-time Diagnostics
Diagnostics are continuously updated on `didOpen`, `didChange`, and `didClose` text document sync notifications.
* **Precise Error Spans**: maps path locations (e.g., `server.port`) directly to source locations, highlighting the specific offending key.
* **Repair Hints**: Diagnostic warnings automatically include repair hints derived from the validation constraints.
* **FNV-1a Variant Clustering**: Emits stable diagnostic codes and variant fingerprints.

### 2. Auto-completions
* **Structure & Key Suggestions**: Offers auto-complete templates for tables, arrays, and standard settings.
* **Trigger Characters**: Triggered by default on `[`, `=`, and ` ` (space).

### 3. Hover Cards
Hovering over a key displays its expected schema type, description, and validation bounds (e.g., numeric ranges, enum lists, and path policies).

### 4. Document Symbols
Lists all defined tables, nested fields, and arrays of tables in the editor's outline view (supporting quick navigation).

### 5. Document Formatting (Deferred)
* Enforces alphabetically-sorted canonical TOML formatting (deferred to a future release).

### 6. Code Actions (Quick-Fixes)
* Provides quick-fixes for common syntax errors, such as removing duplicate key-value definitions (suggests manual remediation).

### 7. Custom LSP Extension APIs (Deferred)
* Custom validator extension hooks are deferred to a future release.

---

## 🔧 Building and Running

Compile the language server binary:
```bash
cargo build --bin star-toml-lsp -p star-toml-lsp
```

Run the language server via standard I/O:
```bash
cargo run --bin star-toml-lsp -p star-toml-lsp
```

---

## 🔌 Editor Configuration Examples

### VS Code (via `LSP Client` extension)
Add the following command to your client binary path configuration:
```json
{
  "star-toml.lsp.path": "star-toml-lsp"
}
```

### Neovim (`nvim-lspconfig`)
Add custom server configuration:
```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.star_toml_lsp then
  configs.star_toml_lsp = {
    default_config = {
      cmd = { 'star-toml-lsp' },
      filetypes = { 'toml' },
      root_dir = lspconfig.util.root_pattern('Cargo.toml', '.git'),
      settings = {},
    },
  }
end

lspconfig.star_toml_lsp.setup{}
```

### Helix (`languages.toml`)
Define the language server in your local Helix configuration:
```toml
[[language]]
name = "toml"
language-servers = [ "star-toml-lsp" ]

[language-server.star-toml-lsp]
command = "star-toml-lsp"
```
