## 2026-06-27T22:09:40Z
<USER_REQUEST>
You are teamwork_preview_worker.
Your working directory is `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1`.
Your role is E2E Infra Designer.
Your task:
1. Analyze the star-toml codebase (src/lib.rs, README.md, PROJECT.md) to understand the requirements and API.
2. Design a comprehensive E2E test plan with 7 key features:
   - F1: Layered Loading (Defaults, Files, Env Prefix, Type Coercion)
   - F2: Env Var Expansion (`$VAR` and `${VAR}`)
   - F3: Path Resolution & Write-back (`ConfigFile::resolve`, `save_file`)
   - F4: Imperative Validation Structuring (Nesting, Array Indexing, Severity, DECLARE/Consistent)
   - F5: Built-in Validator Check Helpers (empty, range, one_of, semver, host, path, size)
   - F6: Declarative Schema Validation (`Schema` builder, Value/Str validation)
   - F7: Validation Analytics (Fitness, Variant ID, Section Grouping)
3. Draft and write the `TEST_INFRA.md` file at the project root (`/Users/sac/star-toml/TEST_INFRA.md`) following the template in the prompt:
   - Test Philosophy (Opaque-box, Category-Partition, BVA, Pairwise, Workload)
   - Feature Inventory (listing each of the 7 features and how they are checked across Tier 1, Tier 2, Tier 3)
   - Test Architecture (Rust integration test file `tests/e2e_tests.rs`, run with `cargo test --test e2e_tests`)
   - Real-World Application Scenarios (Tier 4 list, at least 5 scenarios)
   - Coverage Thresholds (Tier 1: >=35, Tier 2: >=35, Tier 3: >=7, Tier 4: >=5, total >= 82)
4. Verify that the project compiles by running `cargo check`. (Do NOT run cargo test or implement test cases yet, only publish TEST_INFRA.md and run cargo check).
5. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work.
6. Write a handoff report in `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1/handoff.md` and keep `/Users/sac/star-toml/.agents/sub_orch_e2e/worker_e1/progress.md` updated.
7. Send a message back to parent conversation ID `f0616e4f-e52f-4731-956b-25682da8e271` once you are done.

</USER_REQUEST>
