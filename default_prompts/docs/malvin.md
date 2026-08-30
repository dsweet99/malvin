# malvin (top-level CLI)

malvin is a non-interactive research and coding agent. It runs agent sessions against a workspace through the Cursor SDK (`cursor:` models via a Node bridge to `@cursor/sdk`), an in-process Pi SDK (`pi:` models via linked `pi_agent_rust`), or a local Codex app-server (`codex:` models via `codex app-server`). Each agent-backed invocation creates an isolated run directory under `~/.malvin_home/logs/<hash>/` and records prompts, stdout, and artifacts there.

## How to read this documentation

- **Humans:** skim **Commands**, then open `malvin <COMMAND> --doc` for the workflow you need.
- **Agents:** treat each `--doc` file as a self-contained contract for that command; global flags and run-directory rules live in this file.
- **Help vs doc:** `malvin --help` lists flags; `--doc` explains behavior, logs, and when to use each command.

## Usage

```text
malvin [OPTION]... [REQUEST]
   or: malvin [OPTION]... <COMMAND>
```

These forms are mutually exclusive: pass a request **or** a subcommand, not both on one synopsis line. `malvin --help` uses the same two-line usage.

Bare `malvin REQUEST` runs autonomous routing (`router_a` / optional `router_b`, stop on `__MALVIN_DONE__`, exit `router_summarize`). With no request and no subcommand, malvin prints a short command catalog and exits 0. `malvin -g` without a request runs the gate-fix workflow (fixed request `Get the gates to pass.` with `--gates` on). Use `--do` for a one-shot turn, or subcommands `write`, `admin`. Omitting `REQUEST` for `--do` or `write` likewise prints short usage and exits 0.

## Commands

| Command | Purpose |
|---------|---------|
| *(default)* | Bare `malvin REQUEST` — `header` → `router_a` → optional `router_b`; exit `router_summarize`; outer `--max-loops` sessions |
| `--do` | One-shot agent turn (non-looping) |
| `malvin -g` | Fix quality gates via the default router with fixed request `Get the gates to pass.` (no positional request) |
| `write` | Write a LaTeX PDF on code or concepts via a composed default-router request |
| `admin` | Operator maintenance (`models`, `reset-herdr`, …) |

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`); for the one-shot workflow use `malvin --do --doc`. The default-route contract (`router.md`) is printed after this overview when you run `malvin --doc`.

## Global options

`--doc` is a true global: it may appear before or after any subcommand, including `admin`.

Agent-session flags (`-b` / `--background`, `--model`, `--gates`, `-q`, `-v`, `--git`, `--creative[=PROB]`, `--no-force`, `--no-tenacious`, `--max-acp-retries`, …) apply to bare `malvin REQUEST`, `--do`, and `write`. On `write` they may appear before or after the subcommand name. The `admin` help listing omits them; pass `--model` before `admin models` only when you want to set that command’s `Current:` footer.


### `-b` / `--background`

Suppress all stdout from malvin and the agent. Run logs under `~/.malvin_home/logs/` are unchanged.

### `-q` / `--quiet`

On the **default router** (bare `malvin REQUEST` and `malvin -g`) and on one-shot agent commands that tee styled agent stdout (`write`), print only the text between `__MALVIN_DM_START__` and `__MALVIN_DM_END__` fences to process stdout. Startup chrome, agent stream, heartbeats, prompt-name lines, fence markers, and TIMING/COST lines are omitted from stdout. Run-dir logs and stderr are unchanged.

This is **not** the same as `-b` / `--background` (which suppresses all stdout, including DM bodies). It is also **not** required for plain `malvin --do`: without `--verbose`, `--do` is already DM-body-only on stdout. With `--verbose`, `--do` tees the same live agent log classes as the default workflow (see `-v` / `--verbose` below).

### `--model <MODEL>`

Model id for agent-backed commands. Default: `cursor:auto`. Use `cursor:` for the Cursor SDK backend, or `pi:<provider>/<model>` for the in-process Pi backend (linked `pi_agent_rust`; uses env keys or credentials already stored by Pi). Optional bracket overrides select thinking / speed where the backend supports them, for example `cursor:claude-opus-5[effort=high,fast=true]` or `pi:openai/gpt-5[thinking=high]` (see `malvin admin models --doc`). Legacy `prime:` ids are rejected.

### `--max-loops <N>` (default: 1)

Outer agent-session budget for bare `malvin REQUEST` and `malvin -g`. `0` is treated as `1`. `write` exposes its own `--max-loops` with a default of `3`.

### `--max-hypotheses <N>` (default: 5)

Hypothesis budget for bare `malvin REQUEST` and `malvin -g`. When the flag is omitted, `[default_workflow].max_hypotheses` from `~/.malvin_home/config.toml` is used (fallback 5). Explicit CLI wins over config. `0` is treated as `5`. `write` exposes its own `--max-hypotheses`.

### `--no-force`

By default agent backends run tools headlessly (auto-approved). `--no-force` is not supported on `cursor:`, `pi:`, or `codex:` (no interactive approval prompt); malvin fails fast with a clear error before any session starts.

### `--no-tenacious`

By default `write` expands to `--max-loops=9999` and `--max-acp-retries=9999`. The bare default route and `malvin -g` expand both `--max-loops=9999` and `--max-acp-retries=9999` unless the matching flag was set explicitly on the command line. `--no-tenacious` restores normal budgets.

### `-g` / `--gates`

Inject workspace check command text into agent prompts and, for workflows that use harness gates as loop criteria, treat failures as loop or exit criteria. Off by default. When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first (default router with request from `init_constraints.md`, harness gates off) to discover and write `.malvin/gates`. On bare `malvin REQUEST`, `-g` / `--gates` also runs workspace `.malvin/gates` after `router_a` emits `__MALVIN_DONE__`: pass stops success; fail continues the outer loop; exhausted budget with failing gates fails the run. `malvin -g` without a request runs the gate-fix workflow with this flag on and fixed request `Get the gates to pass.` When work runs, check text is still injected into the work prompt. Agent prompts may still include available `.malvin/gates` guidance when this option is off.



### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown. For `malvin --do`, also unlock the same live agent stdout log classes as the default workflow (thought tokens and narrative tee); without `--verbose`, `--do` stays DM-body-only.

### `--max-acp-retries <N>` (default: 3)

Maximum bounded attempts per Cursor SDK bridge spawn or `send`/`wait`, with 1s / 3s backoff between tries. `--tenacious` on gate-loop commands sets this to 9999.

### `--git`

Allow the agent to run `git commit`. Off by default (agents are otherwise steered away from committing).

### `--creative[=PROB]`

On the default router (bare `malvin REQUEST` and `malvin -g`), when creative mode is sampled for an outer iteration: send `mbc2.md` after `kpop_common.md`, and use `router_b_creative.md` instead of `router_b.md` for the optional work turn. Both changes share one Bernoulli draw per outer iteration. `--creative` alone uses probability `1.0`; `--creative=0.6` uses `0.6`. Off by default.

### Session names

For bare `malvin REQUEST`, `--do`, and `malvin -g`, malvin assigns a unique five-character session id (`[a-z0-9]`) and acquires a session name lock before substantive work.

Malvin registers the top-level process under this id in a per-user registry at `~/.malvin_home/names/<ID>` (one line: holder PID). If another live malvin process already holds the same id, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin_home/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live agent/bridge session per lock slot in a workspace). Two malvin processes with different session ids may both register names and hold live sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/gates/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--help`, `--version`, and `malvin` with no subcommand do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --doc` — this overview, then the default-route contract (`router.md`).
- `malvin <COMMAND> --doc` — documentation for that subcommand.
- `malvin --do --doc` — documentation for the one-shot `--do` workflow.

Other subcommand arguments (for example `<REQUEST>`) are not required when `--doc` is set.

## Quality gates (`.malvin/gates`)

When `--gates` is set and `.malvin/gates` is missing, malvin runs the init workflow first: it renders `init_constraints.md` (cwd as `repo_root_path`) and invokes the **default router** with harness gates off to discover and write `.malvin/gates`.

With `--gates` and an existing `.malvin/gates`, malvin runs workspace quality gates from that file at the repo git root (one shell command per non-empty, non-comment line). Full-line comments starting with `#` are ignored. `malvin -g` without a request always enables this harness.

Other invocations (`--do`, bare `malvin REQUEST`, `write`) do not require `.malvin/gates` at startup and may run outside a git repo. With `--gates` on a bare `malvin REQUEST`, malvin runs workspace gates when `router_a` emits `__MALVIN_DONE__` and continues that outer loop when they fail (see the default-route section of `malvin --doc`). Without `--gates` (the default for other commands), malvin does not run those checks directly on the default route. `header.md` notes about gates lines remain advisory when a workspace happens to have gates; they are not a startup requirement for those commands.

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
| `_run/exp_log_*.md` | Experiment / gate-loop logs |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Session footnotes (`TIMING` / `COST`)

At the end of a timed run (before `DONE`), malvin writes footnote lines to `stdout.log` (and to process stdout unless `-q` / `-b`):

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

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, …). Use for skimming a run and vocabulary/ordering checks.
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

Top-level keys include `mem_limit_gb` and `theme`. Cursor cost rates `usd_per_microtoken_in`, `usd_per_microtoken_out`, `usd_per_microtoken_cache_read`, and `usd_per_microtoken_cache_write` (dollars per million tokens; all default `0`) live under per-model tables such as `[agent.cursor.auto]` (model id `cursor:auto`). Sections include `[agent]`, `[review]` (`max_hypotheses` for `malvin write` when `--max-hypotheses` is omitted), `[default_workflow]` (`max_hypotheses` for bare `malvin REQUEST` when `--max-hypotheses` is omitted, default 5), and `[logs]`.

## Log retention

After most agent-backed commands create a new run directory and emit the startup `Command:` line, malvin may prune older directories under `~/.malvin_home/logs/<hash>/` according to `~/.malvin_home/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). The active run is protected during prune. Set `max_count = 0` for unlimited run count (byte and age caps still apply). Agent-backed commands (including `malvin --do` and `malvin -g`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command may delete excess oldest runs once.

## External dependencies

- **Node.js**: ≥ 22.13 with `npm` on `PATH`. `cargo install malvin` / `cargo build` run `build.rs`, which installs the Cursor SDK bridge under `~/.malvin_home/sdk-bridges/` when the in-tree bridge is not already built (required for `cursor:` agent backends). Set `MALVIN_SKIP_SDK_BRIDGES=1` only to compile the binary without that SDK.
- **Cursor SDK**: `@cursor/sdk` via `cursor-sdk-bridge/` (installed at build time), and a Cursor API key (`CURSOR_API_KEY`, or `CURSOR_AGENT_API_KEY` / `AGENT_API_KEY`) for `cursor:` models. `malvin admin models` lists Cursor models via the bridge when possible; falls back to `agent` / `cursor-agent` on `PATH` if the SDK path fails.
- **OpenRouter**: `OPENROUTER_API_KEY` when using `pi:openrouter/…` models.
- **Pi SDK**: malvin links crates.io `pi_agent_rust` and lists or runs `pi:` models from that registry. Provider keys follow Pi’s env vars or credentials already stored under Pi’s auth path (`PI_CODING_AGENT_DIR` / `~/.pi/agent`). An external `pi` binary is not required.
- **pre-commit**: optional; malvin does not install hooks automatically.

## Request syntax

Several commands accept a positional request. `<REQUEST>` is always exactly **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| bare `malvin REQUEST`, `--do` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

Examples:

```text
malvin --do "fix the typo"
malvin --creative "explore API boundaries"
```

## Gate-loop and document commands

`malvin -g` without a request is a thin wrapper: it composes a fixed request (`Get the gates to pass.`) and invokes the **default router** with `--gates` on. When `.malvin/gates` is missing, malvin runs the init workflow first, then this gate-fix workflow (see **Quality gates** above).

`malvin write` starts one agent session and sends two prompts in order (`write_a.md`, then `write_b.md`). It does not use the default router.

See `malvin write --doc` and the default-route section of `malvin --doc`.

