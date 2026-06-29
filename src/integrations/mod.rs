//! wasm4pm Oracle integration module.
//!
//! Provides AC1-AC6 implementation for wasm4pm shell integration, OCEL validation,
//! evidence gate integration, and oracle verdict handling.

pub mod wasm4pm_shell;

pub use wasm4pm_shell::{Wasm4pmError, Wasm4pmResult, Wasm4pmShell, Wpm4Command};
