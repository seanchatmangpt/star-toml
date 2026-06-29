//! In-process BLAKE3 receipt generation (CC-3, AC1).
//! Replaces nightly-only affidavit shell-out with stable Rust.

use std::path::Path;

/// JSON receipt structure for evidence provenance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReceiptJson {
    pub version: String,
    pub generator: String,
    pub evidence_file_hash: String,
    pub oracle_verdict: String,
    pub signer: String,
}

/// Generate a BLAKE3 receipt for an evidence file.
pub fn generate_receipt(evidence_path: &Path, oracle_verdict: &str) -> Result<ReceiptJson, String> {
    let content =
        std::fs::read(evidence_path).map_err(|e| format!("failed to read evidence file: {}", e))?;
    let hash = blake3::hash(&content);
    Ok(ReceiptJson {
        version: "1.0".to_string(),
        generator: "star-toml/26.6.30".to_string(),
        evidence_file_hash: hash.to_hex().to_string(),
        oracle_verdict: oracle_verdict.to_string(),
        signer: "automated".to_string(),
    })
}

/// Validate a receipt against the evidence file it covers.
pub fn validate_receipt(evidence_path: &Path, receipt: &ReceiptJson) -> Result<bool, String> {
    let content =
        std::fs::read(evidence_path).map_err(|e| format!("failed to read evidence file: {}", e))?;
    let hash = blake3::hash(&content);
    Ok(hash.to_hex().to_string() == receipt.evidence_file_hash)
}
