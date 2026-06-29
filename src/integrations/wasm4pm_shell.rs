//! Shell integration for the `wpm` (wasm4pm) binary.
//!
//! Provides discovery, command execution, and verdict inference for the
//! external `wpm` oracle binary.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Errors that can occur when using the wpm oracle shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wasm4pmError {
    /// The `wpm` binary could not be found.
    NotFound,
    /// The command exited with a non-zero status.
    CommandFailed(String),
    /// The output could not be interpreted.
    InvalidOutput(String),
    /// An environment variable was set but pointed to a bad path.
    EnvError(String),
}

impl std::fmt::Display for Wasm4pmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "wpm binary not found"),
            Self::CommandFailed(msg) => write!(f, "wpm command failed: {msg}"),
            Self::InvalidOutput(msg) => write!(f, "invalid wpm output: {msg}"),
            Self::EnvError(msg) => write!(f, "wpm env error: {msg}"),
        }
    }
}

impl std::error::Error for Wasm4pmError {}

/// Result type for wasm4pm shell operations.
pub type Wasm4pmResult<T> = Result<T, Wasm4pmError>;

/// Commands supported by the `wpm` oracle binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wpm4Command {
    Audit,
    Lean,
    SpcStatus,
    ReceiptDoctor,
}

/// Verdict returned by the `wpm` oracle after running a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WpmVerdict {
    /// All checks passed.
    Pass,
    /// Warnings present but not blocking.
    Warn,
    /// One or more failures found.
    Fail,
    /// Only partial data available; result is inconclusive.
    Partial,
    /// The oracle is not available; result is unknown.
    NotAvailable,
}

/// Shell wrapper around the `wpm` oracle binary.
#[derive(Debug, Clone)]
pub struct Wasm4pmShell {
    binary_path: PathBuf,
}

impl Wasm4pmShell {
    /// Auto-discover the `wpm` binary.
    ///
    /// Discovery order:
    /// 1. `WPM_BIN` env var — if set, path must exist.
    /// 2. `WPM_PATH` env var — secondary explicit path.
    /// 3. `WPM_SEARCH_PATHS` — colon-separated list of directories.
    /// 4. Relative paths: `./wpm`, `./bin/wpm`, `../wpm`.
    /// 5. `PATH` environment variable.
    pub fn discover() -> Wasm4pmResult<Self> {
        // 1. WPM_BIN
        if let Ok(val) = std::env::var("WPM_BIN") {
            let p = PathBuf::from(&val);
            if p.exists() {
                return Ok(Self { binary_path: p });
            }
            return Err(Wasm4pmError::EnvError(format!("WPM_BIN={val} does not exist")));
        }

        // 2. WPM_PATH
        if let Ok(val) = std::env::var("WPM_PATH") {
            let p = PathBuf::from(&val);
            if p.exists() {
                return Ok(Self { binary_path: p });
            }
        }

        // 3. WPM_SEARCH_PATHS
        if let Ok(val) = std::env::var("WPM_SEARCH_PATHS") {
            for dir in val.split(':') {
                let p = PathBuf::from(dir).join("wpm");
                if p.exists() {
                    return Ok(Self { binary_path: p });
                }
            }
        }

        // 4. Relative paths
        for rel in &["./wpm", "./bin/wpm", "../wpm"] {
            let p = PathBuf::from(rel);
            if p.exists() {
                return Ok(Self { binary_path: p });
            }
        }

        // 5. PATH
        if let Some(path_val) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_val) {
                let p = dir.join("wpm");
                if p.exists() {
                    return Ok(Self { binary_path: p });
                }
            }
        }

        Err(Wasm4pmError::NotFound)
    }

    /// Create a shell wrapper from an explicit path.
    pub fn from_path(path: impl Into<PathBuf>) -> Wasm4pmResult<Self> {
        let p = path.into();
        if p.exists() {
            Ok(Self { binary_path: p })
        } else {
            Err(Wasm4pmError::NotFound)
        }
    }

    /// Return the path to the `wpm` binary.
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// Run a `wpm` command with extra arguments.
    pub fn run_command(&self, cmd: Wpm4Command, args: &[&str]) -> Wasm4pmResult<String> {
        let cmd_str = match cmd {
            Wpm4Command::Audit => "audit",
            Wpm4Command::Lean => "lean",
            Wpm4Command::SpcStatus => "spc_status",
            Wpm4Command::ReceiptDoctor => "receipt_doctor",
        };

        let output = Command::new(&self.binary_path)
            .arg(cmd_str)
            .args(args)
            .output()
            .map_err(|e| Wasm4pmError::CommandFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(Wasm4pmError::CommandFailed(stdout))
        }
    }

    /// Run `wpm audit` on evidence data (writes to a temp file).
    pub fn audit(&self, evidence_data: &str) -> Wasm4pmResult<String> {
        use std::io::Write;
        let tmp_path = std::env::temp_dir().join(format!(
            "star_toml_wpm_audit_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        let mut f =
            std::fs::File::create(&tmp_path).map_err(|e| Wasm4pmError::EnvError(e.to_string()))?;
        f.write_all(evidence_data.as_bytes()).map_err(|e| Wasm4pmError::EnvError(e.to_string()))?;
        drop(f);
        let path = tmp_path.to_string_lossy().into_owned();
        let result = self.run_command(Wpm4Command::Audit, &[&path]);
        let _ = std::fs::remove_file(&tmp_path);
        result
    }

    /// Run `wpm lean`.
    pub fn lean(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::Lean, &[])
    }

    /// Run `wpm spc_status`.
    pub fn spc_status(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::SpcStatus, &[])
    }

    /// Run `wpm receipt_doctor`.
    pub fn receipt_doctor(&self) -> Wasm4pmResult<String> {
        self.run_command(Wpm4Command::ReceiptDoctor, &[])
    }

    /// Infer a `WpmVerdict` from command output and exit status.
    pub fn infer_verdict(output: &str, exit_success: bool) -> WpmVerdict {
        if !exit_success {
            return WpmVerdict::Fail;
        }
        let lower = output.to_lowercase();
        if lower.contains("fail") || lower.contains("error") {
            WpmVerdict::Fail
        } else if lower.contains("warn") || lower.contains("drift") {
            WpmVerdict::Warn
        } else if lower.contains("pass") || lower.contains("ok") || output.trim().is_empty() {
            WpmVerdict::Pass
        } else {
            WpmVerdict::Warn
        }
    }

    /// Run `wpm audit` and return a verdict.
    pub fn audit_with_verdict(&self, evidence_data: &str) -> WpmVerdict {
        match self.audit(evidence_data) {
            Ok(output) => Self::infer_verdict(&output, true),
            Err(Wasm4pmError::NotFound) => WpmVerdict::NotAvailable,
            Err(_) => WpmVerdict::Fail,
        }
    }

    /// Run `wpm lean` and return a verdict.
    pub fn lean_with_verdict(&self) -> WpmVerdict {
        match self.lean() {
            Ok(output) => Self::infer_verdict(&output, true),
            Err(Wasm4pmError::NotFound) => WpmVerdict::NotAvailable,
            Err(_) => WpmVerdict::Fail,
        }
    }

    /// Run `wpm spc_status` and return a verdict.
    pub fn spc_status_with_verdict(&self) -> WpmVerdict {
        match self.spc_status() {
            Ok(output) => Self::infer_verdict(&output, true),
            Err(Wasm4pmError::NotFound) => WpmVerdict::NotAvailable,
            Err(_) => WpmVerdict::Fail,
        }
    }

    /// Run `wpm receipt_doctor` and return a verdict.
    pub fn receipt_doctor_with_verdict(&self) -> WpmVerdict {
        match self.receipt_doctor() {
            Ok(output) => Self::infer_verdict(&output, true),
            Err(Wasm4pmError::NotFound) => WpmVerdict::NotAvailable,
            Err(_) => WpmVerdict::Fail,
        }
    }
}
