# malvin (top-level CLI)

malvin is a non-interactive research and coding agent. It runs agent sessions against a workspace through the Cursor SDK (`cursor:` models via a Node bridge to `@cursor/sdk`), the official TypeScript/npm Pi agent (`pi:` models via RPC) or a local Codex app-server (`codex:` models via `codex app-server`). Each agent-backed invocation creates an isolated run directory under `~/.malvinconf/logs/<hash>/` and records prompts, stdout, and artifacts there. When the workspace root contains a non-empty `AGENTS.md`, malvin embeds it in `header.md` via `{{ agents_insert }}` so every fresh-header session sees that guidance without relying on Cursor rule auto-load.

## How to read this documentation

- **Humans:** skim **Commands**, then open `malvin <COMMAND> --doc` for the workflow you need.
- **Agents:** treat each `--doc` file as a self-contained contract for that command; global flags and run-directory rules live in this file.
- **Help vs doc:** `malvin --help` lists flags; `--doc` explains behavior, logs, and when to use each command.

## Usage

```text
malvin [OPTION]... [REQUEST]...
   or: malvin [OPTION]... <COMMAND>
```

These forms are mutually exclusive: pass request(s) **or** a subcommand, not both on one synopsis line. `malvin --help` uses the same two-line usage.

Bare `malvin REQUEST` runs autonomous routing once (`router_a`, then `router_a_2`, optional `router_b`, then `router_summarize`). A lone-line `__MALVIN_DONE__` in the `router_a_2` reply skips `router_b`. `router_a_2.md` asks for that marker only on an impasse. Multiple `REQUEST` arguments each run as an independent default-route invocation (new run directory, one router pass, summarize). With no request and no subcommand, malvin prints a short command catalog and exits 0. `malvin -g` without a request runs the gate-fix workflow (fixed request `Get the gates to pass.` with `--gates` on). Each `--do` applies only to the `REQUEST` that immediately follows it (one-shot turn); other `REQUEST` args still use the router. Each `--creative[=PROB]` likewise applies only to the `REQUEST` that immediately follows it (repeatable; other `REQUEST` args stay non-creative). The `admin` subcommand covers operator maintenance; `malvin admin` alone likewise prints its command catalog and exits 0. Omitting `REQUEST` after a lone `--do` prints short usage and exits 0.

## Commands

| Command | Purpose |
|---------|---------|
| *(default)* | Bare `malvin REQUEST` — one outer iteration starts a new agent for aggregated `header` + optional `mbc2` + `router_a`, then `router_a_2`, then optional `router_b` on that agent; exit `router_summarize` on that same agent. There is no `--max-loops` flag (`DEFAULT_MAX_LOOPS` is 1) |
| `--do` | One-shot agent turn for the following REQUEST (repeatable; other REQUESTs stay on the router) |
| `--creative[=PROB]` | Creative mode for the following REQUEST only (repeatable; optional probability, default `1.0`) |
| `malvin -g` | Fix quality gates via the default router with fixed request `Get the gates to pass.` (no positional request) |
| `admin` | Operator maintenance (`models`, `remotes`, `reset-herdr`/`rh`) |

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`); for the one-shot workflow use `malvin --do --doc`. The default-route contract (`router.md`) is printed after this overview when you run `malvin --doc`.

## Global options

`--doc` is a true global: it may appear before or after any subcommand, including `admin`.

Agent-session flags (`--model`, `--gates`, `-q`, `-v`, `--creative[=PROB]`, `--max-acp-retries`, `--ml=N`, …) apply to bare `malvin REQUEST` and `--do` (`--watch` applies only to bare `malvin REQUEST` and `malvin -g`). The `admin` help listing omits them; pass `--model` before `admin models` only when you want to set that command’s `Current:` footer.

### `--remote=PROVIDER:SERVICE[KEY=VALUE,...]`

Run the invocation on a remote machine instead of on this one, then apply its file changes and copy its run logs back. Applies to bare `malvin REQUEST`, `--do`, and `malvin -g`. A remote is written `PROVIDER:SERVICE[KEY=VALUE,...]`, the same shape as a `--model` id (`provider:model[key=value,...]`). `malvin admin remotes` lists the remotes and their suboptions, much as `malvin admin models` lists the ids `--model` accepts. The only remote is `modal:sandbox` (provider `modal`, service `sandbox`), which runs in a Modal Sandbox: `--remote=modal:sandbox` (or `--remote modal:sandbox`) uses default resources, and `--remote=modal:sandbox[gpu=...,ncpu=...,mem=...,timeout=...]` chooses the Sandbox's GPU, CPU count, memory, and lifetime for this run; each suboption is optional and the order does not matter (for example `--remote=modal:sandbox[timeout=2h,gpu=A100]`). A name listed in `[aliases.remotes]` in `~/.malvinconf/config.toml` expands to its value first (see **Home config**). Any other remote exits 1, including a bare `modal` without its service. See **Running on Modal** below for the suboptions, setup, what is uploaded, how results return, and what is rejected.


### `-q` / `--quiet`

On the **default router** (bare `malvin REQUEST` and `malvin -g`), print only the text between `__MALVIN_DM_START__` and `__MALVIN_DM_END__` fences to process stdout. Startup chrome, agent stream, heartbeats, prompt-name lines, and fence markers are omitted from stdout. Run-dir logs and stderr are unchanged. The process still prints one `TIMING` line and one `COST` line at the end (see session footnotes).

Plain `malvin --do` is already DM-body-only on stdout without `--verbose`, and that mode omits the `TIMING` and `COST` lines. With `--verbose`, `--do` tees the same live agent log classes as the default workflow and prints the footnote pair (see `-v` / `--verbose` below).

### `--model <MODEL>`

Model id for agent-backed commands. Default: `cursor:auto` (or `[agent].model` in `~/.malvinconf/config.toml`). Prefixes: `cursor:` for the Cursor SDK backend; `pi:<provider>/<model>` for the official TypeScript/npm Pi agent (RPC; uses env keys or credentials already stored by Pi; keyless locals are `pi:local/<provider>/<model>`); `codex:<model>` for a local Codex app-server. An unprefixed name is looked up in `[aliases.models]` in the home config. Optional bracket overrides select thinking / speed where the backend supports them, for example `cursor:claude-opus-5[effort=high,fast=true]` or `pi:openai/gpt-5[thinking=high]` (see `malvin admin models --doc`). Legacy `prime:`, `mini:`, and `rpi:` ids are rejected.

### Outer router iteration

Bare `malvin REQUEST` and `malvin -g` run the outer router once. `DEFAULT_MAX_LOOPS` is 1. Passing `--max-loops` is an unexpected-argument error.

### `-g` / `--gates`

Inject workspace check command text into agent prompts and, when `router_a_2` emits `__MALVIN_DONE__`, run workspace gates as an exit check. Off by default. When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first (default router with request from `init_constraints.md`, harness gates off) to discover and write `.malvin/gates`. On bare `malvin REQUEST`, `-g` / `--gates` runs workspace `.malvin/gates` after that marker: pass ends the run successfully after `router_summarize`; fail ends the run with a workspace gate error after `router_summarize`. The outer router does not start another iteration. `malvin -g` without a request runs the gate-fix workflow with this flag on and fixed request `Get the gates to pass.` When work runs, check text is still injected into the work prompt. Agent prompts may still include available `.malvin/gates` guidance when this option is off.



### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown. For `malvin --do`, also unlock the same live agent stdout log classes as the default workflow (thought tokens and narrative tee) and print the `TIMING` and `COST` footnotes. Without `--verbose`, `--do` stays DM-body-only and omits those footnotes.

### `--max-acp-retries <N>` (default: 3)

Stop after N consecutive identical backend errors (spawn, header, or prompt), with 1s / 3s backoff between tries. When the flag is omitted, `[agent].max_acp_retries` from `~/.malvinconf/config.toml` is used. Distinct errors reset the consecutive counter, and so does a gap of more than 60 seconds since the previous error, so a rare error that recurs over a long job does not end it. Among successes, only a successful prompt turn clears it; a successful respawn or header delivery does not. For keyless local providers (`pi:local`, `pi:ollama`, and similar), malvin also stops after 10 backend errors or 5 minutes without a successful turn, even when the errors differ. Fail-fast classes (billing, usage limit, invalid model, and similar) still exit immediately.

### `--creative[=PROB]`

On the default router (bare `malvin REQUEST` and `malvin -g`), when creative mode is sampled for an outer iteration: include `mbc2.md` in the aggregated initial prompt (after the header), and fill `{{ creative_lead }}` in `router_b.md` for the optional work turn. `{{ satisfy_line }}` stays `router_b_satisfy.md` on every work turn. Both creative changes share one Bernoulli draw per outer iteration. `--creative` alone uses probability `1.0`; `--creative=0.6` uses `0.6`. Off by default.

Like `--do`, each `--creative` applies only to the `REQUEST` that immediately follows it and may be repeated (at most once per `REQUEST`). Example: `malvin "plain" --creative "spark" "plain2" --creative=0.4 "spark2"`. Intervening global flags (for example `--quiet`) may appear between `--creative` and its `REQUEST`. A trailing `--creative` after other requests, or `--creative` immediately followed by `--do` (or the reverse), is an error. For `malvin -g` with no positional request, `--creative` still enables creative sampling for that gates-only run.

### `--watch`

On the default router (bare `malvin REQUEST` and `malvin -g`), before the single outer iteration, re-copy the operator's request `.md` file onto the run's `plan_*.md` artifact (overwrite). No effect when `REQUEST` is literal text (not an existing `.md` path). Has no effect on `--do` requests; when the invocation is pure `--do` (no router REQUEST), `--watch` is rejected.

### `--ml=N`

Run the meta-loop N times. After every REQUEST in the invocation has run once (preserving `--do` vs router tagging and order), start again from the first REQUEST until the sequence has run N times — as if the same command line were re-invoked. `N` is a positive integer. `N=inf` repeats forever. The default is `1` (a single pass). A failing REQUEST stops the process (the loop does not continue past an error). Init bootstrap, when needed, still runs once at the start of the process. Applies to bare `malvin REQUEST…`, mixed/`--do` request lists, and `malvin -g`.

### Session names

For bare `malvin REQUEST`, `--do`, and `malvin -g`, malvin assigns a unique five-character session id (`[a-z0-9]`) and acquires a session name lock before substantive work.

Malvin registers the top-level process under this id in a per-user registry at `~/.malvinconf/names/<ID>` (one line: holder PID). If another live malvin process already holds the same id, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvinconf/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live agent/bridge session per lock slot in a workspace). Two malvin processes with different session ids may both register names and hold live sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/gates/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--advice`, `--credits`, `--help`, `--version`, and `malvin` with no subcommand do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvinconf/logs/`.

- `malvin --doc` — this overview, then the default-route contract (`router.md`).
- `malvin <COMMAND> --doc` — documentation for that subcommand (`malvin admin --doc` for `admin`, `malvin admin models --doc` for `models`).
- `malvin --do --doc` — documentation for the one-shot `--do` workflow.

Other subcommand arguments (for example `<REQUEST>`) are not required when `--doc` is set. Argument validation still runs first: invalid values or combinations (for example `--model foo:bar`, `-g` or `--remote` with `admin`, `--do` with a subcommand, or `--watch` with pure `--do`) exit 1 with the error instead of printing documentation.

### `--advice`

Print an embedded advice document for `TAG` to stdout and exit, or list available tags when `TAG` is omitted. Does not spawn an agent or create a run directory under `~/.malvinconf/logs/`.

- `malvin --advice` — prints how to open a full document (`malvin --advice TAG`), then a `TAG: Description` heading line, then one `<tag>: <description>` line per TAG (description at most 7 words).
- `malvin --advice TAG` — body of the matching file under `default_prompts/advice/*.md`.
- Each document’s TAG is the filename without `.md`: lowercase letters, digits, and underscores, starting with a letter, at most 32 characters (examples: `doc_design` for `doc_design.md`; `scholar` for `scholar.md`; `report` for `report.md`). Every `.md` file in that directory is listed; rebuild absorbs added or renamed files.
- The first line of each advice file must be `description: ...` (value at most 7 words). The build fails if that line is missing or malformed; the listed description is the value after `description:`.

Other subcommand arguments are not required when `--advice` is set.

### `--credits`

Print credits for the published ideas malvin builds on (embedded from `default_prompts/credits.md`) to stdout and exit. Does not spawn an agent or create a run directory under `~/.malvinconf/logs/`. Other arguments (for example `<REQUEST>`) are ignored when `--credits` is set.

## Quality gates (`.malvin/gates`)

When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first: it renders `init_constraints.md` (cwd as `repo_root_path`) and invokes the **default router** with harness gates off to discover and write `.malvin/gates`.

With `--gates` and an existing `.malvin/gates`, malvin runs workspace quality gates from that file at the repo git root (one shell command per non-empty, non-comment line). Full-line comments starting with `#` are ignored. `malvin -g` without a request always enables this harness.

Other invocations (`--do`, bare `malvin REQUEST`) do not require `.malvin/gates` at startup and may run outside a git repo. With `--gates` on a bare `malvin REQUEST`, malvin runs workspace gates when `router_a_2` emits `__MALVIN_DONE__` and fails the run when they fail (see the default-route section of `malvin --doc`). Without `--gates` (the default for other commands), malvin does not run those checks directly on the default route. `header.md` notes about gates lines remain advisory when a workspace happens to have gates; they are not a startup requirement for those commands.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Run directories and logs

Every agent-backed command creates `~/.malvinconf/logs/<hash>/<timestamp>_<token>/`. Typical files:

| File | Role |
|------|------|
| `plan_<random>.md` or `request.md` | Copy of user input for this run |
| `do.log`, `router_1.log` | `--do` transcript, or the single outer-iteration router transcript (`router_a`, `router_a_2`, optional `router_b`, exit summarize) |
| `stdout.log` | Tee of agent stdout — **narrative** channel |
| `trace.jsonl` | Audit record (sdk-shaped JSONL; Pi and Codex events are mapped into the same shapes) — **authoritative** for semantics (tool results, shrink/fork, LLM usage) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `run_timing.json` | Wall/LLM timing, token/step aggregates, and optional cost |
| `_run/exp_log_*.md` | Experiment / gate-loop logs (`exp_log_<run>.md` scaffold; `exp_log_<run>_g1.md` for the single outer iteration — kept, never truncated) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Session footnotes (`TIMING` / `COST`)

When an agent workflow is about to exit, malvin prints one footnote pair. That print is outside `--ml`: a finite count returns after N passes, `--ml=inf` returns only when a request fails, and the lines are also printed on interrupt. They are not printed at the end of each request. `wall` is the elapsed time of this malvin process. Token counts, step counts, tool time, LLM wait, and dollars are the sum across init, every request, and every meta-loop cycle. Carried totals inside one session are not counted twice. The lines go to process stdout under `-q`, and they are appended to the active `stdout.log` when a run directory is open. Plain `malvin --do` omits them on stdout and in `stdout.log` unless `--verbose` is set; with `--verbose`, `--do` prints the same pair:

```text
TIMING: wall = … llm_wait = … …
COST: steps = N tokens_in = X tokens_out = Y cache_read = A cache_write = B cost_in = … cost_out = … cost_read = … cost_write = … cost_tot = …
```

Durations on the `TIMING` line are seconds. A span of 0 ms prints as `0.0s`. A span from 1 ms through 49 ms prints as `<0.1s`. From 50 ms upward, the value rounds to the nearest tenth of a second (`50` ms is `0.1s`; `23451` ms is `23.5s`).

- **`steps`:** Approximate count of agent steps. Cursor SDK counts SDK `onStep` boundaries; the other backends (`pi:`, `codex:`) derive steps from assistant replies and tool-call batches (one batch of parallel tool calls counts as one step). Raw tool-call counts are not printed as `steps`.
- **`tokens_in` / `tokens_out`:** Numeric when the backend reports usage. Cursor SDK folds one `result.usage` (`TokenUsage`) per `send` into these fields (cache read/write counted in `tokens_in`). When usage is absent, fields stay `n/a`.
- **`cache_read` / `cache_write`:** Separate cache token totals from the same usage objects when reported (`cacheReadTokens` / `cacheWriteTokens`). Still included in `tokens_in`. When absent, fields stay `n/a`.
- **`cost_in` / `cost_out` / `cost_read` / `cost_write` / `cost_tot`:** Estimated USD from per-model rates in `~/.malvinconf/config.toml` × token counts / 1e6. Rates are dollars per million tokens (`usd_per_microtoken_*`) under `[agent.<provider>.<name>]` for the run model (e.g. `[agent.cursor.auto]` for `cursor:auto`):
  - `cost_in = usd_per_microtoken_in ×` non-cache input tokens `/ 1_000_000` (stored `tokens_in` minus `cache_read` / `cache_write`)
  - `cost_out = usd_per_microtoken_out × tokens_out / 1_000_000`
  - `cost_read = usd_per_microtoken_cache_read × cache_read / 1_000_000`
  - `cost_write = usd_per_microtoken_cache_write × cache_write / 1_000_000`
  - `cost_tot` = sum of the four components
  All rates default to `0`, so with unset rates the estimate is `0` (shown as `0.0000`), not `n/a`. Set rates for a non-zero estimate. When usage was never observed, cost fields stay `n/a`.

### Narrative vs audit (trust rule)

Each run writes two parallel channels with different contracts:

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, `a|`, `r|`, …); `r|` marks untagged output relayed from a `--remote=modal:sandbox` Sandbox. Use for skimming a run and vocabulary/ordering checks. An `a|<provider>:<model>` line (for example `a|cursor:auto`) is written each time a fresh agent context is started.
- **`trace.jsonl` (audit):** machine-authoritative JSONL (bridge events such as `assistant` / `thinking` / `tool_call` / `progress` / `run_done`). Use for tool results, shrink/fork events, and gate-loop audit tooling.

Consumers must know which file to trust for which question. Named types live in `src/observability/` (`ObservabilityChannel`, `AuditEventKind`).

## SDK drain idle (Cursor / Pi / Codex)

While waiting for the next Cursor SDK bridge, Pi RPC, or Codex app-server line, malvin applies a **per-event** idle budget. There is no total-prompt wall clock: a turn may run as long as the backend keeps emitting events or its sandbox processes keep showing progress. Every backend session must implement the `TurnTimeoutExtension` trait (`src/backends/bridge_sdk/turn_timeout.rs`, no default methods), which reports open tools (`tools_in_flight`).

| Clock | Meaning | Default |
|-------|---------|---------|
| Idle budget | Max silence since the last successful bridge/Pi event (or since the wait started) | `MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS` (600000 ms) |
| Slice | How long to block on one read before sampling sandbox child health | `min(60000 ms, idle remaining)` |
| Health extend | If sandbox PIDs (excluding malvin itself) show CPU / ctxt / thread progress (`StillBusy`), refresh the idle budget; I/O-bound work with open tools is treated like `StillBusy` | — |

Missing a line for the full (possibly health-extended) idle window fails with `bridge timed out … without a bridge event (bridge quiet; …)` — that is the hung/stalled bridge signal (no NDJSON lines, including heartbeats). Local stdout heartbeats (`Starting`, …) do **not** reset drain idle.

The Cursor SDK bridge also emits automatic `{ "event": "progress", "kind": "heartbeat" }` lines when a run is in flight and no SDK message/step has been forwarded for 15s. Those `progress` events reset the per-event idle budget like any other bridge line (they are recorded in `trace.jsonl`, not teed to narrative stdout).

**Differentiation:** continuing heartbeats (or other bridge lines) ⇒ SDK bridge alive, keep waiting; full idle window with no bridge lines ⇒ quiet/hung bridge, fail and tear down. Open tracked tools additionally remap sandbox `AppearsHung` → `StillBusy` as a backup when the event loop cannot heartbeat during I/O-bound work.

**Limitation:** work backgrounded outside the bridge sandbox process group (for example a nested Docker `malvin` after the outer shell tool call has already completed) is not visible to child-health sampling. That case relies on the outer SDK run staying open so automatic `progress` heartbeats (or other bridge events) keep arriving inside the idle budget. Once `run_done` fires, progress stops; further silence still hits idle.

## Home config (`~/.malvinconf/config.toml`)

`~/.malvinconf/` holds malvin's per-user state: `config.toml`, `local_llms.json`, `logs/`, `names/`, and `sdk-bridges/`. Older releases used `~/.malvin_home/`. On startup, when `~/.malvin_home/` is a real directory, malvin renames it to `~/.malvinconf`, or, if `~/.malvinconf` already exists, merges its contents in (files from `~/.malvin_home/` win on conflict). It then leaves a symlink at the old path, so malvin processes still running an older build keep working.

Top-level keys include `mem_limit_gb` and `theme`. Cursor cost rates `usd_per_microtoken_in`, `usd_per_microtoken_out`, `usd_per_microtoken_cache_read`, and `usd_per_microtoken_cache_write` (dollars per million tokens; all default `0`) live under per-model tables such as `[agent.cursor.auto]` (model id `cursor:auto`). Sections include `[agent]`, `[logs]`, and optional `[aliases.models]` (map short unprefixed names to full model ids for `--model` / `[agent].model`, e.g. `astra = "pi:openrouter/openai/gpt-astra"`) and `[aliases.remotes]` (map short names to full `--remote` values, e.g. `big = "modal:sandbox[gpu=A100,mem=32]"`, so `--remote=big` means `--remote=modal:sandbox[gpu=A100,mem=32]`). An alias matches only the whole value: `--remote=big[timeout=2h]` exits 1 with a message saying so, and an alias may not be named after a built-in remote provider such as `modal` or contain `:`, `[`, `]`, `,`, or `=`; model aliases may not contain `:` either. An invalid alias entry is skipped with a warning; the other entries still work. The older `[nicknames]` table was renamed to `[aliases.models]`; malvin rejects a config that still has it, with a message saying to move its entries.

## Local LLMs (`~/.malvinconf/local_llms.json`)

Malvin runs keyless local models through `pi:local/<provider>/<model>` (keyless providers: `ollama`, `llamacpp`, `mistralrs`). The operator’s curated registry lives at `~/.malvinconf/local_llms.json`. When that file lists one or more models, `malvin admin models` keeps only those keyless-local ids (cloud providers are unchanged). When the file is missing or `"models"` is empty, listing stays unfiltered (every reachable local model appears).

### Schema

```json
{
  "models": [
    {
      "id": "ollama/malvin-qwen14:latest",
      "source": "qwen2.5-coder:14b",
      "notes": "optional free-text"
    }
  ]
}
```

- **`id`** (required): Pi-style `provider/model` (no `pi:` / `local/` prefix). Display and CLI use `pi:local/<id>`.
- **`source`** (optional): upstream pull tag or weights origin used to install the model.
- **`notes`** (optional): why this model is kept (host RAM, FT results, tool support, and so on).
- **`context_size`** (optional, positive integer): per-model override of `context_size` from `~/.malvinconf/config.toml`. Malvin writes it as `contextWindow` (and one quarter of it as `maxTokens`) into Pi’s `models.json` for sessions on this model. Reasoning models that think before answering (for example Qwen3) need a large value, at least 32768; the server’s own context (`num_ctx`, `llama-server -c`) must be at least this large.

### Agent workflow (install / configure)

When the operator asks to find, install, or configure a local LLM via the normal malvin interface:

1. **Research** size and tool support against host RAM (`Sandbox memory` / machine GiB). Prefer Ollama library tags or a Modelfile wrapper with `PARAMETER num_ctx` equal to the model’s effective context size: its `context_size` in `local_llms.json` when set, otherwise `context_size` in `~/.malvinconf/config.toml`. For reasoning models, set a per-model `context_size` large enough that one quarter of it covers thinking plus the answer. Ollama returns thinking in a `reasoning` field that Pi does not display, so a too-small budget can end a turn with an empty answer. When that happens, malvin stops with the error "output cap reached while thinking; raise `context_size` for this model" and a nonzero exit code instead of retrying.
2. **Install** with the provider CLI (Ollama: `ollama pull <tag>`, or `ollama create <name> -f Modelfile`). Malvin does not bundle a download subcommand.
3. **Configure** by upserting an entry in `~/.malvinconf/local_llms.json` (create the file with the schema above if missing). Keep only models the operator wants listed.
4. **Verify** with `malvin admin models pi:local` (keyless-local catalogs are always live-fetched when the provider is listening; cloud providers still use the daily cache / `--refresh`) and a short `malvin --do --model=pi:local/<provider>/<model> …` probe when appropriate.
5. **Remove** by deleting the Ollama tag (optional) and removing the matching object from `local_llms.json`.

Runtime auto-start / idle stop for Ollama is separate (local LLM manager under `~/.malvinconf/`); the JSON file is the curated catalog, not the process supervisor. Malvin auto-starts only Ollama. Other keyless providers (`llamacpp` at `http://127.0.0.1:8080/v1`, `mistralrs`) must already be listening, for example under a launchd user agent; use them when a model needs a runtime Ollama cannot provide (such as a vendor fork of llama.cpp).

On a small host, a resident server of one provider can exhaust GPU memory for another (on Apple silicon the log shows `kIOGPUCommandBufferCallbackErrorOutOfMemory`). Malvin then fails with `Compute error` or `unexpected Content-Type application/x-ndjson` (after `--max-acp-retries` identical errors, or the local cap of 10 errors / 5 minutes) and adds a hint pointing here. Stop the other server (for example `launchctl bootout gui/$UID/<label>`), run `ollama stop <model>` to unload the failed runner, and retry.

Small local models have limits of their own. Models below about 7B parameters, or without the `tools` capability (for example Gemma 3 4B), can answer questions but should not be expected to edit files; many of them never make a tool call. Full mode (bare `malvin REQUEST`) sends a long header and runs several turns, so on local models it is slow and often times out; prefer `malvin --do` for them. The default route puts the request text (up to 8 KB) into the prompt for `pi:` models, which saves them a tool call. When a local model spends its whole output budget thinking and returns no answer or tool call, malvin stops the turn with an output-cap error; raise `context_size` for that model.

## Log retention

After most agent-backed commands create a new run directory and emit the startup `Command:` line, malvin may prune older directories under `~/.malvinconf/logs/<hash>/` according to `~/.malvinconf/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). The active run is protected during prune. Set `max_count = 0` for unlimited run count (byte and age caps still apply). Agent-backed commands (including `malvin --do` and `malvin -g`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command may delete excess oldest runs once.

## External dependencies

- **Rust**: ≥ 1.95 (`rust-version` in this package; crates.io `0.2.6` declared 1.96). With an older rustc, `cargo install malvin` fails.
- **Node.js**: ≥ 22.13 with `npm` (≥ 22.19 for `pi:`, which runs Pi 1.x), needed at run time only by `cursor:` and `pi:` models. Building malvin does not need Node. `codex:` models do not need Node.
- **Cursor SDK**: `@cursor/sdk` via `cursor-sdk-bridge/`. The compiled bridge is embedded in the malvin binary. The first time a `cursor:` model runs, malvin writes it to `~/.malvinconf/sdk-bridges/cursor-sdk-bridge/` and runs `npm ci --omit=dev` there; later runs reuse that install until the bundled lock file changes. A repo checkout whose `cursor-sdk-bridge/` already has `node_modules` is used in place. Node is found via `MALVIN_NODE`, `PATH`, or the Cursor `agent` install; npm via `MALVIN_NPM`, next to that Node, or `PATH`. `MALVIN_CURSOR_SDK_BRIDGE` overrides the bridge path. `cursor:` models also need a Cursor API key (`CURSOR_API_KEY`, or `CURSOR_AGENT_API_KEY` / `AGENT_API_KEY`). `malvin admin models` lists Cursor models via the bridge when possible; falls back to `agent` / `cursor-agent` on `PATH` if the SDK path fails.
- **OpenRouter**: `OPENROUTER_API_KEY` when using `pi:openrouter/…` models.
- **Pi**: `pi:` models run the official npm package `@earendil-works/pi-coding-agent` in RPC mode. Malvin does not install it. It uses `MALVIN_PI` (path to the package's `cli.js` or `rpc-entry.js`) when set, and otherwise looks for the package in `./node_modules`, `~/.malvinconf/sdk-bridges/node_modules`, and the npx cache (`~/.npm/_npx`). If none is found, `pi:` runs fail with a hint to install it (for example `npm install --prefix ~/.malvinconf/sdk-bridges @earendil-works/pi-coding-agent`). Provider keys come from Pi’s env vars or credentials already stored under Pi’s auth path (`PI_CODING_AGENT_DIR` / `~/.pi/agent`).
- **curl**: used for local-model probes (Ollama) and pricing catalogs (OpenRouter, Portkey).
- **Codex**: `codex:` models require a separate `codex` binary (`PATH` or `MALVIN_CODEX`; not bundled) and a Codex login (`codex login`, `OPENAI_API_KEY`, or `$CODEX_HOME/auth.json`).
- **pre-commit**: optional; malvin does not install hooks automatically.

## Request syntax

Several commands accept positional request arguments. Each `<REQUEST>` is **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request. On the bare default route, multiple `REQUEST` arguments each run independently (new log directory, one router pass, summarize). Each `--do` tags only the next `REQUEST` as a one-shot session; any other `REQUEST` (before or after) uses the router. Each `--creative[=PROB]` tags only the next `REQUEST` for creative sampling; other `REQUEST` args stay non-creative. You can interleave them, for example `malvin "router task" --do "one-shot" "another router task"` or `malvin "plain" --creative "spark"`.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| bare `malvin REQUEST…`, `--do REQUEST` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

Examples:

```text
malvin --do "fix the typo"
malvin --do "Hello" "Research the topic"
malvin "Write a function" --do "What time is it?" "Find a bug" --do "Summarize in report.md"
malvin --creative "explore API boundaries"
malvin "plain task" --creative "spark ideas" "another plain" --creative=0.4 "biased spark"
malvin request_1.md request_2.md
```

## Gate-loop commands

`malvin -g` without a request is a thin wrapper: it composes a fixed request (`Get the gates to pass.`) and invokes the **default router** with `--gates` on. When `.malvin/gates` is missing, malvin runs the init workflow first, then this gate-fix workflow (see **Quality gates** above).

See the default-route section of `malvin --doc`.


## Running on Modal (`--remote=modal:sandbox`)

`malvin --remote=modal:sandbox ...` runs the same command in a [Modal Sandbox](https://modal.com/docs/guide/sandboxes) instead of on this machine. The stream, the run logs, and the file changes match a local run. Every other flag and argument is passed to the remote malvin unchanged; `--remote` itself is not.

- **Setup**: Node.js ≥ 22.13 with `npm` on this machine (malvin installs the Modal JS SDK under `~/.malvinconf/sdk-bridges/modal-bridge/` with `npm ci`), and Modal credentials from `modal setup` (`~/.modal.toml`) or `MODAL_TOKEN_ID` plus `MODAL_TOKEN_SECRET`. Modal credentials never leave this machine.
- **Image**: malvin publishes a Modal image named `malvin-bin:<version>-<hash>`, built once and reused by later runs, which then start in seconds. On x86-64 Linux with glibc, the image contains this machine's own malvin binary. On other hosts, the image builds malvin with `cargo install malvin --version <same version>`, which works only for versions published on crates.io. The default base is `node:22-trixie-slim` plus `git`, `curl`, Python 3, and `build-essential`. `codex:` models add the `codex` CLI and `pi:` models add the npm Pi agent, each pinned to the version installed on this machine (or `latest` when that version cannot be read).
- **What is uploaded**: in a git work tree, tracked and untracked-but-not-ignored files under the current directory (never `.git`); outside git, everything under the current directory except `.git`, `target`, and `node_modules`; if that is more than 1 GiB, malvin exits 1 before contacting Modal. Also uploaded: the five newest run directories of this workspace, `~/.malvinconf/config.toml`, and any REQUEST files. The workspace is unpacked at the same absolute path, with `HOME` set to the local home path, so log directories and HISTORY paths match a local run.
- **Credentials**: `CURSOR_API_KEY`, `CURSOR_AGENT_API_KEY`, `AGENT_API_KEY`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, and `OPENROUTER_API_KEY` are passed to the remote command when set, and never written into an image. `codex:` models also get `~/.codex/auth.json`, and `pi:` models get Pi's `auth.json` and `models.json`. Anyone with access to the Modal workspace can, in principle, inspect a running Sandbox.
- **Results**: when the remote run ends, its changes come back as a git patch, and malvin prints which of these outcomes occurred:
  - The remote run changed no files.
  - `git apply` succeeded: the changes are in the working tree, unstaged.
  - `git apply` failed but `git apply --3way` succeeded: the changes are in the working tree **and staged in the index**.
  - The 3-way merge hit conflicts: conflict markers are left in the working tree, and the patch is also kept as `modal.patch`.
  - The patch could not be applied at all, or the directory is not a git repository: the working tree is untouched and the patch is kept as `modal.patch`.

  The new run directories are copied into this workspace's log directory. When the patch is kept, `modal.patch` goes into the newest of them, or into the workspace's log directory if none came back.
- **Exit status and failures**: `malvin --remote=modal:sandbox` exits 0 only when the remote malvin exits 0; otherwise it exits 1. A failed remote run still returns its changes and logs, applied as above, so check the working tree after a nonzero exit. If talking to Modal fails partway (for example an upload, download, or remote setup step fails), malvin prints the error and exits 1 without applying any changes or copying any logs.
- **Resources**: `--remote=modal:sandbox[gpu=...,ncpu=...,mem=...,timeout=...]` sets the Sandbox's resources for one run. Each suboption is optional, they may appear in any order, and an omitted one falls back to the `[modal]` setting of the same name in `~/.malvinconf/config.toml`, then to the built-in default. `malvin admin remotes` prints this list as a table.
  - `gpu`: `none` (default), a Modal GPU type such as `T4` or `A100` (`malvin admin remotes` lists the current types), or `TYPE:COUNT` for several GPUs (for example `T4:2`).
  - `ncpu`: number of CPU cores reserved for the Sandbox, a positive whole number (default 1).
  - `mem`: the Sandbox's memory in GiB, a positive whole number with an optional `G`, `GB`, or `GiB` suffix (default 8; for example `mem=16` or `mem=16G`). The remote malvin's `mem_limit_gb` is 2 less than this, or this machine's `mem_limit_gb` if that is smaller.
  - `timeout`: the Sandbox's hard lifetime (default 30 minutes, at most 24 hours). A bare number means minutes (`timeout=45`); the suffixes `s`, `m`, and `h` select seconds, minutes, and hours (`90s`, `45m`, `2h`).

  The brackets are shell glob characters, so quote the flag (`'--remote=modal:sandbox[gpu=T4]'`) if your shell complains or a file name could match it. malvin prints the resources it chose when the Sandbox starts, for example `Sandbox sb-… started (T4, 2 CPU, 8 GiB, timeout 10 min)`.
- **Remote output**: lines the remote malvin prints with a who-tag (such as `o|`) appear unchanged. Every other remote line, such as npm's `added 11 packages in 3s` or the remote agent's reply, is shown with the who-tag `r|`. With `--do` (and no `--verbose`), malvin prints only the DM body, as in a local `--do`: the remote agent's reply is printed untagged (rendered as markdown when stdout is a terminal), and the `modal:` status lines and other remote output are not shown; only error (`e|`) lines still reach stderr. Add `--verbose` to stream the remote run's log and the status lines.
- **Lifetime**: Ctrl-C terminates the Sandbox. When the Sandbox reaches its `timeout`, Modal stops it mid-run, so no changes or logs come back, and malvin's error says the timeout was likely reached. At the start of each `--remote=modal:sandbox` run, malvin terminates Sandboxes left by this host's exited `--remote=modal:sandbox` runs.
- **Rejected**: `--watch`, `--ml=inf`, and the `admin` subcommand exit 1 with an error before anything is uploaded. A finite `--ml=N` is allowed. With `--doc`, only the `admin` combination is still rejected; the others print documentation.
- **Not supported, but not rejected**: local LLMs (`pi:local/…`, `pi:ollama/…`). The Sandbox runs no local model server, so a local-model run is expected to fail inside the Sandbox rather than at startup.

Optional settings in `~/.malvinconf/config.toml`:

```toml
[modal]
gpu = "none"            # default GPU: "none" (default), a type such as "A100", or "TYPE:COUNT"
ncpu = 1                # default CPU cores (default 1)
timeout = "30m"         # default hard Sandbox lifetime: minutes as an integer, or "90s" / "45m" / "2h" (default 30 minutes, at most 24 hours)
mem = 8                 # default Sandbox memory in GiB (default 8); the remote mem_limit_gb becomes mem - 2 if smaller
image = "python:3.12"   # replaces the default base; needs glibc at least as new as this host's, plus Node >= 22.13 with npm
setup = ["RUN pip install -r requirements.txt"]   # extra Dockerfile lines, cached by Modal
```

The older keys `cpu`, `timeout_h`, `memory_gb`, and `memory` are rejected with a message naming their replacements, `ncpu`, `timeout`, and `mem`.
