# Incomplete handoff

## Done
- Read the user requirements from `plan_akv5t.md` and inspected the router prompt constraints.
- Confirmed the original Codex command works: `target/debug/malvin --model=codex:gpt-5.6 --do Hello` returned a response.
- Confirmed `malvin models codex:` lists live Codex catalog entries and family aliases resolve.
- Fixed the unresolved prompt placeholder in `default_prompts/router_code_extra.md`: `quality_gates.log` → `quality_gates_log`.
- Fixed Codex model discovery test pipe handling in `src/codex_sdk/discover.rs`:
  - Batch initialize/initialized/model-list JSON-RPC writes.
  - Treat `BrokenPipe` during the initial fixture write/flush as non-fatal so an already-available response can be read.
- Added test-mode teardown synchronization and immediate process-group signaling in:
  - `src/acp/unix_process_group_teardown.rs`
  - `src/acp/unix_process_group_teardown_poll.rs`
- Existing prompt changes in `default_prompts/router_a.md` and `default_prompts/router_b.md` remain in the working change set.
- Before the latest changes, these passed: `ruff check`, `kiss check`, Clippy, and `pytest tests` (156 passed).
- The router-A unresolved-brace tests passed after the placeholder correction.
- The Codex discovery test passed in isolation after the discovery fix.
- The process-group teardown test passed in isolation after the first synchronization fix.

## Remains
- The latest queued targeted command was skipped by the runtime handoff request. Run and verify:
  - `cargo test acp::unix_process_group_teardown::unix_process_group_teardown_tests::terminate_process_group_kills_sleep_child --lib -- --exact`
  - `cargo test acp::unix_process_group_teardown::unix_process_group_teardown_tests::terminate_agent_process_group_kills_sleep_child --lib -- --exact`
- Rerun the full required gates sequentially after the final edits:
  1. `ruff check`
  2. `kiss check`
  3. `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`
  4. `pytest tests`
  5. `./admin/malvin_rust_test_gate.sh`
- The last full Rust gate before the final teardown edits passed the first 803-test phase but failed the second phase on Codex `BrokenPipe` and two teardown tests. Those failures motivated the latest fixes; full confirmation remains outstanding.
- Inspect final `git diff --check` and `git status` after verification.

## Next-agent starting position
- Current commit is being created now and contains only the six intended tracked files:
  - `default_prompts/router_a.md`
  - `default_prompts/router_b.md`
  - `default_prompts/router_code_extra.md`
  - `src/acp/unix_process_group_teardown.rs`
  - `src/acp/unix_process_group_teardown_poll.rs`
  - `src/codex_sdk/discover.rs`
- Do not claim all requirements pass until the targeted tests and all five gates pass after this commit.
- If a gate fails, inspect the exact failure before changing scope. The prior plan’s two-file prompt scope was superseded by the user’s explicit request to satisfy all requirements, and the placeholder fix is required by the Rust tests.
