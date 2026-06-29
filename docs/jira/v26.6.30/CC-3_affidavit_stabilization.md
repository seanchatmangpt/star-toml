# CC-3: Affidavit Provenance Receipts (Stable Rust)

**Type**: Story  
**Project**: cargo-cicd  
**Status**: To Do  
**Priority**: BLOCKER  
**Effort**: 1 day  
**Prerequisite**: None  
**Dependency**: None (independent)  

---

## Objective

Migrate provenance receipt generation from nightly-only `affidavit` shell-out to stable Rust in-process BLAKE3 hashing. Current state: feature-gated, requires nightly + external `affi` binary. Target: stable Rust 1.86+ with built-in receipt generation using blake3 crate (already a dependency).

---

## Current State

### Problem

- `affidavit` feature is nightly-only (requires `affi` binary from nightly toolchain)
- Cannot ship default-enabled feature on stable Rust
- Blocks release to stable users (affects crates.io binary distributions)
- Location: `src/integrations/affidavit_shell.rs` (shell-out adapter)

### Solution

- Remove nightly dependency; use in-process BLAKE3 from `blake3` crate (already in Cargo.toml, unconditional)
- Generate BLAKE3-signed receipts directly in cargo-cicd
- Keep `affidavit` as optional feature (if users have nightly + affi binary available), but don't require it

---

## Acceptance Criteria

### AC1: BLAKE3 Receipt Generation (In-Process)

- [ ] `src/receipt_validation.rs` already has `blake3_file_hex(path)` — use it
- [ ] Create `fn generate_receipt(evidence_path: &Path, config_witness: &ConfigWitness) -> Result<ReceiptJson>`
- [ ] Receipt JSON structure:
  ```json
  {
    "version": "1.0",
    "generator": "cargo-cicd/26.6.30",
    "timestamp": "2026-06-29T...",
    "evidence_file_hash": "d74981ef...",
    "config_witness_hash": "c1d2e3f4...",
    "oracle_verdict": "accept|warn|fail",
    "signer": "automated"
  }
  ```
- [ ] Test: `test_receipt_generation_produces_valid_json`, `test_receipt_hash_deterministic`

### AC2: Remove affidavit Feature Dependency on Nightly

- [ ] Remove `nightly` requirement from `affidavit` feature in Cargo.toml
- [ ] Keep optional shell-out to `affi` binary if user has it installed (graceful fallback)
- [ ] Default feature set (no flags) must work on stable Rust 1.86+
- [ ] Test: `cargo build --stable --no-default-features 2>&1` passes

### AC3: Deprecate Affidavit Shell-out from Default

- [ ] Mark `src/integrations/affidavit_shell.rs` with `#[deprecated]` comment (will be removed in v26.6.31)
- [ ] Update Cargo.toml: `affidavit` feature now off by default
- [ ] Document: "Use BLAKE3 receipts by default. Enable `affidavit` feature for external cryptographic attestation."

### AC4: Receipt Validation Still Works

- [ ] `src/receipt_validation.rs::validate_receipt_file()` validates both in-process receipts and (if present) affidavit receipts
- [ ] Validation checks:
  1. Receipt file exists
  2. BLAKE3 hash matches (in-process receipts)
  3. Signature verifies (if affidavit receipt — optional)
  4. All required fields present (version, timestamp, oracle_verdict)
- [ ] Test: `test_validate_in_process_receipt`, `test_validate_affidavit_receipt_if_present`, `test_receipt_validation_rejects_tampering`

### AC5: Evidence Gate Uses BLAKE3 Receipts

- [ ] `src/nouns/gate.rs` calls `generate_receipt()` after evidence emission
- [ ] Receipt file saved to `target/cargo-cicd/evidence/<timestamp>.receipt.json`
- [ ] Gate report includes receipt path + hash
- [ ] Test: `test_gate_generates_blake3_receipt`, `test_gate_receipt_hash_in_report`

### AC6: CLI Does Not Require Nightly

- [ ] `cargo install cargo-cicd` on stable Rust succeeds
- [ ] `cargo cicd` binary runs without nightly toolchain installed
- [ ] Test (manual): `rustc --version` shows stable, `cargo install --path . --locked` succeeds

---

## Implementation Tasks

### 1. In-Process Receipt Generation (2 hours)

**File**: `src/receipt_generation.rs` (new)

```rust
pub fn generate_receipt(
    evidence_path: &Path,
    config_witness: &ConfigWitness,
    oracle_verdict: &str,
) -> Result<serde_json::Value> {
    let evidence_hash = blake3_file_hex(evidence_path)?;
    let receipt = json!({
        "version": "1.0",
        "generator": "cargo-cicd/26.6.30",
        "timestamp": jiff::Timestamp::now().to_rfc3339(),
        "evidence_file_hash": evidence_hash,
        "config_witness_hash": config_witness.hash(),
        "oracle_verdict": oracle_verdict,
        "signer": "automated"
    });
    Ok(receipt)
}
```

### 2. Update Cargo.toml (30 min)

**File**: `Cargo.toml`

```toml
[features]
# ... existing features ...
affidavit = ["process-data"]  # Remove "nightly" requirement; optional now

# blake3 already unconditional:
blake3 = { version = "1" }
jiff = { version = "0.2", features = ["serde"] }
```

### 3. Integrate Receipt Generation into Gate (1 hour)

**File**: `src/nouns/gate.rs`

After evidence emission:
```rust
let receipt = generate_receipt(
    &evidence_path,
    &config_witness,
    &oracle_verdict,
)?;
let receipt_path = evidence_dir.join(format!("{}.receipt.json", Timestamp::now()));
serde_json::to_writer(std::fs::File::create(&receipt_path)?, &receipt)?;

// Add to GateReport
gate_report.receipt_path = Some(receipt_path);
gate_report.receipt_hash = Some(receipt["evidence_file_hash"].as_str().unwrap().to_string());
```

### 4. Update Evidence Gate Tests (1 hour)

**File**: `tests/wasm4pm_evidence_gate.rs`

- [ ] Add test: `test_gate_generates_blake3_receipt`
- [ ] Add test: `test_gate_receipt_hash_deterministic`
- [ ] Add test: `test_gate_report_includes_receipt_path`

### 5. Deprecate Affidavit Shell-out (30 min)

**File**: `src/integrations/affidavit_shell.rs`

Add docstring:
```rust
/// DEPRECATED (v26.6.30): In-process BLAKE3 receipts replace affidavit shell-out.
/// This module will be removed in v26.6.31.
/// Enable `affidavit` feature only if you have the external `affi` binary installed.
```

---

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Remove nightly from `affidavit` feature; keep `blake3` unconditional |
| `src/receipt_generation.rs` | New: in-process BLAKE3 receipt generator |
| `src/nouns/gate.rs` | Call receipt generator after evidence emission |
| `src/integrations/affidavit_shell.rs` | Add deprecation notice |
| `tests/wasm4pm_evidence_gate.rs` | Add receipt generation tests |

---

## Testing Strategy

### Stable Rust Build

```bash
rustup default stable
cargo build --no-default-features 2>&1 | grep -i "error\|nightly"
# Expected: no errors, no nightly requirement
```

### Receipt Generation

```bash
cargo test receipt 2>&1 | grep "test.*ok"
# Expected: 5+ passing (hash, JSON structure, determinism, validation, gate integration)
```

### Manual Verification

```bash
cargo cicd gate 2>&1 | grep "receipt"
# Expected: receipt path and hash in gate output
cat target/cargo-cicd/evidence/*.receipt.json | jq .
# Expected: valid JSON with all required fields
```

---

## Rollback Plan

If BLAKE3 receipt generation causes issues:
1. Revert to affidavit shell-out (keep nightly feature for now)
2. Document as "experimental in v26.6.30, fallback in v26.6.31"
3. Defer removal to v26.6.32

---

## Verification Checklist

- [ ] In-process receipt generation working
- [ ] Receipt JSON valid and deterministic
- [ ] Gate report includes receipt path + hash
- [ ] Stable Rust build succeeds (no nightly requirement)
- [ ] Receipt validation passes both in-process and affidavit receipts
- [ ] Compiler warnings = 0
- [ ] Documentation updated: README, docs/, examples/

---

## References

- **Receipt Validation**: `src/receipt_validation.rs`
- **BLAKE3 Integration**: `src/receipt_validation.rs::blake3_file_hex()`
- **Gate Implementation**: `src/nouns/gate.rs`
- **Evidence Emission**: `src/evidence.rs`
