# Incomplete handoff: Codex/Cursor shared `run_done` representation

Status: production mapping for canonical `run_done` status, Codex usage, duration, reasoning-item thinking, and step counts is in the working tree. Unit tests for the new helpers passed. `ruff check` passed. `kiss check` **failed** on `json_u64` / `json_i64` nested-closure depth. Remaining `.malvin/gates` not run. No `--git`. Stopped at 80% tool-iteration budget.

Operator request: `malvin -g Fix the discrepancies in report-disc.md. Ideally, change our representation so that discrepancy is impossible.`

VISION.md: `pi:` / `codex:` logs should look like `cursor:` logs.

## Done (in tree)

Shared representation so Codex cannot emit Cursor-incompatible `run_done` vocabulary:

1. `src/bridge_protocol.rs`: `canonical_run_done_status`
   - `completed`/`finished` → `finished`
   - `failed`/`error` → `error`
   - `interrupted`/`cancelled` → `cancelled`
2. `src/codex_sdk/map_event.rs`:
   - `thread/tokenUsage/updated` → `BridgeEvent::Usage` with Cursor keys (`inputTokens`, `outputTokens`, `cacheReadTokens`, `cacheWriteTokens`, `reasoningTokens`, `totalTokens`)
   - `item/completed` of type `reasoning` → `Thinking` (fallback when deltas never arrive)
3. `src/codex_sdk/session_turn.rs`: remember last usage; `note_sdk_step` on tool `start`
4. `src/codex_sdk/session_turn_done.rs` (new): `run_done_from_turn` / `finish_codex_turn` emit shared `RunDone` with status, usage, `durationMs`, error
5. Tests: `run_done_uses_shared_finished_status_and_usage`, duration from `durationMs` or `startedAt`/`completedAt`, usage+reasoning mapping, `canonical_run_done_status` in protocol decode test

Unit tests run (passed, 11): `maps_assistant_and_reasoning_deltas`, command/file/collab tool maps, `failed_and_interrupted_turns_are_errors`, `agent_text_comes_from_items_not_last_agent_message`, `idle_status_does_not_complete_a_turn`, `turn_duration_and_canonical_failure_status`, `run_done_uses_shared_finished_status_and_usage`, `decode_run_done_and_fatal`.

`ruff check` passed.

## Remains

1. **Fix kiss now** (`src/codex_sdk/session_turn_done.rs` lines 86 and 93):
   ```
   nested_function_depth json_u64 / json_i64: 3 nested closure depth (threshold: 2)
   ```
   Flatten `or_else(|| v.as_i64().and_then(...))` into match / if-let, no nested closures.
2. Run `.malvin/gates` **one line at a time**:
   - `kiss check` (after the flatten)
   - `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`
   - `pytest tests`
   - `./admin/malvin_rust_test_gate.sh`
3. Format only touched files with `rustfmt --edition 2024 <files>`. Do **not** `cargo fmt --all`.
4. Optional leftover discrepancies from `report-disc.md` **not** made representation-impossible this turn:
   - `--no-force` still fails before spawn on Codex vs after Node spawn on Cursor
   - Codex does not resume (`ephemeral: true`, no `last_agent_id`)
   - In-agent sandbox still Codex `workspace-write` vs Cursor default
   - Tool *names* still differ (`shell` wrapping bash vs Cursor `Read`/`Glob`) — host `t|` grammar is already shared
   - Live Codex thinking still depends on CLI emitting deltas or completed `reasoning` items
5. Do not commit (no `--git`). Do not revert unrelated dirty/untracked files (handoff md, `report-disc.md`, `session_spawn_unix_mock.sh`).
6. After gates: `malvin --model=codex:gpt-5.6 --do Hello` should still greet; a `--do` trace `run_done` should say `"status":"finished"` not `"completed"`.

## Next-agent start

Work dir: `/home/dsweet/Projects/malvin`. First edit: flatten `json_u64` / `json_i64` in `src/codex_sdk/session_turn_done.rs`. Then `kiss check`. Touched files:

- `src/bridge_protocol.rs`
- `src/bridge_sdk/mod.rs`
- `src/codex_sdk/map_event.rs`
- `src/codex_sdk/map_event_tests.rs`
- `src/codex_sdk/map_event_more_tests.rs`
- `src/codex_sdk/mod.rs`
- `src/codex_sdk/session_turn.rs`
- `src/codex_sdk/session_turn_done.rs` (new)
- `src/codex_sdk/session_turn_tests.rs`

KPop log: `~/.malvin_home/logs/eb7ef333a92a6d41/20260821_135016_sl3hvcdx/`
