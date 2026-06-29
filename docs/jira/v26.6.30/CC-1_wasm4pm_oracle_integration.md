# CC-1: wasm4pm Oracle Full Integration

**Type**: Epic  
**Project**: cargo-cicd  
**Status**: To Do  
**Priority**: BLOCKER  
**Effort**: 3–5 days  
**Prerequisite**: None  
**Dependency**: Unblocks CC-2, CC-4  

---

## Objective

Integrate the wasm4pm oracle capability scanning and decision-making into the cargo-cicd gate pipeline. Current state: stub implementation in `src/integrations/wasm4pm_current.rs` with 75 lines of non-functional code. Target: full FILE_EXCHANGE protocol + OCEL 2.0 JSON validation + wpm doctor shell-out gate integration.

---

## Current State

- **Capability scan**: 22/75 capabilities ready (2026-06-02 audit)
- **Path selected**: D (SHELL_OUT via confirmed working commands)
- **Status**: Decision documented; implementation deferred pending API stability
- **Blocker document**: `/Users/sac/cargo-cicd/docs/deferred/WASM4PM_CONTRIB_EXTRACTION.md`

### Confirmed Working Commands (Shell-out)

```bash
wpm audit <input.ocel.json>          # OCEL 2.0 conformance audit
wpm receipt doctor <file>            # Receipt forensic audit
wpm lean                             # Lean Six Sigma waste audit
wpm spc status                       # Statistical Process Control
wpm doctor                           # System health check
wpm telco status                     # Telco routing status
wpm autoprocess                      # AutoProcess pipeline
```

### Blockers (Do Not Use)

- `wpm oracle check` — confirmed stub; AndonPull not implemented
- `wpm mining conformance` — stubs model loading to DFG::new()
- `wpm doctor` — reports FAIL for Cargo.toml/src/ when run outside wasm4pm tree (non-blocking)

---

## Acceptance Criteria

### AC1: wpm Binary Discovery

- [ ] `discover_wpm_binary()` resolves via env var (WPM_BIN, WPM_PATH, WPM_SEARCH_PATHS), relative paths (.bin/wpm, bin/wpm), or PATH lookup
- [ ] Returns `None` gracefully if wpm not found (gate verdict = PARTIAL)
- [ ] Test: `test_wpm_discovery_env_var`, `test_wpm_discovery_path_lookup`, `test_wpm_discovery_missing`

### AC2: Shell-out Integration

- [ ] `Wasm4pmShell::audit(path)` calls `wpm audit` on evidence file
- [ ] `Wasm4pmShell::lean()` calls `wpm lean`
- [ ] `Wasm4pmShell::spc_status()` calls `wpm spc status`
- [ ] `Wasm4pmShell::receipt_doctor(path)` calls `wpm receipt doctor`
- [ ] All shell commands capture stdout/stderr and infer verdict: Pass / Warn / Fail / Partial / NotAvailable
- [ ] Test: `test_wpm_audit_ocel_json`, `test_wpm_lean_waste_detection`, `test_wpm_spc_trending`, `test_wpm_receipt_forensics`

### AC3: Evidence Gate Integration

- [ ] `src/nouns/gate.rs` calls wpm oracle AFTER evidence emission
- [ ] Integrates `Wasm4pmShell::audit()` result into `GateReport`
- [ ] Evidence file must exist before oracle call; gate verdict = PARTIAL if file missing
- [ ] Test: `test_gate_calls_wpm_audit`, `test_gate_partial_when_wpm_unavailable`, `test_gate_report_includes_oracle_result`

### AC4: OCEL JSON Schema Validation

- [ ] Before passing to wpm, validate emitted evidence OCEL against `wasm4pm-compat` OCEL 2.0 schema
- [ ] Validation reports: version (2.0), object-centric events (omap), required attributes
- [ ] Test: `test_ocel_emitted_by_evidence_passes_schema`, `test_ocel_version_field_is_2_0`, `test_ocel_event_attributes_present`

### AC5: Error Handling & Diagnostics

- [ ] If wpm fails with exit code != 0: verdict = Fail, capture stderr
- [ ] If wpm output parsing fails: verdict = Warn, log parse error
- [ ] If wpm not found: verdict = Partial, documented as graceful degradation
- [ ] Test: `test_wpm_exit_code_1_returns_fail`, `test_wpm_malformed_output_returns_warn`, `test_wpm_not_found_returns_partial`

### AC6: Deterministic Ordering

- [ ] Gate verdict ordering: ADMIT → SUGGEST → WARN → FAIL (wpm verdict flows into final gate decision)
- [ ] If wpm returns Warn, gate escalates to WARN unless already FAIL
- [ ] Test: `test_gate_verdict_ordering_with_wpm`, `test_wpm_warn_escalates_gate_to_warn`

---

## Implementation Tasks

### 1. Finalize Wasm4pmShell API (Day 1)

**File**: `src/integrations/wasm4pm_shell.rs`

- [ ] Extend `Wasm4pmShell` struct with bidirectional command execution (audit, lean, spc_status, doctor, etc.)
- [ ] Implement `invoke()` private method that spawns `Command::new(&self.binary)` with args
- [ ] Implement `infer_verdict()` parsing stdout/stderr to map to `WpmVerdict` enum
- [ ] Tests: All 7 confirmed commands each have pass/warn/fail case coverage

### 2. OCEL Schema Validation (Day 1–2)

**File**: `src/evidence.rs` + new `src/ocel_validation.rs`

- [ ] Before emitting evidence file, deserialize to `toml::Value` and validate against OCEL 2.0 schema
- [ ] Use `serde_json` to convert OCEL JSON to schema-validatable form
- [ ] Schema validation checks:
  - Version field = "2.0"
  - At least one event in omap
  - Each event has required attributes (concept:name, time:timestamp)
  - Trace structure (objects, relationships optional)
- [ ] Return `ValidationError` if schema violation detected
- [ ] Tests: `test_ocel_schema_validation_passes`, `test_ocel_missing_version_fails`, `test_ocel_malformed_event_fails`

### 3. Gate Integration (Day 2)

**File**: `src/nouns/gate.rs`

- [ ] After `build_receipt()`, call `Wasm4pmShell::detect()` to check for wpm availability
- [ ] If wpm available, call `wpm.audit(evidence_path)` and capture result
- [ ] Add `oracle_verdict: Option<WpmVerdict>` field to `GateReport`
- [ ] Integrate oracle verdict into final gate decision:
  - If oracle Fail → gate Fail
  - If oracle Warn → gate escalates to Warn (but not below current verdict)
  - If oracle Partial → gate continues without oracle input
- [ ] Update SARIF output to include oracle verdict as additional rule result
- [ ] Tests: `test_gate_with_wpm_pass`, `test_gate_with_wpm_fail`, `test_gate_without_wpm_returns_partial`

### 4. Diagnostics & Error Messages (Day 3)

**File**: `src/nouns/gate.rs` + `src/diagnostics/`

- [ ] Add new counterexample variant `OracleUnavailable` if wpm not found
- [ ] Add new counterexample variant `OracleAuditFailed` if wpm exits non-zero
- [ ] Emit clear diagnostics to user:
  - "oracle verdict: PASS (wpm audit successful)"
  - "oracle verdict: WARN (wpm detected drift in SPC trend)"
  - "oracle verdict: PARTIAL (wpm not found in PATH; skipping oracle gate)"
  - "oracle verdict: FAIL (wpm audit detected conformance violation)"
- [ ] Tests: All diagnostic messages appear in gate report output

### 5. Integration Testing (Day 4–5)

**File**: `tests/wasm4pm_evidence_gate.rs` (expand existing)

- [ ] **Mock wpm binary** (Python/shell script) that simulates real wpm behavior
  - Reads OCEL JSON, validates format
  - Returns exit 0 (pass), 1 (fail), or hang (timeout test)
  - Outputs verdict summary JSON
- [ ] **Environment-based mocking**: Set `WPM_BIN` to mock script for CI runs
- [ ] **End-to-end flow**:
  1. `cargo cicd test changed` → emit evidence
  2. `cargo cicd gate` → evidence exists → call wpm → integrate verdict
  3. Verify gate report includes oracle result
- [ ] **Stress tests**:
  - wpm exits with signal (SIGTERM)
  - wpm takes > 5 seconds (timeout behavior)
  - wpm outputs malformed JSON (error handling)

---

## Files to Modify

| File | Lines | Change |
|------|-------|--------|
| `src/integrations/wasm4pm_shell.rs` | 50–100 | Implement full Wasm4pmShell API |
| `src/evidence.rs` | +30 | Add OCEL schema validation before emission |
| `src/ocel.rs` | +40 | OCEL 2.0 schema definition + validator |
| `src/nouns/gate.rs` | +50 | Integrate oracle verdict into GateReport |
| `src/diagnostics/code.rs` | +2 | Add OracleUnavailable, OracleAuditFailed variants |
| `tests/wasm4pm_evidence_gate.rs` | +80 | Expand with full oracle integration tests |

---

## Testing Strategy

### Unit Tests (in-tree)

```bash
cargo test --test wasm4pm_evidence_gate 2>&1 | grep "test.*ok"
# Expected: 8+ passing (discovery, audit, lean, spc, error handling, determinism, diagnostics)
```

### Integration Tests (with mock wpm)

```bash
# Set WPM_BIN to mock script
export WPM_BIN=/tmp/mock_wpm.sh
cargo cicd gate 2>&1 | grep -i "oracle"
# Expected: oracle verdict appears in output
```

### Manual Validation (if wpm installed)

```bash
which wpm && cargo cicd gate 2>&1 | grep -i oracle || echo "wpm not in PATH"
```

---

## Rollback Plan

If wpm integration causes regressions:
1. Set `WPM_BIN=/dev/null` → oracle verdict = PARTIAL (graceful degradation)
2. Revert commits to `src/nouns/gate.rs` (oracle verdict logic)
3. Keep OCEL schema validation (safe, non-blocking)
4. Defer full integration to v26.6.31

---

## Verification Checklist

- [ ] All unit tests passing: `cargo test wasm4pm_evidence_gate`
- [ ] All integration tests passing: mock wpm + end-to-end flow
- [ ] Gate report includes oracle verdict field
- [ ] SARIF output includes oracle rule results
- [ ] Evidence OCEL validates against schema before oracle call
- [ ] Graceful fallback when wpm unavailable (verdict = PARTIAL)
- [ ] Compiler warnings = 0
- [ ] Documentation updated: examples/02_ocel_evidence.rs includes oracle step

---

## References

- **Capabilities Scan**: `/Users/sac/cargo-cicd/docs/deferred/WASM4PM_CONTRIB_EXTRACTION.md`
- **Confirmed Commands**: `src/integrations/wasm4pm_shell.rs` (lines 1–40, docstring)
- **Evidence Format**: `src/evidence.rs` (OCEL 2.0 emission)
- **Gate Architecture**: `src/nouns/gate.rs` (GateReport, verdict ordering)
