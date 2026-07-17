//! Integration with external affidavit shell utilities (DEPRECATED).
//!
//! # Deprecation Notice
//!
//! Shell-based affidavit receipt generation is deprecated in favor of in-process
//! BLAKE3 receipt generation via [`crate::receipt_generation`].
//!
//! This module is maintained for backwards compatibility only. New code should
//! use [`crate::receipt_generation::generate_receipt`] and
//! [`crate::receipt_generation::validate_receipt`] instead.
//!
//! ## Why Deprecate Shell-Out?
//!
//! - **Reliability**: Shell invocation adds process overhead and failure modes
//! - **Portability**: Not all platforms have `affidavit` binary in PATH
//! - **Performance**: In-process BLAKE3 is faster and simpler
//! - **Stability**: Stable Rust 1.82+ includes all required dependencies
//!
//! AC3: Deprecate affidavit shell-out from default

/// Represents a result from invoking the external affidavit utility.
///
/// # Deprecation
///
/// **Deprecated**: Use [`crate::receipt_generation::generate_receipt`] instead.
#[deprecated(
    since = "26.6.30",
    note = "shell-based affidavit is deprecated; use receipt_generation::generate_receipt instead"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellAffidavitResult {
    /// The receipt returned from the shell utility
    pub receipt: String,
    /// Any diagnostic output from the shell invocation
    pub diagnostics: Option<String>,
}

/// Attempts to invoke an external affidavit binary to generate a receipt.
///
/// # Deprecation
///
/// **Deprecated in 26.6.30**: Use [`crate::receipt_generation::generate_receipt`] instead.
///
/// This function is provided only for backwards compatibility. It attempts to:
/// 1. Locate the `affidavit` binary in PATH
/// 2. Invoke it with configuration data
/// 3. Parse and return the result
///
/// If the binary is not found or invocation fails, this will return an error.
///
/// # Errors
///
/// Returns an error if:
/// - The `affidavit` binary cannot be found in PATH
/// - The binary fails to execute
/// - The binary returns a non-zero exit code
///
/// # Stability
///
/// This function is available on the `affidavit-shell` feature gate, which
/// is **disabled by default**. The feature is marked deprecated and will be
/// removed in a future release.
#[deprecated(
    since = "26.6.30",
    note = "shell-based affidavit is deprecated; use receipt_generation::generate_receipt instead"
)]
pub fn invoke_shell_affidavit(_config_bytes: &[u8]) -> Result<ShellAffidavitResult, String> {
    Err(
        "Shell-based affidavit support has been removed. Use receipt_generation::generate_receipt instead."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_shell_affidavit_returns_not_supported() {
        let result = invoke_shell_affidavit(b"test config");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("receipt_generation::generate_receipt"));
    }
}
