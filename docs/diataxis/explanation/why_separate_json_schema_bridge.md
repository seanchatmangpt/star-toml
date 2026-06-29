# Why Separate JSON Schema Bridge?

This article explains the design of the JSON Schema bridge and why it operates differently on native vs. WASM targets.

## The Problem: WebAssembly Restrictions

The native JSON Schema bridge utilizes crate dependencies that rely on blocking HTTP clients (`reqwest::blocking`) to resolve remote schema references. WebAssembly runtimes (specifically `wasm32-unknown-unknown`) run in single-threaded environments where blocking network operations are disallowed.

## The Solution: Split Architecture

To resolve this target mismatch:
- **Native target**: Executes full bridge checks and resolves remote references dynamically.
- **WASM target**: Bypasses bridge execution and returns a structured `WASM-BRIDGE-UNAVAILABLE` result.

This ensures WebAssembly compiles cleanly for web browsers and Node.js environments while native clients retain full validation capabilities.
