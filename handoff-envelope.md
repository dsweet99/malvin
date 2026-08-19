# Incomplete handoff envelope

## Done
- Implemented the requested Codex backend path through the local `codex app-server` JSONL protocol.
- Fixed app-server initialization by sending required `clientInfo` metadata.
- Fixed the current sandbox enum spelling to `workspace-write`.
- Added live Codex model discovery through `model/list`; `malvin models codex:` currently reports the installed catalog.
- Verified `malvin --model=codex:gpt-5.6 --do Hello` exits successfully against the installed Codex CLI.
- `ruff check` passed.
- `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo` passed.
- `pytest tests` passed: 156 tests.
- `./admin/malvin_rust_test_gate.sh` passed both nextest runs: 797 + 818 tests, all passed.

## Remaining / unresolved
- `kiss check` failed on the repository’s 90% per-file Rust coverage gate. It reported `src/codex_sdk/discover.rs` at 75%, `src/codex_sdk/session_io.rs` at 20%, `src/codex_sdk/session_spawn.rs` at 17%, and pre-existing `src/pi_sdk/discover.rs` at 86%.
- Direct tests are still needed for Codex discovery and app-server protocol branches: model-list errors/malformed responses, initialize/thread request handling, prompt deltas/completion/failure, cancellation, and cleanup.
- `cargo fmt --check` reports broad pre-existing formatting drift in unrelated test files; changed Codex files were rustfmt’d, but the repository-wide named check was not green.
- The user-facing `report-codex.md` and the implementation authority choice still need reconciliation: the implementation uses the documented app-server route, while the report may recommend the SDK.
- The exact live Codex model catalog is environment-dependent; model discovery should be tested with a fake app-server or a controlled binary rather than relying only on the installed account.
- No completion sentinel is permitted while `kiss check` remains failing.

## Next-agent starting position
- Current implementation files: `src/codex_sdk/discover.rs`, `src/codex_sdk/mod.rs`, `src/codex_sdk/session_io.rs`, `src/codex_sdk/session_spawn.rs`.
- Current tracked integration changes include `src/cli/models_cmd.rs`, `src/cli/shared_opts.rs`, and the existing Codex model/backend wiring files shown by `git status`.
- Start by inspecting the committed diff and `kiss check` coverage output. Add focused tests using a fake Codex app-server protocol, then rerun `kiss check` directly.
- Do not treat the successful Rust test gate as evidence that the coverage gate is satisfied.
