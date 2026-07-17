use star_toml::receipt_generation::{generate_receipt, validate_receipt};
use std::io::Write;

fn tmp_evidence() -> (tempfile::NamedTempFile, String) {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    let content = r#"{"ocel:version":"2.0","ocel:events":{}}"#;
    f.write_all(content.as_bytes()).unwrap();
    (f, content.to_string())
}

#[test]
fn test_receipt_generation_produces_valid_json() {
    let (f, _) = tmp_evidence();
    let receipt = generate_receipt(f.path(), "accept").unwrap();
    assert_eq!(receipt.version, "1.0");
    assert!(!receipt.evidence_file_hash.is_empty());
    assert_eq!(receipt.oracle_verdict, "accept");
}

#[test]
fn test_receipt_hash_deterministic() {
    let (f, _) = tmp_evidence();
    let r1 = generate_receipt(f.path(), "accept").unwrap();
    let r2 = generate_receipt(f.path(), "accept").unwrap();
    assert_eq!(r1.evidence_file_hash, r2.evidence_file_hash);
}

#[test]
fn test_validate_in_process_receipt() {
    let (f, _) = tmp_evidence();
    let receipt = generate_receipt(f.path(), "accept").unwrap();
    assert!(validate_receipt(f.path(), &receipt).unwrap());
}

#[test]
fn test_receipt_validation_rejects_tampering() {
    let (f, _) = tmp_evidence();
    let mut receipt = generate_receipt(f.path(), "accept").unwrap();
    receipt.evidence_file_hash = "deadbeef".to_string();
    assert!(!validate_receipt(f.path(), &receipt).unwrap());
}
