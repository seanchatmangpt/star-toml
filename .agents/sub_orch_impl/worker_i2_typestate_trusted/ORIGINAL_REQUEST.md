## 2026-06-27T22:12:16Z
You are teamwork_preview_worker for the star-toml project.
Your working directory is `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i2_typestate_trusted`.
Your task is to:
1. Implement the Config<S> typestate lifecycles and trusted() builder as detailed in `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i2_typestate_trusted/README.md`.
2. Add comprehensive unit tests verifying:
   - Config<Raw> -> Config<Merged> -> Config<Deserialized<T>> -> Config<Validated<T>> -> Config<Frozen<T>> transitions.
   - trusted() builder loading, env overrides, validation reporting (fitness, checks run/passed, error list), and ConfigDigest calculation.
3. Run `cargo check` and `cargo test` in `/Users/sac/star-toml` to verify compilation and tests.
4. Write your handoff report in `/Users/sac/star-toml/.agents/sub_orch_impl/worker_i2_typestate_trusted/handoff.md`.
5. Message your parent (conv ID: 5b7ec365-a7ba-4428-a41f-ba63f899910f) via send_message when done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
