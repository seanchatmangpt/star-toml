//! AC1 & AC2: wpm binary discovery and shell-out integration.
//!
//! This module handles:
//! - AC1: wpm binary discovery (env var, PATH lookup, relative paths)
//! - AC2: Shell-out integration (wpm audit, lean, spc_status, receipt_doctor)
//!
//! The Wasm4pmShell struct manages discovery of the `wpm` binary and
//! provides methods to invoke wpm commands with proper error handling.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Result type for wasm4pm shell operations.
pub type Wasm4pmResult<T> = Result<T, Wasm4pmError>;

/// Errors that can occur during wpm binary discovery or invocation.
#[derive(Debug, Clone)]
pub enum Wasm4pmError {
    /// wpm binary not found in any location.
    NotFound(String),
    /// Command execution failed.
    CommandFailed(String),
    /// Invalid output format.
    InvalidOutput(String),
    /// Environment variable error.
    EnvError(String),
}

impl std::fmt::Display for Wasm4pmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wasm4pmError::NotFound(msg) => write!(f, "wpm binary not found: {}", msg),
            Wasm4pmError::CommandFailed(msg) => write!(f, "wpm command failed: {}", msg),
            Wasm4pmError::InvalidOutput(msg) => write!(f, "invalid wpm output: {}", msg),
            Wasm4pmError::EnvError(msg) => write!(f, "environment error: {}", msg),
        }
    }
}

impl std::error::Error for Wasm4pmError {}

/// wpm command type for shell invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wpm4Command {
    /// `wpm audit` — runs validation audit.
    Audit,
    /// `wpm lean` — lean mode check.
    Lean,
    /// `wpm spc_status` — spec status check.
    SpcStatus,
    /// `wpm receipt_doctor` — receipt validation.
    ReceiptDoctor,
}

impl Wpm4Command {
    /// Convert command to CLI argument string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Wpm4Command::Audit => "audit",
            Wpm4Command::Lean => "lean",
            Wpm4Command::SpcStatus => "spc_status",
            Wpm4Command::ReceiptDoctor => "receipt_doctor",
        }
    }
}

/// wpm shell integration handler.
///
/// Manages discovery of the `wpm` binary and provides safe invocation
/// of wpm subcommands with proper error handling.
#[derive(Debug, Clone)]
pub struct Wasm4pmShell {
    /// Path to the discovered wpm binary.
    binary_path: PathBuf,
}

impl Wasm4pmShell {
    /// Discover the wpm binary using environment variable, PATH lookup, or relative path.
    ///
    /// Discovery order (AC1):
    /// 1. WPM_BIN environment variable (if set)
    /// 2. Relative path: ./wpm, ./bin/wpm, ../wpm
    /// 3. PATH search for `wpm` executable
    ///
    /// # Errors
    ///
    /// Returns `Wasm4pmError::NotFound` if wpm cannot be located.
    pub fn discover() -> Wasm4pmResult<Self> {
        // Try env var first (AC1: env var)
        if let Ok(env_path) = std::env::var("WPM_BIN") {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                return Ok(Wasm4pmShell { binary_path: path });
            }
            return Err(Wasm4pmError::EnvError(format!(
                "WPM_BIN points to non-existent path: {}",
                env_path
            )));
        }

        // Try relative paths (AC1: relative paths)
        let relative_candidates = [
            PathBuf::from("wpm"),
            PathBuf::from("bin/wpm"),
            PathBuf::from("../wpm"),
        ];
        for candidate in relative_candidates {
            if candidate.exists() {
                return Ok(Wasm4pmShell { binary_path: candidate });
            }
        }

        // Try PATH search (AC1: PATH lookup)
        if let Ok(path_var) = std::env::var("PATH") {
            for path_dir in std::env::split_paths(&path_var) {
                let full_path = path_dir.join("wpm");
                if full_path.exists() {
                    return Ok(Wasm4pmShell { binary_path: full_path });
                }
            }
        }

        Err(Wasm4pmError::NotFound(
            "wpm not found in WPM_BIN env var, relative paths, or PATH".to_string(),
        ))
    }

    /// Discover from an explicit path instead of auto-discovery.
    pub fn from_path(path: impl AsRef<Path>) -> Wasm4pmResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Wasm4pmError::NotFound(format!(
                "wpm binary not found at {}",
                path.display()
            )));
        }
        Ok(Wasm4pmShell { binary_path: path.to_path_buf() })
    }

    /// Return the discovered binary path.
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// AC2: Execute a wpm command and return the output.
    ///
    /// Invokes the wpm binary with the given command and optional arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails or the command returns a non-zero exit code.
    pub fn run_command(
        &self,
        cmd: Wpm4Command,
        args: &[&str],
    ) -> Wasm4pmResult<String> {
        let output = Command::new(&self.binary_path)
            .arg(cmd.as_str())
            .args(args)
            .output()
            .map_err(|e| Wasm4pmError::CommandFailed(format!(
                "failed to execute wpm {}: {}",
                cmd.as_str(),
                e
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Wasm4pmError::CommandFailed(format!(
                "wpm {} exited with error: {}",
                cmd.as_str(),
                stderr
            )));
        }

        String::from_utf8(output.stdout).map_err(|_| {
            Wasm4pmError::InvalidOutput("wpm output is not valid UTF-8".to_string())
        })
    }

    /// AC2: Run `wpm audit` on the provided evidence data.
    /// Writes evidence to a temp file and passes the path to `wpm audit`.
    pub fn audit(&self, evidence_data: &str) -> Wasm4pmResult<String> {
        let temp_file = std::env::temp_dir().join("star_toml_evidence.json");
        std::fs::write(&temp_file, evidence_data)
            .map_err(|e| Wasm4pmError::CommandFailed(format!(
                "failed to write evidence file: {}",
                e
            )))?;

        self.run_command(Wpm4Command::Audit, &[temp_file.to_string_lossy().as_ref()])
    }

    /// AC2: Run `wpm lean` validation.
    pub fn lean(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::Lean, &[])
    }

    /// AC2: Run `wpm spc_status` check.
    pub fn spc_status(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::SpcStatus, &[])
    }

    /// AC2: Run `wpm receipt_doctor` check.
    pub fn receipt_doctor(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::ReceiptDoctor, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_command_as_str() {
        assert_eq!(Wpm4Command::Audit.as_str(), "audit");
        assert_eq!(Wpm4Command::Lean.as_str(), "lean");
        assert_eq!(Wpm4Command::SpcStatus.as_str(), "spc_status");
        assert_eq!(Wpm4Command::ReceiptDoctor.as_str(), "receipt_doctor");
    }

    #[test]
    fn test_wasm4pm_error_display() {
        let err = Wasm4pmError::NotFound("test".to_string());
        assert!(err.to_string().contains("wpm binary not found"));
    }
}
