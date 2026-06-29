# CC-2: Autonomic Policy --apply Full Execution

**Type**: Epic  
**Project**: cargo-cicd  
**Status**: To Do  
**Priority**: BLOCKER  
**Effort**: 2 days  
**Prerequisite**: CC-1 (wasm4pm oracle ready for testing)  
**Dependency**: Unblocks CC-4  

---

## Objective

Complete execution of all 7 autonomic policies via `cargo cicd autonomic --apply`. Current state: only `evidence_stale` policy executes safely. Target: all 7 policies execute with proper idempotency guards, subprocess isolation, and rollback capability.

---

## Current State

### Working (evidence_stale)
- Detects stale evidence files (no OCEL update in N days)
- Spawns safe subprocess: `cargo cicd test changed`
- Idempotent (re-running gives same result)
- Tested: `tests/autonomic_policies.rs::test_evidence_stale_policy_*`

### Partial or Stubbed (6/7 policies)

| Policy | Status | Issue | Fix Required |
|--------|--------|-------|--------------|
| **TargetPressurePolicy** | Stubbed | Knows size but doesn't prune | Safe cargo clean subprocess + disk space guard |
| **ToolchainMismatchPolicy** | Stubbed | Detects mismatch but doesn't apply | Safe rustup update subprocess |
| **TrybuildChangedPolicy** | Stubbed | Detects but doesn't rerun | Safe cargo test --test changed subprocess |
| **BranchBehindPolicy** | Stubbed | Detects but doesn't merge | Safe git merge subprocess + rebase option |
| **PublishNotAdjudicatedPolicy** | Stubbed | Detects but doesn't publish | Safe cargo publish subprocess + dry-run first |
| **GitPhaseDirectyPolicy** | Stubbed | Detects but doesn't commit | Safe git add + commit subprocess + message template |

---

## Acceptance Criteria

### AC1: TargetPressurePolicy (Disk Space)

- [ ] Policy detects when `target/` exceeds configured `max_size_gb` (from cicd.toml)
- [ ] When `--apply` is set: run `cargo clean` in subprocess
- [ ] Guard: refuse to clean if disk usage has just recovered (idempotency guard)
- [ ] Verdict progression: WARN (approaching limit) → FAIL (over limit) → PASS (cleaned)
- [ ] Test: `test_target_pressure_policy_cleans_target_dir`, `test_target_pressure_idempotent`

### AC2: ToolchainMismatchPolicy

- [ ] Policy detects mismatch between rust-toolchain.toml and `cicd.toml` (toolchain field)
- [ ] When `--apply` is set: run `rustup update <channel>` subprocess
- [ ] Guard: refuse update if build is already running (lock detection)
- [ ] Verdict progression: WARN (mismatch detected) → PASS (updated)
- [ ] Test: `test_toolchain_mismatch_policy_updates_rustup`

### AC3: TrybuildChangedPolicy

- [ ] Policy detects when trybuild test files changed (e.g., `tests/ui/*.rs`)
- [ ] When `--apply` is set: run `cargo test --test changed` subprocess (reuses evidence_stale subprocess pattern)
- [ ] Guard: refuse rerun if test is still running from prior build (process check)
- [ ] Verdict progression: WARN (trybuild changed) → PASS (tests re-run)
- [ ] Test: `test_trybuild_changed_policy_reruns_tests`

### AC4: BranchBehindPolicy

- [ ] Policy detects when local branch is behind origin (N commits behind)
- [ ] When `--apply` is set: offer safe `git merge origin/main` (or `git rebase` if user prefers)
- [ ] Guard: refuse merge if working tree is dirty (safety check)
- [ ] Verdict progression: WARN (behind) → PASS (merged) OR FAIL (conflict)
- [ ] Test: `test_branch_behind_policy_merges_safely`, `test_branch_behind_policy_refuses_dirty_tree`

### AC5: PublishNotAdjudicatedPolicy

- [ ] Policy detects when Cargo.toml version is new and cargo publish hasn't run
- [ ] When `--apply` is set: run `cargo publish --dry-run` first, then `cargo publish`
- [ ] Guard: refuse publish if CI is not passing (check gate verdict first)
- [ ] Verdict progression: WARN (not published) → FAIL (publish error) OR PASS (published)
- [ ] Test: `test_publish_not_adjudicated_policy_publishes_with_dry_run`

### AC6: GitPhaseDirectyPolicy

- [ ] Policy detects when working tree is dirty (uncommitted changes)
- [ ] When `--apply` is set: run `git add` + `git commit` with templated message
- [ ] Guard: refuse commit if commit hooks fail (pre-commit validation)
- [ ] Verdict progression: WARN (dirty) → PASS (committed)
- [ ] Test: `test_git_phase_dirty_policy_commits_safely`

### AC7: All Policies Respect --apply Flag

- [ ] Each policy checks `PolicyMode::Suggest` vs `PolicyMode::Apply`
- [ ] `Suggest` mode: return verdict only (no execution)
- [ ] `Apply` mode: execute subprocess + return verdict
- [ ] Test: `test_all_policies_respect_apply_mode`, `test_all_policies_suggest_without_apply`

### AC8: Subprocess Isolation & Error Handling

- [ ] Each subprocess runs with captured stdout/stderr
- [ ] Subprocess exit code determines verdict (0 = success, non-zero = failure)
- [ ] If subprocess hangs > 30s: timeout and return FAIL (with timeout diagnostic)
- [ ] Subprocess inherits cicd.toml config (test.changed.base, workspace settings)
- [ ] Test: `test_subprocess_timeout_returns_fail`, `test_subprocess_exit_code_determines_verdict`

### AC9: Idempotency Guarantees

- [ ] Running `cargo cicd autonomic --apply` twice on same state → same outcome
- [ ] No partial writes (atomic file operations)
- [ ] No duplicate commits or published versions
- [ ] Test: `test_autonomic_apply_idempotent_across_runs`

---

## Implementation Tasks

### 1. Refactor Policy Executor (Day 1, Morning)

**File**: `src/autonomic/policy_engine.rs`

Current code (lines 103–130): Only evidence_stale calls subprocess

```rust
pub async fn execute(&mut self, mode: PolicyMode) -> Result<Vec<PolicyVerdict>> {
    let mut verdicts = vec![];
    
    match &self.config.test.changed.base {
        Some(base) => {
            // Only evidence_stale actually executes
            let verdict = self.policy_evidence_stale(mode).await?;
            verdicts.push(verdict);
        }
        None => {}
    }
    
    Ok(verdicts)
}
```

**Changes**:
- [ ] Create `PolicyExecutor` trait with `execute(&self, mode: PolicyMode) -> PolicyVerdict` (async)
- [ ] Implement trait for each policy (7 impls)
- [ ] Each executor has pre/post guards (lock checks, dirty tree checks, disk space checks)
- [ ] Centralized subprocess runner: `fn run_subprocess(cmd, args, timeout) -> Result<(exit_code, stdout, stderr)>`

### 2. Implement TargetPressurePolicy (Day 1, Afternoon)

**File**: `src/autonomic/policies/target_pressure.rs` (new)

- [ ] Read `cicd.toml` `target.max_size_gb` field
- [ ] Compute `target/` directory size via `walkdir` / `fs::metadata`
- [ ] If exceeded: suggest `cargo clean`
- [ ] If `--apply`: run `cargo clean`, re-check size, return verdict
- [ ] Guard: Don't clean if recently cleaned (e.g., last 1 hour) — idempotency

### 3. Implement ToolchainMismatchPolicy (Day 1, Afternoon)

**File**: `src/autonomic/policies/toolchain_mismatch.rs` (new)

- [ ] Read `rust-toolchain.toml` channel (e.g., "1.86")
- [ ] Read `cicd.toml` toolchain field (if present)
- [ ] Compare; suggest match if mismatch
- [ ] If `--apply`: run `rustup update <channel>` subprocess
- [ ] Guard: Check if cargo is running (via lock file in target/)

### 4. Implement TrybuildChangedPolicy (Day 1, Afternoon)

**File**: `src/autonomic/policies/trybuild_changed.rs` (new)

- [ ] Detect if trybuild test files changed (glob `tests/ui/*.rs`)
- [ ] If `--apply`: run `cargo test --test changed` via same subprocess pattern as evidence_stale
- [ ] Guard: Check if test process already running

### 5. Implement BranchBehindPolicy, PublishNotAdjudicatedPolicy, GitPhaseDirectyPolicy (Day 2)

**Files**: 
- `src/autonomic/policies/branch_behind.rs`
- `src/autonomic/policies/publish_not_adjudicated.rs`
- `src/autonomic/policies/git_phase_dirty.rs`

Each follows same pattern:
1. Detect condition (git log, version check, git status)
2. Suggest action
3. If `--apply`: run subprocess, capture result
4. Guards (dirty tree, CI passing, commit hooks)

### 6. Subprocess Isolation & Timeout (Day 2)

**File**: `src/autonomic/subprocess.rs` (new)

```rust
pub async fn run_subprocess(
    cmd: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<(i32, String, String)> {
    // Spawn command with 30s timeout
    // Capture stdout, stderr
    // Return (exit_code, stdout, stderr)
}
```

### 7. Integration Testing (Day 2)

**File**: `tests/autonomic_policies.rs` (expand from 31 → 50+ tests)

- [ ] Mock each subprocess (echo exit code, capture args)
- [ ] Test each policy in Suggest mode (no execution)
- [ ] Test each policy in Apply mode (execution)
- [ ] Test guards (refuse dirty tree, refuse timeout, refuse lock)
- [ ] Test idempotency (run twice, same state)

---

## Files to Modify

| File | Change |
|------|--------|
| `src/autonomic/policy_engine.rs` | Refactor executor; loop over all 7 policies |
| `src/autonomic/policies/` | Create 6 new policy modules (target_pressure, toolchain_mismatch, trybuild_changed, branch_behind, publish_not_adjudicated, git_phase_dirty) |
| `src/autonomic/subprocess.rs` | New: central subprocess runner with timeout |
| `tests/autonomic_policies.rs` | Expand from 31 → 50+ tests, cover all policies in both modes |

---

## Testing Strategy

```bash
# Unit tests (mocked subprocesses)
cargo test autonomic 2>&1 | grep "test.*ok"
# Expected: 50+ passing

# Integration (with real cargo)
cargo cicd autonomic --suggest 2>&1 | grep "WARN\|FAIL\|PASS"
cargo cicd autonomic --apply 2>&1 | grep "executed\|verdict"
```

---

## Rollback Plan

If a policy causes regressions:
1. Mark policy as `#[deprecated]` in code + docs
2. Set default mode to `--suggest` (non-executing)
3. Revert policy executor code
4. Defer problematic policy to v26.6.31

---

## Verification Checklist

- [ ] All 7 policies have unit tests (Suggest + Apply modes)
- [ ] All 7 policies have integration tests (mock subprocess)
- [ ] All 7 policies respect idempotency guarantee
- [ ] All guards working (dirty tree, lock, timeout)
- [ ] Subprocess timeout = 30s enforced
- [ ] Exit codes properly mapped to verdicts
- [ ] Compiler warnings = 0
- [ ] Examples updated: `examples/03_max_pipeline.rs` demonstrates `--apply` mode

---

## References

- **Policy Engine**: `src/autonomic/policy_engine.rs`
- **Existing Policy** (evidence_stale): `src/autonomic/policies/evidence_stale.rs`
- **Current Tests**: `tests/autonomic_policies.rs` (31 tests)
- **cicd.toml Schema**: `src/cicd_toml.rs` (test.changed.base, target.max_size_gb, etc.)
