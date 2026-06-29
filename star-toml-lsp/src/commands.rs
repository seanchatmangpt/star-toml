//! LSP command execution for star-toml ANDON surface.
//!
//! Commands spawn cargo subprocesses and return output as strings.
//! None of these functions grant q_config standing or call load_admitted().

use std::{path::Path, process::Command};

/// Run a cargo subcommand from the workspace root, return (success, stdout+stderr).
fn run_cargo(args: &[&str], root: &Path) -> (bool, String) {
    match Command::new("cargo").args(args).current_dir(root).output() {
        Ok(out) => {
            let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
            combined.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), combined)
        }
        Err(e) => (false, format!("Failed to spawn cargo: {e}")),
    }
}

/// Execute a named star-toml command.
/// Returns `(success, output)`.
pub fn execute(cmd: &str, root: &Path) -> (bool, String) {
    match cmd {
        "starToml.runDocsExport" => {
            run_cargo(&["run", "--bin", "star_toml_docs", "--", "export"], root)
        }
        "starToml.runDocsVerify" => {
            run_cargo(&["run", "--bin", "star_toml_docs", "--", "verify"], root)
        }
        "starToml.runReleaseVerifier" => run_cargo(&["run", "--bin", "release_verifier"], root),
        "starToml.showStandingReport" => {
            let report = crate::standing::build_report(root);
            (true, report.to_markdown())
        }
        "starToml.refreshWorkspaceIndex" => (true, "Workspace index refreshed.".to_string()),
        _ => (false, format!("Unknown command: {cmd}")),
    }
}

/// Registered command names, declared in `initialize` capabilities.
pub const COMMANDS: &[&str] = &[
    "starToml.runDocsExport",
    "starToml.runDocsVerify",
    "starToml.runReleaseVerifier",
    "starToml.showStandingReport",
    "starToml.refreshWorkspaceIndex",
];
