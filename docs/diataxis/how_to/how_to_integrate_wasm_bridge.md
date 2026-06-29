# How to Integrate the WASM Bridge

This guide explains how to compile and run star-toml config verification inside web browsers or Node.js runtimes using WASM.

## Goal

Compile the WASM build target and load the generated JS interface inside Node.js or browser hosts.

## Implementation Steps

### 1. Build WASM target

Run `wasm-pack` build specifying targets:

```bash
# For browsers
wasm-pack build --target web --out-dir pkg/web

# For Node.js
wasm-pack build --target nodejs --out-dir pkg/node
```

### 2. Import package inside Node.js

Import and use the validate functions:

```javascript
const { validate_config } = require('./pkg/node/star_toml_wasm');

const tomlString = `
  [system]
  log_path = "/var/log/app.log"
`;

const result = validate_config(tomlString);
if (result.status === "ADMITTED") {
    console.log("Configuration is admitted!");
} else {
    console.error("Diagnostics:", result.diagnostics);
}
```

### 3. Handle WASM restrictions

Note that functions requiring blocking HTTP checks return `WASM-BRIDGE-UNAVAILABLE` stub flags and must be delegated to local environment integrations.
