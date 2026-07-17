//! External integrations for star-toml.

pub mod wasm4pm_shell;

pub use wasm4pm_shell::{
    Wasm4pmError, Wasm4pmResult, Wasm4pmShell, Wpm4Command, WpmVerdict,
};
