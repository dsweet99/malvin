# INCOMPLETE HANDOFF — SDK startup timeout and warning-test restoration

Status: incomplete; implementation committed, required final gates not completed.

## Done

- Restored the behavioral Cursor SDK warning regression test in `src/cursor_sdk/node_resolve.rs` (this file is now clean relative to HEAD): it imports `@cursor/sdk`, attempts `Agent.create`, and asserts stderr contains neither `ExperimentalWarning` nor `SQLite is an experimental`.
- Added `SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS = 1_000` and `sdk_bridge_startup_timeout()` in `src/sdk_drain_timeout.rs`.
- Changed bridge create/resume acknowledgement in `src/bridge_sdk/session_io.rs` to use the startup timeout rather than the short configured drain-idle timeout. Ordinary `run_done` draining remains on the configured idle path.
- Preserved the pre-existing tracked change in `src/cli/models_cmd_cursor.rs`, which wraps CLI model-list errors with `agent models failed:`.
- The first commit attempt was correctly blocked by the pre-commit clippy hook for `option_if_let_else` in the new timeout helper; that helper was rewritten with `map_or_else` before the final commit.
  - `cargo test --all-features sdk_drain_timeout -- --nocapture`: 3 passed.
  - `cargo nextest run --all-features -E 'test(never_run_done_idle_timeout_tears_down_and_retries) or test(keep_alive_events_do_not_trip_idle_drain_timeout) or test(quiet_node_cli_suppresses_sqlite_experimental_warning)'`: 3 passed.
- Earlier in this session, before the startup fix: `ruff check`, `kiss check`, clippy, and `pytest tests` passed; the full Rust gate failed twice under concurrent shard execution on bridge startup at 200 ms. Those earlier passes must be re-established against the final commit.

## What remains

- Run the required commands sequentially against the committed final tree:
  1. `ruff check`
  2. `kiss check`
  3. `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`
  4. `pytest tests`
  5. `./admin/malvin_rust_test_gate.sh`
- `cargo fmt --check` currently fails because of extensive pre-existing formatting differences in unrelated test files; do not run broad formatting without reviewing its unrelated changes. Formatting was not among the named required gates.
- Inspect the commit and working tree after gates. Do not commit or remove unrelated untracked files.

## Starting position

Current repository: `/home/dsweet/Projects/malvin`. The implementation changes are committed in the immediately preceding commit. The working tree still contains unrelated untracked files and should not be cleaned. The next agent must verify `git show --stat --oneline HEAD`, then run the five required gates sequentially. Treat the full Rust gate as unsatisfied until it passes completely; prior partial shard output is not sufficient.
