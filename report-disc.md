# Backend connector discrepancies: Cursor, Pi, Codex

This report compares the three agent backends (`cursor:`, `pi:`, `codex:`). It covers the shared abstraction, the three transports, and what a user sees.

Evidence is from the current tree under `src/agent_backend/`, `src/cursor_sdk/`, `src/pi_sdk/`, `src/codex_sdk/`, `src/bridge_sdk/`, `src/cli/models_cmd*.rs`, `src/model_id.rs`, `README.md`, and `VISION.md`.

---

## Shared design (what is the same)

All three backends are selected the same way: `--model` is parsed into `ParsedModel`, then `build_agent_backend` maps `ModelBackend` to `BridgeKind` and builds one `SdkClient` (`src/agent_backend/factory.rs`).

The operator-facing session API is also the same object:

- `ensure_authenticated`
- `ensure_coder_session` / `begin_coder_session` / `end_coder_session`
- `run_coder_prompt`
- retries (`max_acp_retries`), prompt logs, run timing

Streamed assistant text, thinking, tool lines, usage, and `run_done` are supposed to look the same. `VISION.md` says `pi:` logs and stdout should look like `cursor:`. Codex is folded into the same `BridgeEvent` vocabulary (`src/bridge_protocol.rs`) and the same log adapter (`src/bridge_sdk/log_adapter.rs`).

`run_done.status` is already canonicalized (`Finished`, `Error`, and related values) so traces do not keep each vendor’s raw status strings.

---

## Architecture: the abstraction is not three-way

**Claim.** The public type is one client, but the session object is two kinds, not three.

`AgentBackend` is only a type alias for `SdkClient` (`src/agent_backend/backend.rs`). There is no trait with three implementations.

`SdkSession` is:

```text
Bridge(Box<BridgeSession>)   // Cursor and Codex
Pi(Box<PiEmbeddedSession>)   // Pi only
```

(`src/agent_backend/sdk_session.rs`)

Codex reuses Cursor’s child-process session (`BridgeSession`) and switches behavior with `BridgeWire::CodexRpc` versus `BridgeWire::NodeBridge` (`src/bridge_sdk/session.rs`). Pi never uses that wire. It runs `pi_agent_rust` on a dedicated thread (`src/pi_sdk/runtime.rs`).

So the “three connectors” sit on two different session types, and Codex is a mode of the Cursor-shaped session.

Shared spawn arguments mix backend-specific fields. `BridgeSpawnArgs` includes `normalize_pi_usage` (set only for Pi) and `resume_agent_id` (set only for Cursor) (`src/bridge_sdk/session.rs`, `src/agent_backend/sdk_client_session.rs`). Codex ignores both.

`ModelBackend` and `BridgeKind` are three-way and must stay in lockstep (`bridge_kind_matches_backend`). `BridgeWire` is not a third copy of that idea: it is only `NodeBridge` | `CodexRpc`, because Pi is not a child-process wire.

---

## Transport and process model

| | Cursor | Pi | Codex |
|---|---|---|---|
| Process | Node child running `cursor-sdk-bridge` (`@cursor/sdk`) | In-process crate + worker thread; no `pi` binary | External `codex app-server --stdio` |
| Install | Built/installed with malvin (`build.rs`, `~/.malvin_home/sdk-bridges/`) | Linked crate `pi_agent_rust` | Operator must install Codex; `MALVIN_CODEX` optional |
| Wire | Line JSON `BridgeRequest` / `BridgeEvent` | `pi::sdk::AgentEvent` mapped to `BridgeEvent` | JSON-RPC methods (`initialize`, `thread/start`, `turn/start`, …) mapped to `BridgeEvent` |
| Session id | Cursor `agentId`; remembered and reused | None | Codex `threadId` stored in the same `agent_id` field |
| Resume | Yes, on later spawn | No | No (thread is deleted on shutdown) |
| Memory watch | Process group of the Node child | `pgid: 0` plus spawn PID baseline | Process group of the Codex child |

**Claim.** Only Cursor can continue an agent across a bridge restart. `last_agent_id` is saved and passed as `resume_agent_id` only when `BridgeKind::Cursor` (`src/agent_backend/sdk_client_session.rs`). Codex stores a thread id in the same mutex, but spawn never resumes it. Pi has no agent id.

**Claim.** Pi’s memory watcher is not process-group based the way the other two are. `start_embedded_mem_watch` calls `watch_process_group_memory_with_optional_pgid` with `pgid: 0` (`src/pi_sdk/session_spawn.rs`). Cursor and Codex pass the child pid as the group id.

**Hypothesis.** A Pi run that starts many tool children may be measured differently for the memory cap than a Cursor or Codex run, because there is no single child process group to watch.

Pi also has a test-only fake session (`MALVIN_TEST_NO_REAL_AGENT` / `session_fake.rs`). Cursor and Codex use mock binaries/scripts instead.

---

## Authentication

**Claim.** Preflight auth is not the same for the three backends (`SdkClient::ensure_authenticated`):

- **Cursor:** requires `CURSOR_API_KEY`, `CURSOR_AGENT_API_KEY`, or `AGENT_API_KEY`. `agent login` alone is not enough (`src/cursor_sdk/auth.rs`).
- **Pi:** checks the provider in `pi:<provider>/<model>`. Env keys from Pi’s provider metadata, or a credential already stored in Pi’s auth file, count as authenticated (`src/pi_sdk/auth.rs`). Unknown providers are treated as authenticated.
- **Codex:** always `Ok(())`. There is no malvin-side check that Codex is logged in. Failure appears later as a spawn or turn error.

**Claim.** `malvin models` for Pi does **not** use the same auth rule as a real run. Listing keeps a row only when an env API key is set (`provider_authenticated_from_map` in `src/pi_sdk/providers_list.rs`). A stored Pi credential is enough to *run* `pi:openai/...`, but not enough to *list* that provider. Tests lock this in (`src/cli/models_cmd_auth_filter_tests.rs`).

Cursor listing does not filter by whether a key is present. Codex listing talks to a live app-server and will print `(codex models unavailable: …)` if the binary is missing or the catalog call fails.

---

## Model ids and bracket overrides

All ids must be prefixed (`cursor:`, `pi:`, or `codex:`). Unprefixed and legacy prefixes (`mini:`, `openrouter:`, `local:`, `prime:`) are rejected (`src/model_id.rs`).

**Claim.** The id *shape* differs:

- Cursor: any non-empty slug (`cursor:auto`, `cursor:claude-opus-5`, …).
- Pi: must be `pi:<provider>/<model>` (slash required).
- Codex: any non-empty slug (`codex:gpt-5.6`). At spawn, if `model/list` succeeds, that slug is resolved against the live catalog: exact id, else the first catalog id with prefix `slug-` (`resolve_codex_model_slug` in `src/codex_sdk/discover.rs`). So `codex:gpt-5.6` can become `gpt-5.6-sol`. If listing fails, the original slug is kept (`resolve_model_on_session` in `src/codex_sdk/session_protocol.rs`). Cursor and Pi do not rewrite slugs.

**Claim.** Bracket overrides are not a shared language:

| Backend | Allowed keys | Thinking values | How they are sent |
|---|---|---|---|
| Cursor | Any key (no validation) | n/a at parse time | Whole `[…]` string is appended to the model name sent to the Node bridge (`cursor_bridge_model`) |
| Pi | `thinking` only | `off\|minimal\|low\|medium\|high\|xhigh\|max` | Parsed out and set on `SessionOptions.thinking` |
| Codex | `thinking` and `service` | `low\|medium\|high\|xhigh\|max\|ultra` (no `off`/`minimal`) | `thinking` becomes turn `effort`; `service` becomes `serviceTier` on thread start and turn start |

So `cursor:claude-opus-5[effort=high,fast=true]` is valid, `pi:openai/gpt-4o[fast=true]` is rejected, and `codex:gpt-5.6[fast=true]` is rejected. `thinking=off` is valid for Pi and invalid for Codex.

Default model is `cursor:auto` (`src/support_paths.rs`, `assets/default_malvin_home_config.toml`).

Cost-rate tables accept `[agent.cursor.<name>]` and `[agent.pi."<provider/model>"]`. The shipped default file only defines Cursor `auto` rates (all zero). There is no shipped `[agent.codex.*]` example.

---

## `--no-force` and tool approval

Help text says `--no-force` is unsupported on all three and fails fast (`src/cli/shared_opts.rs`).

**Claim.** The *place* of the failure differs:

- **Pi and Codex** refuse in Rust before the real session starts (`pi_spawn_bridge`, `codex_spawn_bridge`).
- **Cursor** still spawns Node, then the bridge rejects `noForcePolicy: "fail_fast"` (`cursor-sdk-bridge/src/bridge.ts`). The error text is also different (“Cursor SDK backend” vs “not supported for pi:” / “not supported for codex:”).

All three run headlessly with tools auto-approved when `--no-force` is omitted. Codex also sets `approvalPolicy: "never"` on `thread/start` (`src/codex_sdk/session_protocol.rs`).

---

## Sandbox and tools

**Claim.** Isolation is not the same layer for each backend.

- **Cursor:** tools run inside the Cursor agent / `@cursor/sdk` process tree. Malvin tracks that Node process group.
- **Pi:** most tools are Pi’s defaults. `bash` is replaced with `IsolatedBash`, which malvin spawns itself (`src/pi_sdk/isolated_bash.rs`). Default timeout is two minutes. Other Pi tools are not wrapped that way.
- **Codex:** default sandbox is Codex `workspace-write`. If `MALVIN_CODEX_OUTER_SANDBOX=1` (used in Docker fast tasks), malvin starts Codex with `--dangerously-bypass-approvals-and-sandbox` and `sandbox: danger-full-access` (`src/codex_sdk/session_process.rs`, `session_protocol.rs`). Cursor and Pi have no equivalent env flag.

Pi spawn also requires `io.force` (same as Codex). Cursor spawn does not check `force` in Rust; the Node bridge does.

---

## Event mapping and logs

All three emit `BridgeEvent::{Assistant, Thinking, ToolCall, Usage, RunDone, Fatal}` into one adapter, so stdout and `trace.jsonl` share a shape.

Remaining differences:

1. **Usage keys.** Pi reports `{input, output, cacheRead, cacheWrite}` and then `normalize_pi_usage` aliases them to `inputTokens` / `outputTokens` / … (`src/pi_sdk/map_agent_event_end.rs`, `src/bridge_sdk/timing.rs`). Codex already emits `inputTokens` and maps `cachedInputTokens` → `cacheReadTokens`, and also keeps `reasoningTokens` (`src/codex_sdk/map_event_usage.rs`). Cursor usage comes through the Node bridge already in the ACP-like names. Timing then folds cache-read into `tokens_in` for all backends.

2. **Tool summaries.** Codex rewrites vendor item types (`commandExecution`, `fileChange`, …) into malvin names (`read` / `grep` / `edit` / `shell`) (`src/codex_sdk/map_event_summary.rs`). Pi uses Pi tool names and a smaller summary helper (`src/pi_sdk/map_event_summary.rs`); bash stays “Run …”, it does not classify read vs search vs edit the way Codex does. Cursor summaries are produced in the Node bridge, not in these Rust mappers.

3. **Unsupported Pi events.** `AgentEvent::ExtensionError` becomes `Fatal` with “pi extension event is unsupported” (`src/pi_sdk/map_agent_event.rs`). Cursor/Codex have no matching fatal for “extension”.

4. **Idle timeout labels.** Shared drain helper, different prefixes: “pi sdk timed out” waiting for `agent_end`; “codex timed out” waiting for `turn event` / `rpc reply`; Cursor uses the Node-bridge drain path.

5. **Run duration.** Codex `run_done` can include `durationMs` from the turn or a local timer (`src/codex_sdk/session_turn_done.rs`). Pi sets `duration_ms: None`.

---

## `malvin models`

One command, three listing engines (`src/cli/models_cmd.rs`):

| | Cursor | Pi | Codex |
|---|---|---|---|
| Source | Node `models.js` via the SDK bridge; fallback `agent` / `cursor-agent models` | In-process `ModelRegistry` | Spawn Codex app-server, `model/list` with `includeHidden: true` |
| Timeout env | `MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS` | unused on the list path (`pi_list_models_timeout` exists; `list_pi_models_sync` never calls it) | `MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS` (catalog child) |
| Row format | `cursor:<id>` (and extra columns the SDK/CLI print) | `pi:<provider>/<id>\t<name>\tthinking=yes\|no` | `codex:<id>\t<name>` |
| Extra behavior | Inserts `cursor:auto` if the catalog omitted it | Hides providers without an env API key; prints a note that listing is in-process | Prints an unavailable line on error instead of failing the command |
| Filter | Prefix on the id | Same, and `pi openai gpt-4o` words are joined with `/` | Prefix on the id |

Pi’s listing note says rows are shown only for providers with an environment API key. That matches the filter, and it does **not** match run-time auth (stored credentials allowed).

Codex listing does not print `thinking=` or `service=` hints, even though those overrides exist on the model id.

---

## User experience

What stays the same for an ordinary run:

- Same CLI flags, same prompts, same log directory layout.
- Same stdout stream of assistant text and tool lines (by design).
- Same retry wrapper and “`{backend} SDK prompt failed after N retries`” error envelope.

What a user will notice:

1. **Status.** README marks `pi:` and `codex:` as experimental. `cursor:` is the default and the documented install path (Node ≥ 22.13).

2. **Setup.** Cursor needs a Cursor API key and the bundled Node bridge. Pi needs a provider key or Pi’s own auth file, and a `provider/model` id. Codex needs a working `codex` binary and Codex’s own login; malvin will not say “not authenticated” up front.

3. **Choosing a model.** `malvin models` output and completeness differ (see above). Codex family names may be rewritten to a catalog variant when listing succeeds. Pi ids always contain a slash.

4. **Overrides.** Bracket syntax looks shared but is not. Cursor accepts vendor knobs inside the model string. Pi and Codex parse a small allow-list and send those fields on different protocol fields (`thinking` vs `effort`, plus Codex `service`).

5. **`--no-force`.** All refuse, but Cursor fails after spawning Node; Pi/Codex fail immediately with backend-specific wording.

6. **Session continuity.** Cursor can resume an agent after a 10-minute bridge recycle (`SDK_BRIDGE_MAX_AGE`). Pi and Codex always start a new session / ephemeral thread.

7. **Tool behavior.** A Pi `bash` call is malvin’s isolated shell. A Codex shell call is Codex’s sandbox (or full access in the Docker fast-task path). Cursor tools are whatever `@cursor/sdk` does. The same request can therefore hit different file and network rules.

8. **Cost line.** Token counts are normalized enough to print, but default USD rates are only defined for `cursor:auto`. Other models show 0.0000 unless the user adds a config table.

---

## Abstraction gaps (internal)

These are the places the “one backend” story is thinnest:

1. `SdkSession` is Bridge vs Pi, not Cursor / Pi / Codex.
2. Codex JSON-RPC lives beside Node-bridge send/cancel on `BridgeSession` (`send_prompt` / `shutdown` match on `wire`).
3. Cursor-only resume and busy-agent forgetting (`agent_string_is_cursor_agent_busy`) sit in the shared client.
4. `normalize_pi_usage` is a boolean on the shared spawn/log types rather than a mapper local to Pi.
5. Factory tests cover Cursor and Pi selection, not Codex (`src/agent_backend/factory.rs`).
6. Three model-list implementations, three auth stories, three spawn stacks — only the client methods and `BridgeEvent` are shared.

---

## Severity (operator-facing first)

Higher impact:

- Auth and `malvin models` disagree for Pi (run vs list).
- Codex has no auth preflight.
- Model id language is only partly shared (shape, brackets, slug rewrite).
- Sandbox and bash isolation differ, so the same prompt is not equally constrained.
- Cursor-only resume changes failure/retry behavior after a long run.

Lower impact (mostly logs / internals):

- Usage key aliasing (already papered over for timing).
- Tool summary wording.
- `--no-force` error text and failure stage.
- Pi memory watch using `pgid: 0`.
- Missing default cost rates for Pi and Codex.

---

## Evidence index

- Shared client: `src/agent_backend/{sdk_client.rs,sdk_client_session.rs,sdk_client_prompt.rs,sdk_session.rs,factory.rs}`
- Shared stream: `src/bridge_protocol.rs`, `src/bridge_sdk/{session.rs,log_adapter.rs,timing.rs}`
- Cursor: `src/cursor_sdk/{session_spawn.rs,auth.rs}`, `cursor-sdk-bridge/src/bridge.ts`
- Pi: `src/pi_sdk/{session_spawn.rs,session.rs,runtime.rs,auth.rs,models_list.rs,providers_list.rs,isolated_bash.rs,map_agent_event.rs}`
- Codex: `src/codex_sdk/{session_spawn.rs,session_process.rs,session_protocol.rs,session_io.rs,discover.rs,map_event.rs,map_event_usage.rs}`
- Models CLI: `src/cli/{models_cmd.rs,models_cmd_cursor.rs,models_cmd_filter.rs}`
- Ids: `src/model_id.rs`, `src/model_id_params.rs`
- Product copy: `README.md` (experimental Pi/Codex), `VISION.md` (Pi should look like Cursor)
