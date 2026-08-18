# Incomplete handoff — malvin models

Status: **incomplete**. Commit `8f265dfd` contains the live Pi provider-auth filtering implementation and the partially applied test-speed Candidate A; do not treat the commit as fully verified.

## Done

- Added live `pi --list-providers` lookup and provider/alias auth-env parsing.
- Preserved empty-auth local providers and unknown providers.
- Preserved fail-open behavior when the provider map is unavailable.
- Added parser, auth-filter, fake-Pi, and non-ASCII robustness tests.
- Ran the named gates before the latest test edits: `ruff check`, `kiss check`, Clippy, and `pytest tests` passed; the Rust gate previously passed 796/796 and 817/817 before the latest edits.
- Applied Candidate A partially: test-only access to the existing Cursor CLI fallback; fake-agent tests no longer invoke the full Cursor SDK dispatcher.
- Committed current tracked task files as `8f265dfd`.

## Remains

- Focused suite currently has one failing assertion in `src/cli/models_cmd_kiss_cov_tests.rs:147`. The direct fallback returns an error containing `` `.../agent models` exited with exit status: 1 ``; the assertion still expects `agent models failed`.
- The exact assertion correction was identified but not applied because the runtime requested handoff.
- After fixing that assertion, rerun `cargo test --lib models_cmd`.
- Rerun all required gates sequentially:
  - `ruff check`
  - `kiss check`
  - `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`
  - `pytest tests`
  - `./admin/malvin_rust_test_gate.sh`
- The last Rust gate was not green: it failed at `cursor_sdk::sdk_drain_idle_tests::never_run_done_idle_timeout_tears_down_and_retries` because the mock bridge timed out while spawning. Treat this as unresolved until a fresh complete gate passes.
- The previous Rust gate also showed model tests over the literal 1.5-second VISION limit. Candidate A is intended to reduce them, but this must be measured after the assertion fix.
- Update or separately verify stale handoff gate counts (`HANDOFF_pi_models_auth_filter.md` reports 795 + 815, while recent runs reported 796 + 817).

## Starting position

Begin at commit `8f265dfd`. Inspect `src/cli/models_cmd_kiss_cov_tests.rs` around line 147, replace the stale error substring assertion with one matching the direct fallback error, then run the focused model suite. Preserve unrelated dirty/untracked files. Do not claim completion from a partial Rust shard or from the earlier pre-edit gate.
