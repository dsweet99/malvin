# Plan: link `pi:` to `pi::sdk` in-process

## Goal

Stop treating `pi:` as an external CLI. Add the published `pi_agent_rust` crate and drive sessions through `pi::sdk` inside the malvin process.

User-visible `pi:` ids, logs, and stdout stay the same. `VISION.md` still applies: `pi:` output should look like `cursor:` output.

This document is a plan only. It does not change code.

Do **not** build the local `pi_agent_rust` checkout. Depend on the crates.io crate. Cargo will compile that crate as a normal dependency of malvin.

## Current path (what we would replace)

Today `pi:` is a subprocess RPC bridge, not a Rust SDK:

- `SdkClient` with `BridgeKind::Pi` builds a `BridgeSession` (`src/agent_backend/`).
- Spawn lives in `src/pi_sdk/session_spawn.rs`: resolve `MALVIN_PI` or `pi` on `PATH`, check version `>= 0.1.23`, then run:

  `pi --rpc --provider <p> --model <m> [--thinking <level>] --no-session --no-extensions`

- Wire protocol is line-delimited JSON (`prompt`, `new_session`, `abort`) in `src/pi_sdk/protocol.rs` and `session_io.rs`.
- Events are mapped to `BridgeEvent` in `src/pi_sdk/map_event.rs`, then fed to the shared log adapter so stdout / `trace.jsonl` / token timing look like the other backends.
- `malvin models` shells out to `pi --list-models` and `pi --list-providers` and parses ASCII tables.
- Auth is “binary exists” plus a hard-coded env-key map (`OPENAI_API_KEY`, and so on).
- Tests fake the binary with `src/pi_sdk/mock_pi.sh`.

Cursor and Codex stay on their own child-process bridges. This plan only changes `pi:`.

## Target path

Add the published library crate and use `pi::sdk`:

```toml
pi = { package = "pi_agent_rust", version = "0.1.23", default-features = false, features = ["sqlite-sessions"] }
```

That is the current crates.io release (`cargo search` / `cargo info`). Its library name is `pi`. The crate’s own `rust-version` is `1.85`, but that is **not** enough: the default graph of `0.1.23` pulls `sysinfo 0.39.6` (via `asupersync ^0.3.9`) and `kstring 2.0.4`. Those need rustc **1.95** / **1.96**, and `sysinfo 0.39.6` actually uses `cfg_select` (rustc 1.93 fails with `E0658`, not just a Cargo metadata gate). Pinning older `asupersync` cannot help: every release that satisfies `^0.3.9` depends on `sysinfo 0.39`.

**Required:** bump malvin’s toolchain to rustc **1.96** before linking. Isolated `cargo +1.96.0 check` of a consumer with `default-features = false, features = ["sqlite-sessions"]` succeeds. Do not stay on 1.93.

Do **not** path-depend on `/home/dsweet/Projects/repos/pi_agent_rust` (that tree is unpublished `0.2.0`, `rust-version = "1.95"`). Do **not** add a git dependency unless 0.1.23 is missing an API we cannot work around.

Stable session API on `0.1.23` (from the published `src/sdk.rs`):

- `pi::sdk::create_agent_session(SessionOptions) -> AgentSessionHandle`
- `AgentSessionHandle::prompt` / `prompt_with_abort`
- `AgentEvent` (typed; same shapes we currently decode from JSON)
- `SessionOptions`: `provider`, `model`, `thinking`, `working_directory`, `no_session` (default `true`), `enabled_tools`, `extension_paths`, `tool_factory`
- `pi::sdk::ModelRegistry` / `ModelEntry` for `malvin models`
- `pi::sdk::Config::auth_path()` for the auth file location
- `pi::sdk::BUILTIN_TOOL_NAMES` and `default_tool_registry` for isolation wrappers

Do **not** use `SessionTransport::rpc_subprocess`. That is the current design with extra wrapping.

Do **not** enable Pi’s `tui` or `jemalloc` features. `tui` is in the crate’s default feature set and pulls the terminal stack. `jemalloc` installs a process-wide allocator inside the `pi` lib. Default features must stay off.

## Constraints that decide the design

### 1. Crate version, not local source

Claim: crates.io `pi_agent_rust` is `0.1.23`, not `0.2.0`. Evidence: `cargo search pi_agent_rust` and `cargo info pi_agent_rust` on this host (2026-08-21).

Claim: `0.1.23` already exports the in-process SDK we need (`create_agent_session`, `AgentEvent`, `SessionOptions`, `ModelRegistry`). Evidence: published crate `src/sdk.rs` and `src/lib.rs` (`pub mod sdk`; `name = "pi"`).

Claim: a rustc **1.96** bump **is** required. Evidence: isolated consumer on rustc 1.93 fails resolution (`kstring 2.0.4` / `sysinfo 0.39.6`) and `--ignore-rust-version` still fails compiling `sysinfo` (`cfg_select`); same consumer `cargo +1.96.0 check` succeeds.

Hypothesis: compile-time and binary-size cost of linking `0.1.23` with `default-features = false, features = ["sqlite-sessions"]` is large. Measure in Phase 0 before merging further work.

If a later published release (`0.2.x`) is needed for a missing API, bump the version pin then. Do not compile unpublished source to get there.

### 2. Two async runtimes

Malvin is tokio. Pi’s SDK is asupersync (`create_agent_session` and `prompt` are asupersync futures). Pi’s own examples `block_on` an asupersync runtime.

Do not poll asupersync futures on the tokio executor. Do not make malvin’s whole agent loop asupersync.

**Chosen shape:** a small adapter owns one asupersync runtime on a dedicated thread. Malvin’s tokio code talks to it over channels / oneshot results.

```text
tokio (malvin SdkClient)
    |  spawn session / send prompt / abort / shutdown
    v
PiRuntime (thread + asupersync Runtime)
    |  create_agent_session / prompt_with_abort
    v
AgentEvent callback  --mpsc-->  map_pi_agent_event  -->  BridgeEvent
                                                      -->  existing log adapter
```

Idle-timeout and cancel stay on the tokio side: if the drain idle timer fires, call `AbortHandle::abort()` on the Pi side, then fail the prompt the same way RPC timeout fails today.

### 3. Process isolation and memory limits

This is the largest product risk.

Today the `pi` child is in its own process group. Malvin:

- sets `MALLOC_ARENA_MAX=2` on the child
- watches the child’s RSS and kills the group on limit
- kills the group on shutdown / parent death
- refuses a second spawn until the previous group is dead

In-process Pi runs tools (`bash`, `write`, …) inside the **malvin** process. A runaway bash then counts against malvin’s own sandbox budget. `start_mem_watch` as written needs a child pgid and will not cover this.

**Required before calling the backend “done”:**

- Decide an isolation policy and implement it. Preferred order:
  1. Keep Pi’s built-in tools, but wrap `bash` (and any other process-spawning tool) so those children still go through `malvin_tokio_command` / process-group isolation / parent-death signal. `SessionOptions::tool_factory` + `default_tool_registry` exist for this.
  2. Watch **malvin’s** process-group RSS for `pi:` sessions (same limit file as today), and abort the session if it trips.
  3. Document that `pi:` no longer has a separate agent process; remaining Cursor/Codex isolation is unchanged.
- `--no-force` still fails fast. Pi has no interactive approval in this mode.
- Extensions stay off (`extension_paths` empty), matching `--no-extensions`. If an extension UI event ever appears, treat it as a hard error the same way `map_event.rs` treats `extension_ui_request` today.

Hypothesis: wrapping only `bash` is enough for the current default tool set, because `read` / `edit` / `write` / `grep` / `find` / `ls` / `hashline_edit` do not spawn shells. Confirm against `pi::sdk::BUILTIN_TOOL_NAMES` at implement time.

### 4. Log parity

Map typed `AgentEvent` values to the same `BridgeEvent`s `map_event.rs` already produces from JSON:

| Pi event | Malvin `BridgeEvent` |
| --- | --- |
| `MessageUpdate` text delta | `Assistant` |
| `MessageUpdate` thinking delta | `Thinking` |
| `ToolExecutionStart` / `Update` / `End` | `ToolCall` phases `start` / `update` / `complete` or `error` |
| `AgentEnd` | `RunDone` (text + usage aggregated from `messages`, same as today’s JSON `agent_end`) |

Reuse `handle_stream_event`, tool summaries (`map_event_summary.rs`), and `record_sdk_usage(..., normalize_pi_usage: true)`. Do not invent a second log path.

Keep prefixed model ids (`pi:openai/gpt-4o`) for cost lookup.

`0.1.23` `AgentEvent` has no `extension_ui_request` variant. With extensions off, that path should stay unused. Still fail closed on any unexpected control event that would have been fatal on the RPC path.

### 5. Auth types vs the `pi::sdk` surface

`create_agent_session` already loads Pi’s auth file (`AuthStorage::load_async(Config::auth_path())`) and refreshes OAuth. A live prompt therefore sees env keys **and** stored `~/.pi` credentials without extra work.

`malvin models` needs a registry. `pi::sdk` re-exports `ModelRegistry` and `Config`, but **does not** re-export `AuthStorage` on `0.1.23`. `ModelRegistry::load` / `load_for_listing` take `&AuthStorage`.

**Chosen workaround:** import `pi::auth::AuthStorage` for listing and pre-flight checks only. The published crate’s `lib.rs` has `pub mod auth`. Session create/prompt stay on `pi::sdk`. If a later crate re-exports `AuthStorage` from `pi::sdk`, switch the import.

`ensure_pi_authenticated` should accept either env keys **or** credentials already stored by Pi. Today we miss the second case and also require the `pi` binary to exist.

## Architecture after the change

Keep `BridgeKind::Pi` and `SdkClient` as the malvin-facing API (`ensure_coder_session`, `run_coder_prompt`, retries, prompt logs). Change only the session object behind `BridgeKind::Pi`.

`BridgeSession` is shaped around `tokio::process::Child` stdin/stdout. Do not stretch it to hold an in-process handle.

Introduce something like:

- `src/pi_sdk/runtime.rs` — asupersync thread, session create/prompt/abort/drop
- `src/pi_sdk/session.rs` — `PiEmbeddedSession` (timing, last response, run dir, abort handle)
- `src/pi_sdk/map_event.rs` — rewrite to take `&AgentEvent` (keep JSON mapping only if we still want a golden-file test against old RPC fixtures)
- `src/agent_backend/sdk_client.rs` — `session` becomes an enum or `Pi` stops using `BridgeSession`

Cursor and Codex `BridgeSession` paths stay as they are.

`SDK_BRIDGE_MAX_AGE` (10 minutes) can still restart a long-lived `pi:` session. Restart means drop the handle and `create_agent_session` again, not kill a child.

## Work in order

### Phase 0 — bump rustc, add the crate, prove malvin still builds

Do this **before** adding the crate. Isolated `cargo check` of `pi_agent_rust 0.1.23` fails on rustc 1.93 and succeeds on rustc 1.96.0 (see Target path).

1. Pin the compiler, both places malvin currently declares 1.87 / inherits the host default:
   - Set `package.rust-version = "1.96"` in `Cargo.toml` (today: `"1.87"`).
   - Add a repo-root `rust-toolchain.toml` with `channel = "1.96.0"` so humans and CI actually run rustc 1.96, not whatever `rustup default` is. Changing only `rust-version` does **not** switch the compiler.
2. Add the `pi` dependency as above (`default-features = false`, `features = ["sqlite-sessions"]`). Keep `sqlite-sessions` on for the session-store `cfg`s; it does **not** change the crate graph (see Settled).
3. Add a tiny compile-only use of `pi::sdk::SessionOptions` so CI fails if the crate does not build.
4. `cargo +1.96.0 check` / `cargo +1.96.0 test` of **malvin** (this compiles the crates.io crate as a dependency; it does not build the local Pi repo). Isolated-consumer success is not enough: malvin is edition 2024, has its own lockfile, and this host’s 4 GiB sandbox already OOM’d a mid-build `target/` (~1.2 GiB). Expect a large compile; use low `-j` if memory-capped.
5. Measure compile-time and binary-size delta. Report numbers before merging further work.

If Phase 0 fails, write the blocker in this file and stop. Do not fall back to compiling unpublished Pi source. Do not revert the rustc pin to “make 1.93 work.”

### Phase 1 — event mapping without a live model

1. Rewrite `map_pi_event` (or add `map_pi_agent_event`) against `pi::sdk::AgentEvent`.
2. Port existing `map_event_tests.rs` cases to constructed `AgentEvent` values.
3. Keep usage aggregation and last-assistant-text extraction equivalent to today’s `agent_end` JSON logic.

No network, no Pi runtime thread yet.

### Phase 2 — runtime adapter

1. Start one asupersync runtime thread per open `pi:` session (or one shared thread with many sessions; start with one thread per session, simpler shutdown).
2. `create_agent_session` with:
   - `provider` / `model` from `pi:<provider>/<model>`
   - `thinking` from the existing `[thinking=...]` param (`ThinkingLevel`)
   - `working_directory` = session cwd
   - `no_session: true` (already the `SessionOptions` default)
   - empty `extension_paths`
   - `enabled_tools: None` (all built-ins) unless isolation requires a subset
3. `prompt` streams events over an `mpsc` into the existing drain / idle-timeout loop.
4. Shutdown drops the handle and joins the thread. Abort uses `AbortHandle`.
5. `--no-force` still errors in spawn, same message as today.

### Phase 3 — isolation and memory

Implement the policy in constraint 3. Wire mem-watch to whatever process set we actually own. Extend sandbox tests that currently assume a child pgid.

### Phase 4 — models and auth without the binary

Replace:

- `list_pi_models_sync` (`pi --list-models`) with `AuthStorage` + `ModelRegistry::load` / `load_for_listing`
- `list_pi_provider_auth_sync` (`pi --list-providers`) with credential status from `AuthStorage` plus the existing env-key map

Keep the printed line format:

```text
pi:<provider>/<model>	<name>	thinking=yes|no
```

and the “cached list” note if it is still true.

Delete `resolve_pi_bin` / `MALVIN_PI` / version check once nothing calls them. Update the missing-binary error to an auth / config error.

### Phase 5 — tests, docs, cleanup

- Replace `mock_pi.sh` client tests with a fake `PiEmbeddedSession` that injects `AgentEvent`s (same coverage: usage, last response, `--no-force`).
- Keep a single optional live test behind an env flag (`MALVIN_PI_LIVE=1`) if we want a smoke prompt. Default gates must stay offline and under 1.5s per unit test (`VISION.md`).
- Drop RPC encode/decode if unused. Keep it only if a test still needs the old fixtures.
- README: remove “requires an externally installed `pi` binary; see `design.md`”. `design.md` is already missing. Say malvin links `pi_agent_rust` and uses the operator’s Pi auth/config.
- `default_prompts/docs/malvin.md` and `default_prompts/docs/models.md`: same edit (`PATH` / `MALVIN_PI` / “malvin does not bundle Pi”).
- Help / `--doc` strings that mention `MALVIN_PI` or “install pi” need the same edit.
- `ops/fast_task.py` currently bind-mounts the host `pi` binary for `pi:` models; stop requiring that once the crate is linked.

## What must not change

- Model id grammar: `pi:<provider>/<model>` and `[thinking=...]`.
- Default workflow, `header.md` regularization language (`VISION.md`).
- Cursor and Codex backends.
- Shared log adapter and “looks like cursor:” requirement.
- Unit tests writing production config files (none should start).
- Auto-approval: malvin still does not ask.

## Acceptance

1. `malvin --model=pi:<provider>/<model> --do Hello` runs with no `pi` binary on `PATH` and no `MALVIN_PI`.
2. stdout / `trace.jsonl` / token timing still look like a `cursor:` run (tool lines, assistant text, usage).
3. `malvin models pi:` lists models from the crate registry and hides providers with no env key and no stored Pi credential.
4. `--no-force` fails before any session is created.
5. Session shutdown does not leave extra threads or leaked asupersync runtimes (test with a begin/end pair).
6. Memory-limit behavior is defined and tested for the in-process case.
7. Default gates (`ruff`, `kiss`, clippy, pytest, rust test gate) pass.
8. No unit test talks to the network or requires a real provider key.
9. Malvin does not depend on a path or git checkout of `pi_agent_rust`.

## Non-goals

- Embedding Cursor or Codex the same way.
- Loading Pi extensions or enabling extension UI handlers.
- Replacing tokio in malvin.
- Vendoring Pi’s TUI.
- Changing `pi:` model id syntax.
- Supporting `--rpc` fallback once the in-process path works.
- Compiling or vendoring unpublished Pi source (`0.2.0` local tree).

## Open questions (answer in Phase 0 / 3)

1. **Binary size / build time:** if the delta is unacceptable, consider a `pi-embedded` cargo feature so `cursor:`-only builds stay slim. Default for this project should still enable it, because `pi:` is a first-class backend.
2. **Isolation:** wrap `bash` only, or also restrict write paths to the work dir? Pi tools already take a cwd; confirm they cannot escape it.
3. **Auth file path:** follow Pi’s default (`Config::auth_path()`), or allow a malvin override? Default to Pi’s path so existing `pi` logins work.

## Settled (do not re-open in Phase 0)

- **`sqlite-sessions`:** keep it on. It is an **empty** Cargo feature (`sqlite-sessions = []`) that only `cfg`-gates session-store code. `sqlmodel-core` / `sqlmodel-sqlite` are **unconditional** deps of `0.1.23`. Dropping the feature will not shrink compile time or binary size. Leave it enabled so a user `session_store=sqlite` config gets a real backend instead of the crate’s JSONL fallback warning.
- **rustc:** bump to **1.96** (`Cargo.toml` `rust-version` + `rust-toolchain.toml`). Do not try to stay on 1.93.

## Suggested first implementation slice

After Phase 0 compiles:

- adapter + event map + `--do` smoke on a cheap model
- then models/auth
- then isolation/mem-watch
- then delete the RPC spawn path

Do not delete `MALVIN_PI` until the in-process path has passed a real `--do` and `malvin models pi:`.
