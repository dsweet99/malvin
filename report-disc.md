# Codex vs Cursor connector discrepancies

Scope: malvin’s `codex:` backend (`src/codex_sdk`) versus the `cursor:` backend (`src/cursor_sdk` + `cursor-sdk-bridge`). Shared host machinery is noted first so the rest of the report can stay on real differences.

Pi is mentioned only when it clarifies a Cursor/Codex split.

---

## What is the same

Both backends are `SdkClient` sessions (`src/agent_backend/sdk_client.rs`). Spawn, prompt, and shutdown go through the same client. Stream events are reduced to `BridgeEvent` and printed/traced by one adapter (`src/bridge_sdk/log_adapter.rs`).

Host-side process wrapping is also shared:

- `malvin_tokio_command` / `malvin_std_command`: new process group, `MALLOC_ARENA_MAX=2`, parent-death signal (`src/malvin_sandbox.rs`).
- One active sandbox session, ACP spawn lock, “dead before next spawn”.
- RSS watcher (`start_mem_watch` → `watch_process_group_memory`).
- Drain-idle timeout with child-health extend (`src/bridge_sdk/drain_idle.rs`).
- Teardown: signal the process group, `kill`/`wait` the child, `clear_active_sandbox_session`.

Log *files* and stdout *line shape* are therefore the same envelope. Differences below are in the child process, the wire protocol, and what events actually get mapped.

Child **stderr** is `Stdio::inherit()` for both session processes. Cursor also sets Node `--no-warnings` / `NODE_NO_WARNINGS=1`. Codex does not. Inherited child stderr is **not** rewritten into `stdout.log` as `e|` lines (malvin’s tagged stderr helper is only for malvin’s own messages). Unlabeled `--do Hello` runs in `command.log` are not a reliable backend label: they follow `~/.malvin_home/config.toml` `agent.model` (currently `pi:`).

---

## 1. Child process and wire protocol

| | Cursor | Codex |
|---|---|---|
| Child | Node (`cursor-sdk-bridge`) | `codex app-server --stdio` |
| Wire | `BridgeRequest` / `BridgeEvent` JSON lines (`BridgeWire::NodeBridge`) | JSON-RPC methods (`BridgeWire::CodexRpc`) |
| Binary | bundled bridge + Node ≥ 22.13 | external `codex` (`PATH` or `MALVIN_CODEX`) |
| Auth at spawn | `CURSOR_API_KEY` (also `CURSOR_AGENT_API_KEY` / `AGENT_API_KEY`) | none in malvin; Codex CLI’s own login |

Evidence: `src/cursor_sdk/session_spawn.rs` vs `src/codex_sdk/session_process.rs`; `BridgeSession::send_prompt` in `src/bridge_sdk/session.rs`.

Cursor speaks a small private protocol (`create` / `resume` / `send` / `cancel` / `close`). Codex speaks Codex app-server RPC (`initialize`, `thread/start`, `turn/start`, `turn/interrupt`, `thread/delete`, plus item notifications).

**Prompt send (same host call, different wire):** `BridgeSession::send_prompt` (`src/bridge_sdk/session.rs`).

- Cursor: one `Send { prompt }` JSON line, then drain until `run_done`.
- Codex: JSON-RPC `turn/start` with `threadId` and `input: [{ "type": "text", "text": prompt }]`, then consume until `turn/completed` for that turn id (`src/codex_sdk/session_io.rs`). There is no Codex equivalent of Cursor `Step`; `note_sdk_step` is never called on the Codex path.

---

## 2. Startup

### Cursor

1. Fail if a previous sandbox is still alive.
2. Resolve Node + `bridge.js`, spawn with piped stdin/stdout, stderr inherited.
3. Set `NODE_COMPILE_CACHE` under `~/.malvin_home/node_compile_cache` (or temp).
4. Pass `CURSOR_API_KEY` into the child env when present.
5. Start the RSS watcher.
6. `create` (or `resume` if `last_agent_id` is set). Wait for `ok` with a **startup** timeout (`sdk_bridge_startup_timeout`), not the drain-idle loop.
7. Store Cursor’s `agentId` for later resume.

`--no-force` is *not* rejected in Rust. The bridge gets `noForcePolicy: "fail_fast"` and emits `fatal` after the Node process is already up (`cursor-sdk-bridge/src/bridge.ts` `rejectNoForce`).

Bracket model params (`thinking`, `effort`, `fast`, …) are forwarded as `cursor_bridge_model()` (`src/model_id.rs`, `spawn_model_wire` in `sdk_client_session.rs`).

### Codex

1. If `!force`, **return before spawn** with `--no-force is not supported for codex:`.
2. Fail if a previous sandbox is still alive.
3. Resolve `MALVIN_CODEX` or `codex` on `PATH`. Spawn `codex app-server --stdio`. stderr inherited. No Node cache, no API key env.
4. Start the RSS watcher.
5. RPC `initialize` (clientInfo `malvin`, `capabilities.experimentalApi: true`) then notify `initialized`.
6. Paginate `model/list` (`includeHidden: true`) on that live session. If listing works, unknown slugs error; if listing fails, keep the user slug. Family names such as `gpt-5.6` map to the first catalog id with that prefix.
7. `thread/start` with `approvalPolicy: "never"`, `sandbox: "workspace-write"`, `ephemeral: true`. Thread id is stored in `agent_id`.

`thinking` brackets are **dropped**. `spawn_thinking_wire` only forwards thinking for `BridgeKind::Pi`.

There is **no resume**. `resume_agent_id` is always `None` for Codex. Ending a session does not remember the thread id (`end_coder_session` only calls `remember_agent_id_from` for Cursor).

Evidence: `src/codex_sdk/session_spawn.rs`, `session_protocol.rs`; `src/agent_backend/sdk_client_session.rs`.

### Extra catalog process (both; different binaries)

`malvin models` for Codex starts a **second** `codex app-server` via `malvin_std_command`, stderr to `/dev/null`, with a 30s timeout (`MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS`). Cursor listing uses Node `models.js` (or `agent` / `cursor-agent models` fallback) with `MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS`, also via `malvin_std_command`. **Neither** listing child is registered with `note_active_sandbox_session`.

---

## 3. Teardown

Shared `BridgeSession::shutdown` (`src/bridge_sdk/session.rs`):

| | Cursor | Codex |
|---|---|---|
| Soft stop | `cancel` then `close` | `turn/interrupt` (only if both thread id and turn id are set) then `thread/delete` |
| Hard stop | same: process-group terminate, `child.kill()`, wait, clear sandbox | same |
| After `end_coder_session` | keep `last_agent_id` for resume (unless Cursor “already has active run”) | forget the thread; next spawn is a new ephemeral thread |

Drop without async shutdown still kills the process group and, on Unix outside a Tokio runtime, `mem::forget`s the Tokio `Child` so Drop does not panic.

Codex interrupt is a no-op if the turn has already finished (`set_codex_turn_id(None)` in `finish_codex_turn`). Cursor cancel is always attempted.

A Codex helper `thread_became_idle` (`thread/status/changed` → idle) exists only under `#[cfg(test)]`. Production completion is **`turn/completed` only**. Idle-status notifications are ignored.

---

## 4. Sandboxing (two layers)

**Host wrapper (malvin):** same for both session children (process group, arena cap, death signal, spawn lock, RSS cap).

**Agent policy (what tools may do):**

- Cursor: default `create` does **not** set Cursor SDK `sandboxOptions`. Those are only enabled if `sandboxEnabled` or `noForcePolicy === "sandbox"` (`cursor-sdk-bridge/src/bridge.ts` `agentOptionsFromBoot`). Production `send_create` never sets those flags. Tools therefore run under Cursor’s default local agent, not malvin’s “workspace-write” policy.
- Codex: every thread is started with Codex `sandbox: "workspace-write"` and `approvalPolicy: "never"`. That is Codex’s own sandbox, not malvin’s process-group wrapper.

**Not live-tested.** Code says Codex threads always pass `sandbox: "workspace-write"`; Cursor production `create` does not set `sandboxOptions`. Whether that is stricter on disk is an untested hypothesis.

Catalog listing (both backends) uses the host wrapper (`malvin_std_command`) and **neither** takes the active-sandbox lock.

---

## 5. Monitoring

| | Cursor | Codex |
|---|---|---|
| RSS watch | yes, after spawn | yes, after spawn |
| Silence during a turn | `bridge timed out waiting for run_done` | `codex timed out waiting for turn event` |
| Create / initialize wait | hard startup timeout on `ok` | drain-idle on each RPC reply (`waiting for rpc reply`) |
| Hung child | same health verdicts (busy / hung / dead) | same |

Transport errors that force session teardown include both `bridge timed out` and `codex timed out` (`src/acp/retry_teardown.rs`). Cursor also tears down and **forgets** `last_agent_id` on “already has active run”. Codex has no equivalent.

---

## 6. Log files and stdout format

### Shared envelope

Run directory files (`command.log`, `stdout.log`, `prompts.log`, `trace.jsonl`, `work_dir`) are the same set.

Stdout lines are `YYYYMMDD.HHMMSS.mmm <who>|<text>` (`u` user, `o` operator/system, `m` model, `t` tool, `b` thought, `e` error, `h` harness).

`trace.jsonl` records:

```json
{"ts":"...","name":"sdk","direction":"in","message":{...}}
```

Neither connector writes outbound RPC/`create`/`send` lines to `trace.jsonl` (direction is always `"in"` for stream events).

### Content differences (observed)

**`--do Hello` traces**

- Codex `20260821_074552_jqik6ywb` (`malvin --model=codex:gpt-5.6-terra --do Hello`): assistant deltas, then `run_done` with `"status":"completed"`, `"usage":null`, `"durationMs":null`. `run_timing.json` has `tokens_in`/`tokens_out` null.
- Cursor router `20260818_001310_dfvpd9gu` (`Model: cursor:auto`): `run_done` `"status":"finished"` with usage keys `inputTokens` / `outputTokens` / `cacheReadTokens` (not the Pi-style `input` / `cacheRead` fold). Explicit `cursor:` `--do Hello` samples in this tree (`20260818_121514_equd8gi9`, `20260818_121529_tngfdy0x`) failed with usage-limit `"status":"error"`. Do not cite `20260819_120243_51agy17v`: its command log says `Model: pi:openrouter/openai/gpt-5.6-luna`.
- Do not treat unlabeled `malvin --do Hello` as Cursor. `20260820_210607_zrg7aj9e` is `finished` with usage whose `tokens_in` equals `input + cacheRead` (Pi fold). `20260820_150121_fbpud6bi` is `completed` with null usage (Codex-shaped).

Stdout for a successful `--do Hello` is the DM fence plus `Hello.` Tagged `u|[do...]` is **not** a connector event. It is host `print_outgoing_prompt_log` (`src/output/mod.rs`), which writes `[<label>...]` to `stdout.log` only (not the live terminal). `--do` uses label `do`. It is skipped when `raw_output` or `no_tee` is set (`emit_prompt_stdout`). Codex run `074552` has the line; Codex run `060704` does not. That is a host tee/log-path difference between those two invocations, not Cursor vs Codex.

**Router runs with tools**

- Cursor `20260818_001202_mz47s23j` (`cursor:auto`): `b|` thought lines on stdout; trace `thinking` + `assistant` + `tool_call`. Tools named `glob`, `shell`, etc. Completions omit `· ✓` except shell (`Glob VISION.md · 103ms` vs `Run … · 402ms · ✓`).
- Codex `20260821_065952_i4tfwqiz` (`codex:gpt-5.6-terra` Hello): **no** `b|` and **no** `thinking` events in `trace.jsonl` (2325 assistant, 65 tool_call, 24 run_done). Tools are almost all `name: "shell"` with summaries like `Run /bin/bash -lc "…"`. Many completions show `· 0ms · ✓`.

The Codex mapper turns `item/reasoning/textDelta` and `summaryTextDelta` into `BridgeEvent::Thinking` (`src/codex_sdk/map_event.rs`; unit test `maps_assistant_and_reasoning_deltas`). `emit_turn_stream` only traces **mapped** events. Other RPC methods (`item/updated`, token notifications, …) become `Vec::new()` and never appear in `trace.jsonl`. Live Codex traces here have **zero** `event: thinking`. That means either the CLI did not send those two methods, or it sent a reasoning shape malvin does not match. The traces cannot distinguish those cases.

**`run_done` status vocabulary**

- Cursor: the Node bridge forwards Cursor SDK `result.status` (`cursor-sdk-bridge/src/bridge.ts`). Observed successes use `"finished"`; failures use `"error"`. Shared drain treats `"error"` and `"cancelled"` as failures (`run_done_status_is_failure`). Pi uses `"finished"` as well, which is why unlabeled Pi `--do` looks Cursor-like.
- Codex: Codex turn status, typically `"completed"`, also `"failed"` / `"interrupted"` treated as errors in `finish_codex_status`. `"completed"` is success even though Cursor uses `"finished"`.

Trace consumers that key on `"finished"` will miss Codex completions.

**Usage / cost / duration / steps**

Cursor `RunDone` carries usage and `durationMs`; `record_sdk_usage` updates run timing. Codex `emit_codex_done` always sets `usage: None` and `duration_ms: None` (`session_turn.rs`). Token cost for Codex sessions is therefore empty unless something else fills it.

Observed on Cursor router `20260818_001310_dfvpd9gu`: `durationMs: 12383` on the last `run_done`, `run_timing.json` `tokens.steps: 194`. Those 194 are **not** in `trace.jsonl` (`event: step` count is 0). Cursor `drain_until_run_done` handles `BridgeEvent::Step` with `note_sdk_step` and does not pass Step to `handle_stream_event`, so Step never reaches the trace. Progress (183 in that trace) is traced. Codex `--do` `20260821_074552_jqik6ywb` and Codex router `20260821_065952_i4tfwqiz`: `durationMs: null`, `steps: 0`, `tokens_*` null, no `cost` object. Codex never calls `note_sdk_step`.

Cursor usage objects in this tree use SDK token keys (`inputTokens`, `outputTokens`, `cacheReadTokens`, `cacheWriteTokens`, plus `reasoningTokens` / `totalTokens` on `20260818_001310_dfvpd9gu`). Short keys `input` / `cacheRead` are the **Pi** wire shape. `normalize_pi_usage` (Pi only) aliases those to `*Tokens` and, in `run_timing.json`, folds cache into `tokens_in`. Cursor and Codex leave `normalize_pi_usage` false. `record_acp_usage_if_present` only reads `*Tokens` fields, so a Cursor `input`/`cacheRead` object would not update timing — that is not what the sampled Cursor traces contain.

**Tool stdout**

Same `t|` formatter. Codex `commandExecution` is labeled `shell` and summarized as `Run …`, so it always gets the `· ✓` / `· ✗` suffix. Cursor file tools do not. Codex does not emit Pi-style `phase: "update"` (those are traced but not printed).

**Thoughts on stdout**

Default router sets `show_thoughts_on_stdout: true`; `--do` sets it false. When thoughts are enabled, Cursor shows `b|`. Codex would too *if* reasoning deltas arrived. They did not in the sampled Codex router logs.

---

## 7. Auth and `--no-force`

| | Cursor | Codex |
|---|---|---|
| `ensure_authenticated` | requires API key | always `Ok(())` |
| `--no-force` | spawn Node, then fatal from the bridge | fail in Rust before spawn |
| Error text | “not supported with the Cursor SDK backend…” | “not supported for codex: (malvin runs Codex tools headlessly…)” |

Help text groups all three backends as unsupported (`shared_opts.rs`). The *path* still differs: Cursor pays spawn cost; Codex does not.

---

## 8. Session identity and retries

Cursor keeps `agent_id` across `end_coder_session` and passes it as `resume_agent_id` on the next spawn. Codex stores the thread id in the same field during the session but never resumes it. Threads are `ephemeral: true` and deleted on shutdown.

On spawn failure, Cursor may append `(resume failed; will create)` and clear `last_agent_id`. Codex has no resume branch.

Max age restart (`SDK_BRIDGE_MAX_AGE`, 10 minutes) applies to both.

---

## 9. Model listing and selection

Cursor: SDK `models.js`, guarantee `cursor:auto` even if the catalog says `default`, optional tab-separated param columns, CLI fallback.

Codex: live `model/list` including hidden ids; listing extras `thinking=`, `service=`, `hidden`, `default`; family alias at **spawn**, not at listing time. Bracket params are not applied.

Pi (for contrast): `pi --list-models` plus auth-env filter; `thinking` bracket becomes `--thinking`.

---

## 10. Event mapping gaps (code, not just logs)

Codex maps:

- `item/agentMessage/delta` → assistant
- `item/reasoning/textDelta` and `summaryTextDelta` → thinking
- `item/started` / `item/completed` for tool-like item types (`commandExecution`, `fileChange`, `mcpToolCall`, `webSearch`, …)

Not mapped (dropped): token/usage notifications, progress, collaboration UI, generic `item/updated`, and anything that is not in that match list. Cursor’s Node mapper (`sdk_map.ts`) forwards SDK usage, step, and richer tool results (byte sizes, exit codes) into the same `BridgeEvent` types.

Codex `item/completed` for `agentMessage` also **replaces** accumulated delta text with the completed item text (`completed_agent_text`). Cursor accumulates deltas only.

---

## 11. Practical impact

What an operator comparing log directories will notice:

1. Same file names and `ts|who|` stdout grammar.
2. Codex `run_done` says `completed` with no usage and no `durationMs`; Cursor successes say `finished` with usage and `durationMs`. Codex `run_timing.json` leaves `steps` at 0; Cursor counts `Step` events. Unlabeled `--do Hello` is not enough to tell them apart (Pi also says `finished` with usage).
3. Codex router traces often lack `thinking` / `b|` even when thoughts are enabled.
4. Codex tools look like one `shell` wrapping `bash -lc`; Cursor shows `Read` / `Glob` / `Run` as separate names.
5. Codex does not resume; each outer loop iteration that respawns is a new ephemeral thread.
6. Host memory/process sandbox is shared; **in-agent** sandbox is Codex `workspace-write` vs Cursor default (no malvin-enabled SDK sandbox).
7. `--no-force` fails earlier on Codex than on Cursor.
8. Codex needs a separate CLI install and does not check an API key inside malvin.

---

## Evidence index

| Topic | Code | Logs |
|---|---|---|
| Factory / kind | `src/agent_backend/factory.rs`, `sdk_client.rs` | |
| Cursor spawn | `src/cursor_sdk/session_spawn.rs` | |
| Codex spawn / RPC | `src/codex_sdk/session_process.rs`, `session_protocol.rs` | |
| Shutdown | `src/bridge_sdk/session.rs` | |
| Stream logging | `src/bridge_sdk/log_adapter.rs` | traces named above |
| Codex mapping | `src/codex_sdk/map_event.rs`, `session_turn.rs` | `20260821_065952_i4tfwqiz` |
| Cursor `run_done` + usage | `cursor-sdk-bridge/src/bridge.ts` | `20260818_001310_dfvpd9gu` (`Model: cursor:auto`) |
| Codex `--do` no usage | | `20260821_074552_jqik6ywb` |
| Unlabeled `--do` is not Cursor | | `20260820_210607_zrg7aj9e` (Pi fold), `20260820_150121_fbpud6bi` (Codex-shaped) |
| Cursor tools + `b|` | | `20260818_001202_mz47s23j` (`Model: cursor:auto`) |
| `[do...]` host log | `src/output/mod.rs` `print_outgoing_prompt_log` | Codex `074552` has it; Codex `060704` does not |
| Host sandbox | `src/malvin_sandbox.rs` | |
| Cursor SDK sandbox flags | `cursor-sdk-bridge/src/bridge.ts` | |
| Codex thread sandbox | `session_protocol.rs` `thread/start` | |
