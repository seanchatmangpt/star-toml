//! Autonomic Policy Engine for cargo/git workflow automation.
//!
//! # Acceptance Criteria Implementation Summary
//!
//! **AC1: TargetPressurePolicy** — Runs `cargo clean` when target/ exceeds size limit.
//! **AC2: ToolchainMismatchPolicy** — Runs `rustup update` when Rust toolchain version mismatches.
//! **AC3: TrybuildChangedPolicy** — Runs `cargo test --test changed` when trybuild files change.
//! **AC4: BranchBehindPolicy** — Safely merges origin/main when local branch is behind.
//! **AC5: PublishNotAdjudicatedPolicy** — Runs `cargo publish --dry-run` before publish.
//! **AC6: GitPhaseDirtyPolicy** — Runs `git add + git commit` when working tree is dirty.
//! **AC7: All policies respect --apply flag** — Implemented in PolicyEngine.
//! **AC8: Subprocess isolation & error handling (30s timeout)** — Implemented in subprocess.rs.
//! **AC9: Idempotency guarantees** — Each policy only executes when conditions are met.
//!
//! # Architecture
//!
//! - **policy_engine.rs** — Main executor loop orchestrating all 7 policies.
//! - **subprocess.rs** — Central subprocess runner with 30-second timeout.
//! - **policies/*** — Individual policy modules (6 AC policies + evidence_stale).

pub mod policies;
pub mod policy_engine;
pub mod subprocess;

pub use policy_engine::{PolicyEngine, PolicyEngineResult, PolicyResult};
pub use subprocess::run_with_timeout;
