# JIRA Board for v26.6.30 (Operational Physics Substrate Hardening)

## Release Vision

**v26.6.30** completes the operational physics substrate across three interconnected crates:
- **cargo-cicd** v26.6.22 → **26.6.30**: Full wasm4pm oracle integration, autonomic policy execution, stable provenance receipts
- **star-toml** v26.6.29 → **26.6.30**: Documentation sync, assertive examples (dfcm_*), README correction
- **chicago-tdd-tools** v26.6.29 → **26.6.30**: Doctest examples, micro-library stabilization

**Release Thesis**: Finish the admission pipeline hardening (star-toml stable), wire the oracle gate (cargo-cicd), and establish behavior-driven testing patterns (chicago-tdd).

---

## JIRA Board Roadmap (v26.6.30)

### Blockers (Must Ship)

| Ticket ID | Type | Project | Title | Est. Effort | Dependency |
|---|---|---|---|---|---|
| **[CC-1](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-1_wasm4pm_oracle_integration.md)** | Epic | cargo-cicd | wasm4pm Oracle Full Integration | 3–5d | None (critical path) |
| **[CC-2](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-2_autonomic_policy_apply.md)** | Epic | cargo-cicd | Autonomic Policy --apply Full Execution | 2d | CC-1 (wpm oracle gate testing) |
| **[CC-3](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-3_affidavit_stabilization.md)** | Story | cargo-cicd | Affidavit Provenance Receipts (Stable Rust) | 1d | None |
| **[ST-1](file:///Users/sac/star-toml/docs/jira/v26.6.30/ST-1_readme_docs_sync.md)** | Story | star-toml | README & Docs Sync Post-v26.6.29 | 0.5d | None |
| **[ST-2](file:///Users/sac/star-toml/docs/jira/v26.6.30/ST-2_examples_assertive.md)** | Story | star-toml | Examples Assertive Validation (dfcm_*) | 0.5d | None |
| **[CC-4](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-4_lsp_witness_validation.md)** | Story | cargo-cicd | LSP Evidence Witness Integration Validation | 0.5d | CC-1 (oracle ready) |
| **[REL-3](file:///Users/sac/star-toml/docs/jira/v26.6.30/REL-3_transitive_dep_audit.md)** | Story | meta | Transitive Dependency Audit | 0.5d | None |

### Nice-to-Have (Can Defer to v26.6.31)

| Ticket ID | Type | Project | Title | Est. Effort |
|---|---|---|---|---|
| **[CC-5](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-5_parallel_tests.md)** | Epic | cargo-cicd | Parallel Test Execution (--jobs) | 5+d |
| **[CC-6](file:///Users/sac/star-toml/docs/jira/v26.6.30/CC-6_structured_logging.md)** | Story | cargo-cicd | Structured Logging (tracing) | 1–2d |
| **[CTT-1](file:///Users/sac/star-toml/docs/jira/v26.6.30/CTT-1_doctest_examples.md)** | Story | chicago-tdd-tools | Doctest Examples Documentation | 0.5d |

### Meta / Coordination

| Ticket ID | Type | Project | Title | Est. Effort |
|---|---|---|---|---|
| **[REL-1](file:///Users/sac/star-toml/docs/jira/v26.6.30/REL-1_release_planning.md)** | Epic | meta | v26.6.30 Release Planning & Coordination | — |
| **[REL-2](file:///Users/sac/star-toml/docs/jira/v26.6.30/REL-2_docs_and_release_notes.md)** | Story | meta | Release Notes, Migration Guides, Architecture Docs | 1–2d |
| **[REL-4](file:///Users/sac/star-toml/docs/jira/v26.6.30/REL-4_integration_testing.md)** | Story | meta | Integration Testing & Validation | 1d |

---

## Ship Criteria (Definition of Done)

### All Blockers Resolved

- [ ] CC-1: wasm4pm FILE_EXCHANGE protocol integrated, OCEL DFG discovery working, oracle verdict gate tests passing
- [ ] CC-2: All 7 autonomic policies execute safely (evidence_stale, target_pressure, toolchain_mismatch, trybuild_changed, branch_behind, publish_not_adjudicated, git_phase_dirty)
- [ ] CC-3: affidavit feature removed from default; BLAKE3 receipts work on stable Rust 1.86+
- [ ] ST-1: README pipeline diagram accurate, usage examples correct, deprecations documented
- [ ] ST-2: dfcm_validation_matrix, dfcm_error_variants, dfcm_merge_semantics all pass, no panics
- [ ] CC-4: LSP evidence witness messages flow correctly through protocol, no dropped fields
- [ ] REL-3: All transitive deps pinned, wasm4pm-compat patch verified, no duplicate versions

### Test Suite Health

- [ ] cargo-cicd: 200 behavior tests, 0 failures, 0 compiler warnings
- [ ] star-toml: 230 tests, 0 failures, 0 warnings
- [ ] chicago-tdd-tools: 45 tests, 0 failures, 0 warnings
- [ ] All examples run without modification

### Documentation

- [ ] `V26_6_30_RELEASE_NOTES.md` — what's new, breaking changes, migration guide
- [ ] `ARCHITECTURE.md` (each crate) — design decisions, invariants, integration points
- [ ] `GAPS_AND_ROADMAP.md` — what's deferred to v26.6.31+, why, effort
- [ ] `INTER_CRATE_DEPENDENCIES.md` — version pins, optional features, tested combinations

### No Compiler Warnings

```bash
cargo check --workspace 2>&1 | grep "^warning" | wc -l
# Must output: 0
```

### All Integration Points Verified

- [ ] cargo-cicd loads cicd.toml via star-toml TrustedLoader + witness binding
- [ ] chicago-tdd-tools aaa_test! macro used in cargo-cicd examples
- [ ] wasm4pm shell-out integration validates OCEL JSON via star-toml schema
- [ ] LSP server (cargo-cicd-lsp) emits evidence events in OCEL format

---

## Effort Estimate

**Critical path**: 2–3 weeks (1 engineer, 40h/week)

| Phase | Tickets | Effort | Duration |
|-------|---------|--------|----------|
| **1. wasm4pm oracle gate** | CC-1 | 3–5d | Week 1 |
| **2. autonomic execution** | CC-2, CC-4 | 2d | Week 1.5 |
| **3. affidavit stability** | CC-3 | 1d | Week 1.5 |
| **4. star-toml docs** | ST-1, ST-2 | 1d | Week 2 |
| **5. Release prep** | REL-1, REL-2, REL-3, REL-4 | 2–3d | Week 2–3 |
| **6. Integration testing** | All | 1d | Week 3 |
| **7. Release tag** | — | — | Week 3 |

**Total**: 8–10 days of focused work → Ship by **2026-07-15** (assuming start 2026-06-29).

---

## Document Deliverables (Must Commit Before Tag)

- [ ] `docs/V26_6_30_RELEASE_NOTES.md` — version history, features, breaking changes, thanks
- [ ] `docs/ARCHITECTURE.md` (each crate) — design, invariants, integration, trade-offs
- [ ] `docs/GAPS_AND_ROADMAP.md` — what's deferred to v26.6.31, why, effort estimates
- [ ] `docs/INTER_CRATE_DEPENDENCIES.md` — version pins, feature flags, tested combinations
- [ ] Updated `README.md` (each crate) — point to v26.6.30 release, known limitations
- [ ] `CHANGELOG.md` entries for cargo-cicd, star-toml, chicago-tdd-tools

---

## Traceability: Gaps to Tickets

| Gap | Severity | Ticket | Blocker? |
|-----|----------|--------|----------|
| wasm4pm oracle full integration (FILE_EXCHANGE + OCEL DFG) | CRITICAL | CC-1 | YES |
| autonomic --apply execute all 7 policies (only evidence_stale works) | CRITICAL | CC-2 | YES |
| affidavit provenance receipts (nightly-only, blocks stable) | CRITICAL | CC-3 | YES |
| README/docs sync (pipeline diagram, usage example, deprecations) | CRITICAL | ST-1 | YES |
| Examples assertive (dfcm_* validation) | CRITICAL | ST-2 | YES |
| LSP evidence witness integration validated | CRITICAL | CC-4 | YES |
| Transitive dep audit (wasm4pm-compat pinning) | CRITICAL | REL-3 | YES |
| Parallel test execution (--jobs N) | NICE-TO-HAVE | CC-5 | NO |
| Structured logging (tracing integration) | NICE-TO-HAVE | CC-6 | NO |
| chicago-tdd doctest examples | NICE-TO-HAVE | CTT-1 | NO |

---

## Inter-Crate Dependencies

```
cargo-cicd (26.6.22)
  ├─ star-toml (26.6.29) ← ST-1, ST-2 must pass first
  ├─ chicago-tdd-tools (26.6.29) [dev]
  └─ cargo-cicd-core (26.6.22) [path]
      └─ star-toml (26.6.29)

cargo-cicd-lsp (26.6.22)
  ├─ cargo-cicd (26.6.22) [path]
  └─ star-toml (26.6.29) [indirect]

wasm4pm-compat [patch] (git)
  └─ external oracle capability scan (stabilization pending)

Critical path: CC-1 (wasm4pm ready) ← CC-2 (autonomic gate tests) ← CC-4 (LSP witness)
```

---

## Sign-Off Checklist

- [ ] All blockers resolved (CC-1, CC-2, CC-3, ST-1, ST-2, CC-4, REL-3)
- [ ] Test suite 100% pass, 0 warnings
- [ ] All examples run without modification
- [ ] Documentation deliverables committed
- [ ] Integration tests passing (REL-4)
- [ ] No dependency conflicts or duplicate versions
- [ ] Release notes spell-checked, links verified
- [ ] Tag created: `v26.6.30` with automated announcement

---

## Release Notes Structure

```markdown
# v26.6.30 Release: Operational Physics Substrate Hardening

## Overview
Completes admission pipeline (star-toml), wasm4pm oracle integration (cargo-cicd), 
and behavior-driven testing patterns (chicago-tdd-tools).

## New Features
- wasm4pm oracle full integration (FILE_EXCHANGE, OCEL DFG)
- Autonomic policy --apply execution (all 7 policies)
- BLAKE3 receipt validation on stable Rust
- star-toml assertive DfCM examples (dfcm_validation_matrix, dfcm_error_variants, dfcm_merge_semantics)

## Breaking Changes
- None (backward compatible)

## Deprecated
- load_admitted_strict() (use load_admitted() — strict by default)
- wasm4pm shell-out for oracle check (use FILE_EXCHANGE protocol)

## Known Limitations
- Parallel test execution (--jobs) deferred to v26.6.31
- Structured logging (tracing) deferred to v26.6.31

## Effort & Timeline
- **Target ship date**: 2026-07-15
- **Critical path**: 8–10 days
- **Team**: 1 engineer

## Thanks
[Contributors, sponsors, upstream maintainers]
```
