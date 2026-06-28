# Scope: E2E Testing Track

This scope document defines the milestones and test case design methodology for the E2E Testing Track of the `star-toml` project, incorporating the updated design requirements.

## Architecture
- Opaque-box testing of the `star-toml` library features.
- No direct dependency on the library's internal implementation details.
- Tests will be written as integration tests under `tests/` or in a standalone E2E test runner.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| E1 | E2E Test Infra | Define test runner, feature inventory, and write `TEST_INFRA.md` | None | DONE |
| E2 | Tier 1-4 Tests | Implement E2E test cases for Tiers 1-4, including typestate lifecycles, derive macro validation, trusted loader, custom checkers, safety tests, and publish `TEST_READY.md` | E1 | DONE |

## Interface Contracts
The E2E tests must exercise:
- Typestate lifecycle transition compilation and runtime restrictions.
- `star_toml::trusted()` config loader returning `TrustedConfig<T>`.
- Procedural `#[derive(Validate)]` macro and declarative `schema!` macro validation.
- Path and hostname safety guards (path traversal, Kelvin, host safety).
- Canonical saving (`save_canonical`, `save_file`, `save_pretty`).
- `ConfigLifecycle<T>` hooks (`normalize`, `validate_lifecycle`).
- Custom profile and policy validators.
