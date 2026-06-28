## 2026-06-27T22:18:57Z
You are teamwork_preview_worker for the star-toml project.
Your working directory is `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i4_save_lifecycle`.
Your task is to:
1. Implement canonical saving, pretty saving, and ConfigLifecycle hooks as detailed in `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i4_save_lifecycle/README.md`.
2. Add comprehensive unit tests verifying:
   - save_pretty, save_file, and save_canonical (verifying recursive key sorting alphabetically).
   - ConfigLifecycle normalization (e.g., modifying fields during normalization) and validate_lifecycle post-deserialization validation.
3. Run `cargo check` and `cargo test` in `/Users/sac/star-toml` to verify compilation and tests.
4. Write your handoff report in `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i4_save_lifecycle/handoff.md`.
5. Message your parent (conv ID: 5b7ec365-a7ba-4428-a41f-ba63f899910f) via send_message when done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
