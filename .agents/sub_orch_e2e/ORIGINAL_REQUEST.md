# Original User Request

## 2026-06-27T22:08:46Z

You are the E2E Testing Orchestrator (sub-orchestrator) for the star-toml project.
Your working directory is `/Users/sac/star-toml/.agents/sub_orch_e2e`.
Your parent is orchestrator (conv ID: 8b7a1e43-e812-4100-baa8-e9069b46b3b0).
Your task is to plan and manage the E2E Testing Track as described in `/Users/sac/star-toml/.agents/sub_orch_e2e/SCOPE.md`.
Specifically:
1. Design the E2E test infrastructure and features. Publish `TEST_INFRA.md`.
2. Implement E2E test cases for Tiers 1-4.
3. Once all Tier 1-4 tests are ready and passing, publish `TEST_READY.md` at the project root (`/Users/sac/star-toml`).
4. Keep `/Users/sac/star-toml/.agents/sub_orch_e2e/progress.md` and `/Users/sac/star-toml/.agents/sub_orch_e2e/SCOPE.md` updated.
5. Do NOT write code directly. Delegate tasks to specialized workers/specialists under `.agents/` (creating directories like `.agents/sub_orch_e2e/worker_...`).

## 2026-06-27T22:10:44Z

Context: Refined project scope and requirements.
Content: The user has refined the requirements for star-toml v26.6.27. I have updated the global PROJECT.md and the track SCOPE.md files.
Please review the new scope at `/Users/sac/star-toml/.agents/sub_orch_e2e/SCOPE.md`. It includes tests for:
1. Typestate lifecycles: Config<Raw>, Config<Merged>, Config<Deserialized<T>>, Config<Validated<T>>, Config<Frozen<T>>, and saving checks.
2. Trusted config loader yielding TrustedConfig<T>.
3. Procedural macro #[derive(Validate)] and declarative schema! macro.
4. Separate save functions (save_file, save_canonical, save_pretty).
5. ConfigLifecycle<T> hooks (normalize, validate_lifecycle).
6. Profile and policy validation helper methods.
7. Safety tests for traversal, null bytes, and DNS hostnames.
Action: Please adapt your planning and test suite design to target these new requirements.
