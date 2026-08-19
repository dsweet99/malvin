# Incomplete handoff envelope

## Done
- Read the named plan: make `malvin --model=codex:gpt-5.6 --do Hello` work and improve `malvin models` support for `codex:` models.
- Confirmed live `malvin models codex:` discovery works and returns six live models: `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-5.6-luna`, `gpt-5.5`, `gpt-5.4`, and `gpt-5.4-mini`.
- Confirmed `codex app-server --stdio` is required by the installed Codex CLI; both Codex launch paths specify it in `src/codex_sdk/discover.rs` and `src/codex_sdk/session_spawn.rs`.
- Direct protocol evidence: `thread/start` accepts `gpt-5.6`, but `turn/start` fails with provider error 400: `The 'gpt-5.6' model is not supported when using Codex with a ChatGPT account.` The live `model/list` response shows the supported suffixed IDs.
- Applied the smallest current repair in `src/codex_sdk/session_spawn.rs`: `codex_start_thread` now resolves an unavailable generic model to the first live model whose ID starts with `<requested>-`, while preserving exact live IDs and falling back to the requested ID if discovery fails.
- `cargo check --all-targets --all-features` passed after the dynamic-resolution edit.
- Earlier complete-gate evidence exists for the prior state: `ruff check` passed; `kiss check` passed after coverage cleanup; Clippy passed; `pytest tests` had 156 passed; the Rust gate had 802 and 829 tests passed. These are not final-state evidence after the newest edit.

## Remaining
- The exact requested command has not yet been rerun after dynamic model resolution. The last authoritative run before this edit recorded `run_done status=failed` with no response, caused by the unsupported generic model.
- All five required checks must be rerun after the newest edit, sequentially: `ruff check`, `kiss check`, `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`, `pytest tests`, and `./admin/malvin_rust_test_gate.sh`.
- Add focused regression coverage for `resolve_codex_model`, including exact live ID preservation, suffixed fallback, and discovery failure fallback, without relying on external network/provider behavior.
- Rebuild and rerun `target/debug/malvin --model=codex:gpt-5.6 --do Hello`; inspect the authoritative newest `trace.jsonl` and require a successful completion plus a nonempty response. Also rerun `target/debug/malvin models codex:`.
- Do not edit linter thresholds/configuration. Do not add or commit unrelated pre-existing untracked operator artifacts.

## Next-agent starting position
1. Inspect `git diff` and `src/codex_sdk/session_spawn.rs` around `resolve_codex_model` and `codex_start_thread`.
2. Add unit tests using a fake `MALVIN_CODEX` app-server/model-list process or extract a pure helper so model resolution is deterministic and covered.
3. Run `cargo check --all-targets --all-features`, then `kiss check`.
4. Run the five required checks one at a time, waiting for each to finish.
5. Build the binary and run the exact requested command. Inspect the newest run directory's `trace.jsonl`, `stdout.log`, `do.log`, and `run_timing.json`; do not treat exit code alone as success.
6. Commit only intended tracked changes after all direct criteria pass.

The named Done criterion remains unresolved until the exact command succeeds with a Codex response and all five required checks pass after the final edits.
