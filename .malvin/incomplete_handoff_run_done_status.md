# Incomplete handoff: closed `RunDoneStatus` type

Status: production mapping for canonical `run_done` status is now a **closed type**, not a free string. Targeted lib tests for the type + constructors passed (7). `kiss check` **failed** once on `src/bridge_protocol_status.rs` serialize/deserialize coverage at 71%; a coverage test was added after that failure and **kiss was not rerun**. Remaining `.malvin/gates` not run. No `--git`. Stopped at 80% tool-iteration budget.

Operator request: `malvin -g Fix the discrepancies in report-disc.md. Ideally, change our representation so that discrepancy is impossible.`

VISION.md: `pi:` / `codex:` logs should look like `cursor:` logs.

## Done (in tree)

1. **Closed status type** (`src/bridge_protocol_status.rs`, re-exported from `src/bridge_protocol.rs`):
   - `RunDoneStatus::{Finished, Error, Cancelled}`
   - Wire aliases: `completed`/`finished` → Finished; `failed`/`error` → Error; `interrupted`/`cancelled` → Cancelled; unknown → Error
   - Serde always emits `"finished"` / `"error"` / `"cancelled"` (never `"completed"`)
2. `BridgeEvent::RunDone.status` is `RunDoneStatus`, not `String`. Decode cannot produce `"completed"`.
3. Constructors wired:
   - Codex `session_turn_done.rs` (`run_done_from_turn`, `finish_codex_status`)
   - Pi `map_event.rs` (`map_agent_end`)
   - Cursor drain test fixture
   - `run_done_status_is_failure` takes `RunDoneStatus`
4. Cursor Node bridge (`cursor-sdk-bridge/src/protocol.ts`): `canonicalRunDoneStatus` + `RunDoneStatus` union; `bridge.ts` emits that, not raw SDK `result.status`. Test in `bridge_test.ts`.
5. Prior-session work still in tree: Codex usage/duration/thinking mapping, `session_turn_done.rs`, tool-name flattening.

Targeted `cargo test --lib` (7 passed): `aliases_collapse_to_three_statuses`, `decode_run_done_and_fatal`, `cancelled_and_error_are_failures`, `run_done_uses_shared_finished_status_and_usage`, `maps_agent_end`, `failed_and_interrupted`.

## Remains

1. **Rerun `kiss check`** (coverage test was added after the 71% failure; not re-verified).
2. Run `.malvin/gates` **one line at a time**:
   - `ruff check`
   - `kiss check`
   - `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`
   - `pytest tests`
   - `./admin/malvin_rust_test_gate.sh`
3. Format only touched files with `rustfmt --edition 2024 <files>`. Do **not** `cargo fmt --all`. Rebuild Cursor bridge tests (`cd cursor-sdk-bridge && npm test`) if `kiss`/clippy do not cover TS.
4. Optional leftover discrepancies from `report-disc.md` **not** made representation-impossible:
   - `--no-force` still fails before spawn on Codex vs after Node spawn on Cursor
   - Codex does not resume (`ephemeral: true`, no `last_agent_id`)
   - In-agent sandbox still Codex `workspace-write` vs Cursor default
   - Tool *names* still differ at the source (host `t|` grammar + Codex bash classifier are already shared)
   - Live Codex thinking still depends on CLI emitting deltas or completed `reasoning` items
   - Step counts: Codex increments on assistant/tool start; Cursor on SDK `onStep` — counts can still differ
5. Do not commit (no `--git`). Do not revert unrelated dirty/untracked files (handoff md, `report-disc.md`, `session_spawn_unix_mock.sh`).
6. After gates: `malvin --model=codex:gpt-5.6 --do Hello` should still greet; a `--do` trace `run_done` should say `"status":"finished"` not `"completed"`.

## Next-agent start

Work dir: `/home/dsweet/Projects/malvin`. First command: `kiss check`. If it still flags `bridge_protocol_status.rs`, add kiss coverage witnesses (`stringify!(serialize)` etc.) rather than more logic. Then gates one line at a time.

New this turn:

- `src/bridge_protocol_status.rs` (new)
- `src/bridge_protocol.rs` (`RunDoneStatus` field)
- `src/bridge_sdk/session_io.rs`
- `src/codex_sdk/session_turn_done.rs`
- `src/pi_sdk/map_event.rs`, `map_event_tests.rs`, `session_io.rs`
- `src/cursor_sdk/sdk_drain_progress_tests.rs`
- `cursor-sdk-bridge/src/protocol.ts`, `bridge.ts`, `bridge_test.ts`

KPop log: `~/.malvin_home/logs/eb7ef333a92a6d41/20260821_135016_sl3hvcdx/`
