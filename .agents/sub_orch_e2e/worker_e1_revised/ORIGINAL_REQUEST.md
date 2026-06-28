## 2026-06-27T22:10:55Z
You are teamwork_preview_worker.
Your working directory is `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1_revised`.
Your role is E2E Infra Revised Designer.
Your task:
1. Analyze the updated star-toml requirements in `/Users/sac/star-toml/.agents/sub_orch_e2e/SCOPE.md` and `/Users/sac/star-toml/PROJECT.md`.
2. Revise the E2E test plan for the 7 new/refined features:
   - F1: Typestate Lifecycle Abstraction (`Config<Raw>` -> `Config<Merged>` -> `Config<Deserialized<T>>` -> `Config<Validated<T>>` -> `Config<Frozen<T>>` transitions, compile-time and runtime checks).
   - F2: Layered Loading & Env Overrides (Defaults, Files, Env prefix mapping, Type coercion, Env var expansion `$VAR`/`${VAR}`).
   - F3: Validation Interfaces & Macros (`#[derive(Validate)]` procedural macro, declarative `schema!` macro, `Validate` trait, custom profile/policy validators).
   - F4: Built-in Safety & Domain Checkers (Path traversal guards, null bytes rejection, Kelvin/host safety, semver check, range, size format).
   - F5: Save Functions & Serialization (`save_file`, `save_canonical`, `save_pretty`, and `ConfigFile::resolve`).
   - F6: Lifecycle Hooks (`ConfigLifecycle<T>` trait: `normalize`, `validate_lifecycle`).
   - F7: Trusted Loader & Analytics (`star_toml::trusted()` yielding `TrustedConfig<T>`, conformance fitness, variant fingerprint, section grouping).
3. Draft and overwrite the `TEST_INFRA.md` at the project root (`/Users/sac/star-toml/TEST_INFRA.md`) following the template in the prompt:
   - Test Philosophy (Opaque-box, Category-Partition, BVA, Pairwise, Workload)
   - Feature Inventory (mapping Tiers 1-3 cases for the 7 features)
   - Test Architecture (Rust integration test file `tests/e2e_tests.rs`, run with `cargo test --test e2e_tests`)
   - Real-World Application Scenarios (Tier 4 list, at least 5 scenarios incorporating the new requirements)
   - Coverage Thresholds (Tier 1: >=35, Tier 2: >=35, Tier 3: >=7, Tier 4: >=5, total >= 82)
4. Verify that the project compiles with `cargo check` (do not run tests or implement them yet).
5. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work.
6. Write a handoff report in `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1_revised/handoff.md` and keep `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1_revised/progress.md` updated.
7. Send a message back to parent conversation ID `f0616e4f-e52f-4731-956b-25682da8e271` once you are done.
