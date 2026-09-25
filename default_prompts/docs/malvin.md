# malvin (top-level CLI)

malvin is a non-interactive research and coding agent. It runs agent sessions against a workspace through the Cursor SDK (`cursor:` models via a Node bridge to `@cursor/sdk`), the official TypeScript/npm Pi agent (`pi:` models via RPC), an in-process rust Pi SDK (`rpi:` models via linked `pi_agent_rust`), or a local Codex app-server (`codex:` models via `codex app-server`). Each agent-backed invocation creates an isolated run directory under `~/.malvin_home/logs/<hash>/` and records prompts, stdout, and artifacts there. When the workspace root contains a non-empty `AGENTS.md`, malvin embeds it in `header.md` via `{{ agents_insert }}` so every fresh-header session sees that guidance without relying on Cursor rule auto-load.

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

Bare `malvin REQUEST` runs autonomous routing (`router_a` / optional `router_b`, stop on `__MALVIN_DONE__`, exit `router_summarize`). Multiple `REQUEST` arguments each run as an independent default-route invocation (new run directory, full outer loop, summarize). With no request and no subcommand, malvin prints a short command catalog and exits 0. `malvin -g` without a request runs the gate-fix workflow (fixed request `Get the gates to pass.` with `--gates` on). Each `--do` applies only to the `REQUEST` that immediately follows it (one-shot turn); other `REQUEST` args still use the router. Each `--creative[=PROB]` likewise applies only to the `REQUEST` that immediately follows it (repeatable; other `REQUEST` args stay non-creative). The `admin` subcommand covers operator maintenance. Omitting `REQUEST` after a lone `--do` prints short usage and exits 0.

## Commands

| Command | Purpose |
|---------|---------|
| *(default)* | Bare `malvin REQUEST` — aggregated initial (`header` when fresh, with `kpop_insert` from `kpop_common` unless `--no-kpop`, + optional `mbc2` + `router_a`) → optional `router_b`; exit `router_summarize`; outer `--max-loops` iterations |
| `--do` | One-shot agent turn for the following REQUEST (repeatable; other REQUESTs stay on the router) |
| `--creative[=PROB]` | Creative mode for the following REQUEST only (repeatable; optional probability, default `1.0`) |
| `malvin -g` | Fix quality gates via the default router with fixed request `Get the gates to pass.` (no positional request) |
| `admin` | Operator maintenance (`models`, `reset-herdr`/`rh`, …) |

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`); for the one-shot workflow use `malvin --do --doc`. The default-route contract (`router.md`) is printed after this overview when you run `malvin --doc`.

## Global options

`--doc` is a true global: it may appear before or after any subcommand, including `admin`.

Agent-session flags (`--model`, `--gates`, `-q`, `-v`, `--creative[=PROB]`, `--max-acp-retries`, `--iml`, …) apply to bare `malvin REQUEST` and `--do` (`--max-loops`, `--max-hypotheses`, and `--watch` apply only to bare `malvin REQUEST` and `malvin -g`). The `admin` help listing omits them; pass `--model` before `admin models` only when you want to set that command’s `Current:` footer.


### `-q` / `--quiet`

On the **default router** (bare `malvin REQUEST` and `malvin -g`), print only the text between `__MALVIN_DM_START__` and `__MALVIN_DM_END__` fences to process stdout. Startup chrome, agent stream, heartbeats, prompt-name lines, fence markers, and TIMING/COST lines are omitted from stdout. Run-dir logs and stderr are unchanged.

It is also **not** required for plain `malvin --do`: without `--verbose`, `--do` is already DM-body-only on stdout. With `--verbose`, `--do` tees the same live agent log classes as the default workflow (see `-v` / `--verbose` below).

### `--model <MODEL>`

Model id for agent-backed commands. Default: `cursor:auto`. Use `cursor:` for the Cursor SDK backend, or `rpi:<provider>/<model>` for the in-process Pi backend (linked `pi_agent_rust`; uses env keys or credentials already stored by Pi). Optional bracket overrides select thinking / speed where the backend supports them, for example `cursor:claude-opus-5[effort=high,fast=true]` or `rpi:openai/gpt-5[thinking=high]` (see `malvin admin models --doc`). Legacy `prime:` ids are rejected.

### `--max-loops <N>` (default: 9999)

Outer agent-session budget for bare `malvin REQUEST` and `malvin -g`. `0` is treated as `1`.

### `--max-hypotheses <N>` (default: 5)

Hypothesis budget for bare `malvin REQUEST` and `malvin -g`. When the flag is omitted, `[default_workflow].max_hypotheses` from `~/.malvin_home/config.toml` is used (fallback 5). Explicit CLI wins over config. `0` is treated as `5`.

### `-g` / `--gates`

Inject workspace check command text into agent prompts and, for workflows that use harness gates as loop criteria, treat failures as loop or exit criteria. Off by default. When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first (default router with request from `init_constraints.md`, harness gates off) to discover and write `.malvin/gates`. On bare `malvin REQUEST`, `-g` / `--gates` also runs workspace `.malvin/gates` after `router_a` emits `__MALVIN_DONE__`: pass stops success; fail continues the outer loop; exhausted budget with failing gates fails the run. `malvin -g` without a request runs the gate-fix workflow with this flag on and fixed request `Get the gates to pass.` When work runs, check text is still injected into the work prompt. Agent prompts may still include available `.malvin/gates` guidance when this option is off.



### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown. For `malvin --do`, also unlock the same live agent stdout log classes as the default workflow (thought tokens and narrative tee); without `--verbose`, `--do` stays DM-body-only.

### `--max-acp-retries <N>` (default: 3)

Stop after N consecutive identical backend errors (spawn, header, or prompt), with 1s / 3s backoff between tries. Distinct errors reset the consecutive counter. Fail-fast classes (billing, usage limit, invalid model, and similar) still exit immediately.

### `--creative[=PROB]`

On the default router (bare `malvin REQUEST` and `malvin -g`), when creative mode is sampled for an outer iteration: include `mbc2.md` in the aggregated initial prompt (after header / kpop insert), and fill `router_b.md` creative template keys (`{{ creative_lead }}`, brief `{{ satisfy_line }}`) for the optional work turn. Both changes share one Bernoulli draw per outer iteration. `--creative` alone uses probability `1.0`; `--creative=0.6` uses `0.6`. Off by default.

Like `--do`, each `--creative` applies only to the `REQUEST` that immediately follows it and may be repeated (at most once per `REQUEST`). Example: `malvin "plain" --creative "spark" "plain2" --creative=0.4 "spark2"`. Intervening global flags (for example `--max-loops`) may appear between `--creative` and its `REQUEST`. A trailing `--creative` after other requests, or `--creative` immediately followed by `--do` (or the reverse), is an error. For `malvin -g` with no positional request, `--creative` still enables creative sampling for that gates-only run.

### `--watch`

On the default router (bare `malvin REQUEST` and `malvin -g`), before each outer loop iteration, re-copy the operator's request `.md` file onto the run's `plan_*.md` artifact (overwrite). No effect when `REQUEST` is literal text (not an existing `.md` path). Has no effect on `--do` requests; when the invocation is pure `--do` (no router REQUEST), `--watch` is rejected.

### `--iml`

the Infinite Meta-Loop. After every REQUEST in the invocation has run once (preserving `--do` vs router tagging and order), start again from the first REQUEST and repeat forever — as if the same command line were re-invoked. A failing REQUEST stops the process (the loop does not continue past an error). Init bootstrap, when needed, still runs once at the start of the process. Applies to bare `malvin REQUEST…`, mixed/`--do` request lists, and `malvin -g`.

### Session names

For bare `malvin REQUEST`, `--do`, and `malvin -g`, malvin assigns a unique five-character session id (`[a-z0-9]`) and acquires a session name lock before substantive work.

Malvin registers the top-level process under this id in a per-user registry at `~/.malvin_home/names/<ID>` (one line: holder PID). If another live malvin process already holds the same id, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin_home/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live agent/bridge session per lock slot in a workspace). Two malvin processes with different session ids may both register names and hold live sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/gates/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--advice`, `--help`, `--version`, and `malvin` with no subcommand do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --doc` — this overview, then the default-route contract (`router.md`).
- `malvin <COMMAND> --doc` — documentation for that subcommand.
- `malvin --do --doc` — documentation for the one-shot `--do` workflow.

Other subcommand arguments (for example `<REQUEST>`) are not required when `--doc` is set.

### `--advice`

Print an embedded advice document for `TAG` to stdout and exit, or list available tags when `TAG` is omitted. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --advice` — list every TAG with a brief description (at most 7 words), plus the document’s line count and character count.
- `malvin --advice TAG` — body of the matching file under `default_prompts/advice/*.md`.
- Each document has one TAG: lowercase letters and digits, starting with a letter, at most 7 characters (example: `design` for `document_design.md`).

Other subcommand arguments are not required when `--advice` is set.

## Quality gates (`.malvin/gates`)

When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first: it renders `init_constraints.md` (cwd as `repo_root_path`) and invokes the **default router** with harness gates off to discover and write `.malvin/gates`.

With `--gates` and an existing `.malvin/gates`, malvin runs workspace quality gates from that file at the repo git root (one shell command per non-empty, non-comment line). Full-line comments starting with `#` are ignored. `malvin -g` without a request always enables this harness.

Other invocations (`--do`, bare `malvin REQUEST`) do not require `.malvin/gates` at startup and may run outside a git repo. With `--gates` on a bare `malvin REQUEST`, malvin runs workspace gates when `router_a` emits `__MALVIN_DONE__` and continues that outer loop when they fail (see the default-route section of `malvin --doc`). Without `--gates` (the default for other commands), malvin does not run those checks directly on the default route. `header.md` notes about gates lines remain advisory when a workspace happens to have gates; they are not a startup requirement for those commands.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Run directories and logs

Every agent-backed command creates `~/.malvin_home/logs/<hash>/<timestamp>_<token>/`. Typical files:

| File | Role |
|------|------|
| `plan_<random>.md` or `request.md` | Copy of user input for this run |
| `do.log`, `router_1.log`, `router_2.log`, … | Per-iteration or per-prompt transcripts |
| `stdout.log` | Tee of agent stdout — **narrative** channel |
| `trace.jsonl` | Audit record (sdk-shaped JSONL for Cursor SDK; Mini uses its own event shapes) — **authoritative** for semantics (tool results, shrink/fork, LLM usage) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `run_timing.json` | Wall/LLM timing, token/step aggregates, and optional cost |
| `_run/exp_log_*.md` | Experiment / gate-loop logs (`exp_log_<run>.md` scaffold; `exp_log_<run>_gN.md` per outer router loop — kept, never truncated) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Session footnotes (`TIMING` / `COST`)

At the end of a timed run (before `DONE`), malvin writes footnote lines to `stdout.log` (and to process stdout unless `-q`):

```text
TIMING: wall = … llm_wait = … …
COST: steps = N tokens_in = X tokens_out = Y cache_read = A cache_write = B cost_in = … cost_out = … cost_read = … cost_write = … cost_tot = …
```

- **`steps`:** Mini / OpenRouter / Local count one step per successful LLM completion. Cursor SDK counts one step per SDK `onStep` boundary (not tool-call batch proxies). Raw tool-call counts are not printed as `steps`.
- **`tokens_in` / `tokens_out`:** Numeric when the backend reports usage. Cursor SDK folds one `result.usage` (`TokenUsage`) per `send` into these fields (cache read/write counted in `tokens_in`). When usage is absent, fields stay `n/a`.
- **`cache_read` / `cache_write`:** Separate cache token totals from the same usage objects when reported (`cacheReadTokens` / `cacheWriteTokens`). Still included in `tokens_in`. When absent, fields stay `n/a`.
- **`cost_in` / `cost_out` / `cost_read` / `cost_write` / `cost_tot`:** Estimated USD from per-model rates in `~/.malvin_home/config.toml` × token counts / 1e6. Rates are dollars per million tokens (`usd_per_microtoken_*`) under `[agent.<provider>.<name>]` for the run model (e.g. `[agent.cursor.auto]` for `cursor:auto`):
  - `cost_in = usd_per_microtoken_in ×` non-cache input tokens `/ 1_000_000` (stored `tokens_in` minus `cache_read` / `cache_write`)
  - `cost_out = usd_per_microtoken_out × tokens_out / 1_000_000`
  - `cost_read = usd_per_microtoken_cache_read × cache_read / 1_000_000`
  - `cost_write = usd_per_microtoken_cache_write × cache_write / 1_000_000`
  - `cost_tot` = sum of the four components
  All rates default to `0`, so with unset rates the estimate is `0` (shown as `0.0000`), not `n/a`. Set rates for a non-zero estimate. When usage was never observed, cost fields stay `n/a`.

### Narrative vs audit (trust rule)

Each run writes two parallel channels with different contracts:

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, `a|`, …). Use for skimming a run and vocabulary/ordering checks. An `a|<provider>:<model>` line (for example `a|cursor:auto`) is written each time a fresh agent context is started.
- **`trace.jsonl` (audit):** machine-authoritative JSONL (Cursor SDK events such as `assistant` / `thinking` / `tool_call` / `progress` / `run_done`; Mini retains its own audit shapes). Use for tool results, shrink/fork events, and gate-loop audit tooling.

Consumers must know which file to trust for which question. Named types live in `src/observability/` (`ObservabilityChannel`, `AuditEventKind`).

## SDK drain idle (bridge / Pi)

While waiting for the next Cursor SDK bridge or Pi RPC line, malvin applies a **per-event** idle budget (not a total-prompt wall clock):

| Clock | Meaning | Default |
|-------|---------|---------|
| Idle budget | Max silence since the last successful bridge/Pi event (or since the wait started) | `MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS` (600000 ms) |
| Turn cap (base) | Max wall clock for one prompt drain before productive extension | `2 × idle` (~1200s default) |
| Turn cap (extended) | Hard ceiling after infra heartbeats / tool activity | `10 × idle` (~6000s default) |
| Slice | How long to block on one read before sampling sandbox child health | `min(60000 ms, idle remaining)` |
| Health extend | If sandbox PIDs show CPU / ctxt / thread progress (`StillBusy`), refresh the idle budget once more, capped at `max_wait = 2 × idle` for that next-event wait | — |
| Infra turn heartbeat | Tool start extends turn cap by `2 × idle`; bridge `progress` heartbeats extend by `idle` whenever the SDK run is open (alive signal); `StillBusy` health extends turn cap by `idle`; I/O-bound work with open tools is treated like `StillBusy` | — |

Missing a line for the full (possibly health-extended) idle window fails with `bridge timed out … without a bridge event (bridge quiet; …)` — that is the hung/stalled bridge signal (no NDJSON lines, including heartbeats). Hitting the cumulative turn cap fails with `… after turn ran … (limit …; turn budget exhausted)` — the bridge may still have been emitting events. Local stdout heartbeats (`Orienting`, …) do **not** reset drain idle.

The Cursor SDK bridge also emits automatic `{ "event": "progress", "kind": "heartbeat" }` lines when a run is in flight and no SDK message/step has been forwarded for 15s. Those `progress` events reset the per-event idle budget like any other bridge line and extend the cumulative turn cap (they are recorded in `trace.jsonl`, not teed to narrative stdout).

**Differentiation:** continuing heartbeats (or other bridge lines) ⇒ SDK bridge alive, keep waiting up to the turn ceiling; full idle window with no bridge lines ⇒ quiet/hung bridge, fail and tear down. Open tracked tools additionally remap sandbox `AppearsHung` → `StillBusy` as a backup when the event loop cannot heartbeat during I/O-bound work.

**Limitation:** work backgrounded outside the bridge sandbox process group (for example a nested Docker `malvin` after the outer shell tool call has already completed) is not visible to child-health sampling. That case relies on the outer SDK run staying open so automatic `progress` heartbeats (or other bridge events) keep arriving inside the idle budget. Once `run_done` fires, progress stops; further silence still hits idle.

## Deferred stdout logging

Malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log` (legacy enrichment path). Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral.

## Home config (`~/.malvin_home/config.toml`)

Top-level keys include `mem_limit_gb`, `theme`, and `disable_rpi` (default `false`; when `true`, `rpi:` is rejected as a backend and omitted from `malvin admin models`). Cursor cost rates `usd_per_microtoken_in`, `usd_per_microtoken_out`, `usd_per_microtoken_cache_read`, and `usd_per_microtoken_cache_write` (dollars per million tokens; all default `0`) live under per-model tables such as `[agent.cursor.auto]` (model id `cursor:auto`). Sections include `[agent]`, `[default_workflow]` (`max_hypotheses` for bare `malvin REQUEST` when `--max-hypotheses` is omitted, default 5), `[logs]`, and optional `[nicknames]` (map short unprefixed names to full model ids for `--model` / `[agent].model`, e.g. `astra = "pi:openrouter/openai/gpt-astra"`).

## Local LLMs (`~/.malvin_home/local_llms.json`)

Malvin runs keyless local models through `rpi:local/<provider>/<model>` (today: Ollama). The operator’s curated registry lives at `~/.malvin_home/local_llms.json`. When that file lists one or more models, `malvin admin models` keeps only those keyless-local ids (cloud providers are unchanged). When the file is missing or `"models"` is empty, listing stays unfiltered (every reachable local model appears).

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

- **`id`** (required): Pi-style `provider/model` (no `rpi:` / `local/` prefix). Display and CLI use `rpi:local/<id>`.
- **`source`** (optional): upstream pull tag or weights origin used to install the model.
- **`notes`** (optional): why this model is kept (host RAM, FT results, tool support, and so on).

### Agent workflow (install / configure)

When the operator asks to find, install, or configure a local LLM via the normal malvin interface:

1. **Research** size and tool support against host RAM (`Sandbox memory` / machine GiB). Prefer Ollama library tags or a Modelfile wrapper with `PARAMETER num_ctx` aligned to `context_size` in `~/.malvin_home/config.toml`.
2. **Install** with the provider CLI (Ollama: `ollama pull <tag>`, or `ollama create <name> -f Modelfile`). Malvin does not bundle a download subcommand.
3. **Configure** by upserting an entry in `~/.malvin_home/local_llms.json` (create the file with the schema above if missing). Keep only models the operator wants listed.
4. **Verify** with `malvin admin models rpi:local` (keyless-local catalogs are always live-fetched when the provider is listening; cloud providers still use the daily cache / `--refresh`) and a short `malvin --do --model=rpi:local/<provider>/<model> …` probe when appropriate.
5. **Remove** by deleting the Ollama tag (optional) and removing the matching object from `local_llms.json`.

Runtime auto-start / idle stop for Ollama is separate (local LLM manager under `~/.malvin_home/`); the JSON file is the curated catalog, not the process supervisor.

## Log retention

After most agent-backed commands create a new run directory and emit the startup `Command:` line, malvin may prune older directories under `~/.malvin_home/logs/<hash>/` according to `~/.malvin_home/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). The active run is protected during prune. Set `max_count = 0` for unlimited run count (byte and age caps still apply). Agent-backed commands (including `malvin --do` and `malvin -g`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command may delete excess oldest runs once.

## External dependencies

- **Rust**: ≥ 1.95 (`rust-version` in this package; crates.io `0.2.6` declared 1.96). With an older rustc, `cargo install malvin` fails; a leftover ≤0.2.3 binary lists every `rpi:` provider instead of only authenticated ones.
- **Node.js**: ≥ 22.13 with `npm` on `PATH`. `cargo install malvin` / `cargo build` run `build.rs`, which installs the Cursor SDK bridge under `~/.malvin_home/sdk-bridges/` when the in-tree bridge is not already built (required for `cursor:` agent backends). Set `MALVIN_SKIP_SDK_BRIDGES=1` only to compile the binary without that SDK.
- **Cursor SDK**: `@cursor/sdk` via `cursor-sdk-bridge/` (installed at build time), and a Cursor API key (`CURSOR_API_KEY`, or `CURSOR_AGENT_API_KEY` / `AGENT_API_KEY`) for `cursor:` models. `malvin admin models` lists Cursor models via the bridge when possible; falls back to `agent` / `cursor-agent` on `PATH` if the SDK path fails.
- **OpenRouter**: `OPENROUTER_API_KEY` when using `rpi:openrouter/…` models.
- **Pi SDK**: malvin links crates.io `pi_agent_rust` and lists or runs `rpi:` models from that registry. Provider keys follow Pi’s env vars or credentials already stored under Pi’s auth path (`PI_CODING_AGENT_DIR` / `~/.pi/agent`). An external `pi` binary is not required.
- **pre-commit**: optional; malvin does not install hooks automatically.

## Request syntax

Several commands accept positional request arguments. Each `<REQUEST>` is **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request. On the bare default route, multiple `REQUEST` arguments each run independently (new log directory, full router loop, summarize). Each `--do` tags only the next `REQUEST` as a one-shot session; any other `REQUEST` (before or after) uses the router. Each `--creative[=PROB]` tags only the next `REQUEST` for creative sampling; other `REQUEST` args stay non-creative. You can interleave them, for example `malvin "router task" --do "one-shot" "another router task"` or `malvin "plain" --creative "spark"`.

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

