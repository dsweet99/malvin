# Backend connector discrepancies: Cursor, Pi, Codex

This report compares the three agent backends (`cursor:`, `pi:`, `codex:`). It covers the shared abstraction, the three transports, and what a user sees.

Evidence is from the current tree under `src/agent_backend/`, `src/cursor_sdk/`, `src/pi_sdk/`, `src/codex_sdk/`, `src/bridge_sdk/`, `src/cli/models_cmd*.rs`, `src/model_id.rs`, `README.md`, and `VISION.md`.

---

## Shared design (what is the same)

All three backends are selected the same way: `--model` is parsed into `ParsedModel`, then `build_agent_backend` builds one `SdkClient`. `BridgeKind` is derived from `ParsedModel.backend` (`bridge_kind_from_backend` in `src/agent_backend/sdk_client.rs`). A mismatched pair cannot be constructed.

The operator-facing session API is also the same object:

- `ensure_authenticated`
- `ensure_coder_session` / `begin_coder_session` / `end_coder_session`
- `run_coder_prompt`
- retries (`max_acp_retries`), prompt logs, run timing

Streamed assistant text, thinking, tool lines, usage, and `run_done` share one `BridgeEvent` vocabulary (`src/bridge_protocol.rs`) and one log adapter (`src/bridge_sdk/log_adapter.rs`). `VISION.md` says `pi:` logs and stdout should look like `cursor:`.

`run_done.status` is canonicalized (`Finished`, `Error`, and related values). Missing `duration_ms` is filled from the session timer in the shared adapter.

`--no-force` is refused once, before any spawn, with one error string (`reject_no_force` in `src/agent_backend/sdk_client_session.rs`).

---

## Architecture: two session kinds, three transports

**Claim.** The public type is one client. The session object is two kinds, not three.

`AgentBackend` is a type alias for `SdkClient`. There is no trait with three implementations.

`SdkSession` is:

```text
Bridge(Box<BridgeSession>)   // Cursor and Codex
Pi(Box<PiEmbeddedSession>)   // Pi only
```

(`src/agent_backend/sdk_session.rs`)

Codex reuses the child-process session (`BridgeSession`) and switches behavior with `BridgeWire::CodexRpc` versus `BridgeWire::NodeBridge`. Pi never uses that wire. It runs `pi_agent_rust` on a dedicated thread.

This split matches the process model: Cursor and Codex are child processes; Pi is in-process. A third session variant would invent a process that does not exist.

Shared spawn arguments (`BridgeSpawnArgs`) hold cwd, model slug, thinking, io, run dir, and timing. Cursor resume is not on that struct. It is an argument only to `cursor_spawn_bridge`. Codex `service` is not on that struct. It is an argument only to `codex_spawn_bridge`.

---

## Transport and process model

| | Cursor | Pi | Codex |
|---|---|---|---|
| Process | Node child running `cursor-sdk-bridge` | In-process crate + worker thread; no `pi` binary | External `codex app-server --stdio` |
| Install | Built/installed with malvin | Linked crate `pi_agent_rust` | Operator must install Codex; `MALVIN_CODEX` optional |
| Wire | Line JSON `BridgeRequest` / `BridgeEvent` | `pi::sdk::AgentEvent` mapped to `BridgeEvent` | JSON-RPC methods mapped to `BridgeEvent` |
| Session id | Cursor `agentId`; remembered and reused | None | Codex `threadId` stored in `agent_id` (ephemeral thread) |
| Resume | Yes, on later Cursor spawn only | No | No (thread is deleted on shutdown) |
| Memory watch | Process group of the Node child (`pgid: Some(pid)`) | No process group (`pgid: None`) plus spawn PID baseline | Process group of the Codex child (`pgid: Some(pid)`) |

**Claim.** Only Cursor can continue an agent across a bridge restart. `last_agent_id` is saved and passed only into `cursor_spawn_bridge`. Pi and Codex spawn functions do not take a resume id, so they cannot represent resume.

**Claim.** Pi has no child process group. `MemWatchHandles.pgid` is `Option<u32>`. Pi passes `None`. Cursor and Codex pass `Some(child_pid)`. The old `pgid: 0` sentinel is gone.

Pi also has a test-only fake session (`MALVIN_TEST_NO_REAL_AGENT`). Cursor and Codex use mock binaries/scripts instead.

---

## Authentication

**Claim.** Preflight auth is backend-specific because the vendors store credentials differently (`SdkClient::ensure_authenticated`):

- **Cursor:** requires `CURSOR_API_KEY`, `CURSOR_AGENT_API_KEY`, or `AGENT_API_KEY`. `agent login` alone is not enough (`src/cursor_sdk/auth.rs`).
- **Pi:** checks the provider in `pi:<provider>/<model>`. Env keys from Pi’s provider metadata, or a credential already stored in Pi’s auth file, count as authenticated (`src/pi_sdk/auth.rs`). Unknown providers are treated as authenticated.
- **Codex:** binary present plus `OPENAI_API_KEY` or a login in `$CODEX_HOME/auth.json` / `~/.codex/auth.json` (`src/codex_sdk/auth.rs`).

**Claim.** `malvin models` for Pi uses the same auth predicate as a run. `print_pi_models` calls `is_provider_authenticated` (the same function as `ensure_pi_authenticated`). A stored Pi credential lists the provider. Tests lock this in (`src/cli/models_cmd_auth_filter_tests.rs`).

Cursor listing does not filter by whether a key is present. Codex listing talks to a live app-server and will print `(codex models unavailable: …)` if the binary is missing or the catalog call fails.

---

## Model ids and bracket overrides

All ids must be prefixed (`cursor:`, `pi:`, or `codex:`). Unprefixed and legacy prefixes are rejected (`src/model_id.rs`).

**Claim.** The id *shape* differs because the vendors name models differently:

- Cursor: any non-empty slug (`cursor:auto`, `cursor:claude-opus-5`, …).
- Pi: must be `pi:<provider>/<model>` (slash required).
- Codex: any non-empty slug (`codex:gpt-5.6-sol`). The slug sent at `thread/start` is the operator slug. Spawn does not rewrite family names to a catalog variant. Use an id printed by `malvin models`.

**Claim.** Bracket overrides are not a shared language, except for a shared `thinking=` vocabulary on Pi and Codex:

| Backend | Allowed keys | Thinking values | How they are sent |
|---|---|---|---|
| Cursor | Any key (no validation) | n/a at parse time | Whole `[…]` string is appended to the model name sent to the Node bridge |
| Pi | `thinking` only | `off\|minimal\|low\|medium\|high\|xhigh\|max\|ultra` | Parsed out and set on `SessionOptions.thinking` (`ultra` → wire `max`) |
| Codex | `thinking` and `service` | same list as Pi | `thinking` becomes turn `effort` (`off`/`minimal` → `low`); `service` becomes `serviceTier` |

So `cursor:claude-opus-5[effort=high,fast=true]` is valid, `pi:openai/gpt-4o[fast=true]` is rejected, and `codex:gpt-5.6[fast=true]` is rejected. Cursor brackets stay vendor-opaque on purpose: malvin does not invent a Cursor parameter parser.

Default model is `cursor:auto`. Cost-rate tables accept `[agent.cursor.<name>]` and `[agent.pi."<provider/model>"]`. The shipped default file only defines Cursor `auto` rates (all zero). There is no shipped `[agent.codex.*]` example.

---

## `--no-force` and tool approval

Help text says `--no-force` is unsupported on all three and fails fast.

**Claim.** The refusal happens once in `begin_coder_session` (`reject_no_force`) before any backend spawn. The error string is one constant (`NO_FORCE_MSG` in `src/acp/agent_helpers.rs`). Pi and Codex spawn still re-check `io.force` as a local guard using that same constant; they cannot succeed if the shared check was skipped.

All three run headlessly with tools auto-approved when `--no-force` is omitted. Codex also sets `approvalPolicy: "never"` on `thread/start`.

---

## Sandbox and tools

**Claim.** Isolation is not the same layer for each backend, because the vendors own different process trees.

- **Cursor:** tools run inside the Cursor agent / `@cursor/sdk` process tree. Malvin tracks that Node process group.
- **Pi:** most tools are Pi’s defaults. `bash` is replaced with `IsolatedBash`, which malvin spawns itself (`src/pi_sdk/isolated_bash.rs`). Default timeout is two minutes. Other Pi tools are not wrapped that way.
- **Codex:** default sandbox is Codex `workspace-write`. If `MALVIN_CODEX_OUTER_SANDBOX=1` (used in Docker fast tasks), malvin starts Codex with `--dangerously-bypass-approvals-and-sandbox` and `sandbox: danger-full-access`. Cursor and Pi have no equivalent env flag.

Unifying those three vendor sandboxes into one wrapper would require replacing Cursor tools and Codex’s own sandbox. That is outside the connector layer. The remaining design constraint is: malvin does not add a fourth, overlapping sandbox on top of a vendor that already isolates.

---

## Event mapping and logs

All three emit `BridgeEvent::{Assistant, Thinking, ToolCall, Usage, RunDone, Fatal}` into one adapter, so stdout and `trace.jsonl` share a shape.

Remaining differences that are still representable:

1. **Usage extras.** Pi and Cursor usage fold into ACP names (`inputTokens` / `outputTokens` / `cacheReadTokens` / `cacheWriteTokens`) at the source. Codex also keeps `reasoningTokens`. Timing folds cache-read into `tokens_in` for all backends.

2. **Tool summaries.** Codex rewrites vendor item types into malvin names (`read` / `grep` / `edit` / `shell`). Pi uses Pi tool names and a smaller summary helper; bash stays “Run …”. Cursor summaries are produced in the Node bridge.

3. **Unsupported Pi events.** `AgentEvent::ExtensionError` becomes `Fatal` with “pi extension event is unsupported”. Cursor/Codex have no matching fatal for “extension”.

4. **Idle timeout prefixes.** Shared drain helper. The three prefixes live in one table (`DRAIN_IDLE_PREFIX_BRIDGE` / `_PI` / `_CODEX` in `src/acp/agent_helpers.rs`). Emit sites and teardown needles use those constants. A timeout cannot miss session recycle because the emitted prefix is the teardown needle.

These remaining items are vendor event shapes and wording, not two predicates for the same fact.

---

## `malvin models`

One command, three listing engines (`src/cli/models_cmd.rs`):

| | Cursor | Pi | Codex |
|---|---|---|---|
| Source | Node `models.js` via the SDK bridge; fallback `agent` / `cursor-agent models` | In-process `ModelRegistry` | Spawn Codex app-server, `model/list` with `includeHidden: true` |
| Timeout env | `MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS` | unused on the list path | `MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS` |
| Row format | `cursor:<id>` (and extra columns the SDK/CLI print) | `pi:<provider>/<id>\t<name>\tthinking=yes\|no` | `codex:<id>\t<name>` plus optional thinking/service hints |
| Extra behavior | Inserts `cursor:auto` if the catalog omitted it | Hides providers you cannot run (env key or stored credential) | Prints an unavailable line on error instead of failing the command |
| Filter | Prefix on the id | Same, and `pi openai gpt-4o` words are joined with `/` | Prefix on the id |

The printed Codex id is the run id. There is no second, rewritten name at spawn.

---

## User experience

What stays the same for an ordinary run:

- Same CLI flags, same prompts, same log directory layout.
- Same stdout stream of assistant text and tool lines (by design).
- Same retry wrapper and “`{backend} SDK prompt failed after N retries`” error envelope.
- Same `--no-force` refusal, before spawn.

What a user will notice:

1. **Status.** README marks `pi:` and `codex:` as experimental. `cursor:` is the default and the documented install path (Node ≥ 22.13).

2. **Setup.** Cursor needs a Cursor API key and the bundled Node bridge. Pi needs a provider key or Pi’s own auth file, and a `provider/model` id. Codex needs a working `codex` binary and Codex login (checked up front).

3. **Choosing a model.** `malvin models` output and completeness differ (see above). Use a printed Codex id. Pi ids always contain a slash.

4. **Overrides.** Bracket syntax looks shared but is not, except `thinking=` on Pi and Codex. Cursor accepts vendor knobs inside the model string.

5. **Session continuity.** Cursor can resume an agent after a 10-minute bridge recycle (`SDK_BRIDGE_MAX_AGE`). Pi and Codex always start a new session / ephemeral thread.

6. **Tool behavior.** A Pi `bash` call is malvin’s isolated shell. A Codex shell call is Codex’s sandbox (or full access in the Docker fast-task path). Cursor tools are whatever `@cursor/sdk` does. The same request can therefore hit different file and network rules.

7. **Cost line.** Token counts are normalized enough to print, but default USD rates are only defined for `cursor:auto`. Other models show 0.0000 unless the user adds a config table.

---

## Abstraction gaps that remain on purpose

1. `SdkSession` is Bridge vs Pi, matching child-process vs in-process.
2. Codex JSON-RPC lives beside Node-bridge send/cancel on `BridgeSession` (`send_prompt` / `shutdown` match on `wire`).
3. Cursor-only resume lives on the Cursor spawn path, not on shared spawn args. Codex-only `service` lives on the Codex spawn path (`BridgeSession.service`), not on `BridgeSpawnArgs` or `StreamLog`.
4. Three model-list implementations and three auth stores — the vendors do not share a catalog or a credential file.
5. Three sandbox layers — the vendors isolate tools themselves.

---

## Severity (operator-facing first)

Resolved by construction (cannot represent the old mismatch):

- Pi list vs run auth (one predicate).
- Independent `BridgeKind` vs `ModelBackend`.
- Distinct Pi/Codex `thinking=` allow-lists at parse time.
- Pi usage keys that needed a shared `normalize_pi_usage` flag.
- `--no-force` failing at different stages with different strings.
- Codex with no auth preflight.
- Codex spawn rewriting the printed model id.
- Shared spawn args carrying a silently ignored `resume_agent_id`.
- Shared spawn args / `StreamLog` carrying a silently ignored Codex `service`.
- Memory-watch `pgid: 0` sentinel.
- Distinct `--no-force` strings in Pi/Codex spawn guards.
- Pi idle timeout emitted as `pi sdk timed out` while teardown matched only `pi rpc timed out`.

Still different, and expected:

- Vendor id shape, Cursor-opaque brackets, three sandboxes, Cursor-only resume capability, default cost-rate table only for `cursor:auto`, tool-summary wording. Idle-timeout prefixes still name the transport (`bridge` / `pi rpc` / `codex`) but they are one table, used for both emit and teardown.

---

## Evidence index

- Shared client: `src/agent_backend/{sdk_client.rs,sdk_client_session.rs,sdk_client_prompt.rs,sdk_session.rs,factory.rs}`
- Shared stream: `src/bridge_protocol.rs`, `src/bridge_sdk/{session.rs,log_adapter.rs,timing.rs}`
- Cursor: `src/cursor_sdk/{session_spawn.rs,auth.rs}`, `cursor-sdk-bridge/src/bridge.ts`
- Pi: `src/pi_sdk/{session_spawn.rs,session.rs,runtime.rs,auth.rs,models_list.rs,isolated_bash.rs,map_agent_event.rs}`
- Codex: `src/codex_sdk/{session_spawn.rs,session_process.rs,session_protocol.rs,session_io.rs,discover.rs,auth.rs,map_event.rs,map_event_usage.rs}`
- Models CLI: `src/cli/{models_cmd.rs,models_cmd_cursor.rs,models_cmd_filter.rs}`
- Ids: `src/model_id.rs`, `src/model_id_params.rs`
- Product copy: `README.md` (experimental Pi/Codex), `VISION.md` (Pi should look like Cursor)
