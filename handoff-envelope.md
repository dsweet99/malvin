# Incomplete handoff envelope

## Done
- Read the named user plan. It requires Codex model selection and `malvin models` discovery for `codex:`.
- Preserved unrelated untracked operator artifacts.
- Direct `kiss check` initially failed only on duplicate executable-helper coverage, then backend-specific names resolved that coverage failure.
- `cargo check --all-targets --all-features` passed before the latest structural extraction sequence.
- `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo` passed before the latest structural extraction sequence.
- Extracted protocol phases from `list_codex_models`, `codex_spawn_bridge`, and `codex_send_prompt` to address kiss calls-per-function violations.

## Remaining
- Current `src/codex_sdk/session_spawn.rs` has just been repaired after a mechanical edit removed the `CodexProcess` destructuring line. Compilation has not been rerun after that repair.
- `kiss check` was still failing before the repair with structural violations in `build_codex_session`, `spawn_codex_process`, `consume_codex_turn`, and `finish_codex_turn`; the latest grouping/extraction attempt was interrupted by the handoff request and must be validated.
- Required checks for the final state remain unverified: `ruff check`, `kiss check`, Clippy after final edits, `pytest tests`, and `./admin/malvin_rust_test_gate.sh`.
- Do not edit linter thresholds/configuration. Do not include untracked operator artifacts.

## Next-agent starting position
1. Run `cargo check --all-targets --all-features` immediately.
2. Inspect `src/codex_sdk/session_spawn.rs` around `CodexProcess`, `build_codex_session`, and `spawn_codex_process`.
3. Run `kiss check`; address only its reported structural violations using private helpers or small structs while preserving protocol order.
4. Run all five named checks sequentially, with no overlapping heavy commands.
5. Review `git diff --check`, status, and the final diff. Commit only intended tracked files.

The named Done criterion is unsatisfied until direct `kiss check` passes and all required checks are rerun successfully.
