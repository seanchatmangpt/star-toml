//! Central subprocess runner with 30-second timeout and error handling.
//!
//! AC8: Subprocess isolation & error handling (30s timeout)

use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus, Output},
    time::Duration,
};

/// Runs a subprocess with a 30-second timeout.
///
/// Uses `mpsc::recv_timeout` so the calling thread blocks at most 30 seconds;
/// the spawned command thread races against the deadline.
///
/// # Arguments
/// * `program` - The program to execute
/// * `args` - Arguments to pass to the program
/// * `dry_run` - If true, logs the command but doesn't execute it
///
/// # Returns
/// `Ok(output)` if the command succeeds within the timeout.
/// `Err(string)` if the command fails or times out.
pub fn run_with_timeout(program: &str, args: &[&str], dry_run: bool) -> Result<Output, String> {
    if dry_run {
        let cmd = format!("{} {}", program, args.join(" "));
        eprintln!("[DRY-RUN] Would execute: {}", cmd);
        return Ok(Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: format!("[DRY-RUN] {}", cmd).into_bytes(),
            stderr: Vec::new(),
        });
    }

    let program_str = program.to_string();
    let program_clone = program_str.clone();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut cmd = Command::new(&program_str);
        for arg in &args_vec {
            cmd.arg(arg);
        }
        let result = cmd.output();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok(result) => match result {
            Ok(output) => Ok(output),
            Err(e) => Err(format!("Failed to run '{}': {}", program_clone, e)),
        },
        Err(_) => Err(format!("Subprocess '{}' timed out after 30 seconds", program_clone)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run_echo() {
        let result = run_with_timeout("echo", &["hello"], true);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("echo"));
    }

    #[test]
    fn test_subprocess_does_not_panic() {
        // Ensure subprocess calls don't panic, even if the command fails
        let result = run_with_timeout("nonexistent_cmd_xyz", &[], false);
        // We expect an error, but we just want to ensure no panic
        let _ = result;
    }

    #[test]
    fn test_nonexistent_command() {
        let result = run_with_timeout("nonexistent_command_xyz", &[], false);
        assert!(result.is_err());
    }
}
