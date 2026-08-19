# Incomplete handoff envelope

## Done
- Re-read the primary request: `Please integrate codex: as a backend, analogously to cursor: or pi:`.
- Re-audited the prior report and found its report-only scope conflicts with the explicit implementation request; implementation is therefore the active requirement.
- Restored the focused Codex changes after reverting accidental workspace-wide formatting changes.
- Added Codex model/backend selection, `codex:` model parsing, CLI model listing, bridge dispatch, and an app-server JSONL session implementation under `src/codex_sdk/`.
- `cargo check -q` passed after the focused changes.
- Focused model and models-command tests passed: 6 + 22 tests.
- Full library suite passed: 1512 passed, 3 ignored.

## Remaining / unresolved
- The Codex implementation is not done: `src/codex_sdk/` is untracked and has little/no direct coverage.
- The existing implementation uses the documented app-server path, while `report-codex.md` recommends the TypeScript SDK. Resolve this explicit artifact conflict: either update the report to select app-server with evidence, or implement the selected SDK act instead.
- Add tests for protocol encoding/decoding, initialize/initialized ordering, thread creation and ID capture, prompt deltas/completion/failure, cancellation, malformed input, and cleanup. The previous commit-hook result measured `session_io.rs` at 20% and `session_spawn.rs` at 17%, below the 90% per-file gate.
- Verify actual Codex wire response shapes, especially completion message extraction and turn/thread correlation; current code falls back to concatenated deltas and does not correlate turn IDs robustly.
- Add Codex to user-facing help/docs and model-filter tests as required by the repository’s behavioral contracts.
- Run `cargo fmt --check`, `cargo check`, full tests, and the repository’s documented quality gate command. Do not treat the prior passing library suite as sufficient; the named quality gate previously failed on coverage and untracked source files.
- Decide what to do with untracked planning/reference artifacts; `git diff` does not include them.

## Next-agent starting position
- Focused tracked modifications currently exist in:
  `src/model_id.rs`, `src/agent_backend/factory.rs`,
  `src/agent_backend/sdk_client.rs`, `src/agent_backend/sdk_client_prompt.rs`,
  `src/agent_backend/sdk_client_session.rs`, `src/bridge_sdk/session.rs`,
  `src/cli/models_cmd.rs`, and `src/lib.rs`.
- Untracked implementation files are `src/codex_sdk/mod.rs`, `src/codex_sdk/session_io.rs`, and `src/codex_sdk/session_spawn.rs`.
- Read `codex-app-server.md`, `codex-sdk.md`, `report-codex.md`, and `.malvin_plan.md` before making the authority decision.
- Start with `git diff` for the focused files, then add protocol tests before changing more production behavior.
- Do not emit the completion sentinel: required runtime tests, coverage gate, report/implementation authority reconciliation, and final gate evidence are unresolved.
