# malvin (top-level CLI)

malvin is a non-interactive CLI agent that drives the Cursor ACP (`cursor-agent` or `agent`) against a workspace. Each agent-backed invocation creates an isolated run directory under `~/.malvin_home/logs/<hash>/` and records prompts, stdout, and artifacts there.

## How to read this documentation

- **Humans:** skim **Commands**, then open `malvin <COMMAND> --doc` for the workflow you need.
- **Agents:** treat each `--doc` file as a self-contained contract for that command; global flags and run-directory rules live in this file.
- **Help vs doc:** `malvin --help` lists flags; `--doc` explains behavior, logs, and when to use each command.

## Usage

```text
malvin [OPTIONS] [<COMMAND> | REQUEST...]
```

Bare invocation (no subcommand):

- `malvin REQUEST` — KPop investigation (same as `malvin kpop REQUEST`)
- `malvin REQUEST...` — run KPop on each request in sequence; each gets its own run directory under `~/.malvin_home/logs/<hash>/`
- Quote a single request when the text contains spaces (e.g. `malvin "Why does the cache miss?"`)

Use subcommands for other workflows: `init`, `do`, `inspire`, `code`, `tidy`, `delight`, `explain`, `revise`, `models`.

## Commands

| Command | Purpose |
|---------|---------|
| `init` | Bootstrap a repo with malvin templates and tooling |
| `do` | One-shot agent turn (non-looping) |
| `inspire` | One-shot MBC2 boundary exploration (batch ideation) |
| `code` | Implement a plan via the KPop gate loop (`code_constraints.md`) |
| `tidy` | Fix quality gates via the KPop gate loop (`tidy_constraints.md`) |
| `delight` | Author a user-delighting feature plan via the KPop gate loop |
| `explain` | Explain code or concepts as a LaTeX PDF via the KPop gate loop |
| `revise` | Revise an existing document in place via the KPop gate loop |
| `models` | List models (Cursor agent CLI by default; `models --mini` lists OpenRouter) |

Hidden (backward compatible): `kpop` — prefer bare `malvin REQUEST` for investigation.

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`).

## Global options

These flags are **global**: they may appear before or after the subcommand name.

### `--no-color`

Disable ANSI color on malvin’s own status and error lines. Does not change the agent’s raw stream.

### `-b` / `--background`

Suppress all stdout from malvin and the agent. Run logs under `~/.malvin_home/logs/` are unchanged.

### `--model <MODEL>`

Model id passed to the Cursor agent for subcommands that spawn a session. Default: `auto` (see `malvin models`).

### `--no-force`

By default malvin passes `--force` to `cursor-agent` so tool calls proceed without interactive approval. `--no-force` disables that (the agent may wait for IDE approval).

### `--no-tenacious`

By default gate-loop commands (`code`, `kpop`, `tidy`, `delight`, `explain`, `revise`, bare `malvin REQUEST`) expand to `--max-loops=9999` and `--max-acp-retries=9999`. `--no-tenacious` restores normal loop/retry budgets.

### `--no-tee`

By default malvin tees agent stdout to the terminal (and `stdout.log` in the run dir). `--no-tee` suppresses live streaming; logs are still written under `~/.malvin_home/logs/`.

### `--no-markdown`

Disable styled markdown rendering of agent stdout for agent-backed subcommands that use the shared ACP client (`code`, `kpop`, `tidy` when the agent runs, `delight`, `explain`, `revise`, `inspire`, and the `init` summary phase). No effect on `models`. **`do` uses plain stdout** on a TTY regardless of this flag; piped `do` output is always plain.

### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown.

### `--max-acp-retries <N>` (default: 3)

Maximum bounded attempts per ACP spawn or `session/prompt`, with 1s / 3s backoff between tries. `--tenacious` on gate-loop commands sets this to 9999.

### `--mini`

Use the in-process mini agent backend (OpenRouter HTTP + bash fence loop) instead of Cursor ACP. Requires `OPENROUTER_API_KEY` and `bash` on `PATH`. Does not spawn `cursor-agent`; suitable for headless eval without Cursor credentials.

When `--mini` is set:

- Model selection precedence: `--model` on the command line (if given), then `[agent]."model-mini"` in `~/.malvin_home/config.toml`, then the built-in default slug `nvidia/nemotron-3-ultra-550b-a55b:free`. Legacy installs may lack `"model-mini"` on disk until you edit config or run `malvin init`; opening config merges the template key in memory only (same as other agent keys).
- `--model` is sent to OpenRouter; `--model auto` resolves to `nvidia/nemotron-3-ultra-550b-a55b:free` (via `MINI_DEFAULT_MODEL`, not ACP `agent.model`).
- `--no-force` is a no-op (nothing to approve).
- `--max-acp-retries` applies to gate iteration retries (not HTTP transport retries; see config below). OpenRouter billing/credit failures (402/403) and ACP “upgrade your plan” errors fail immediately at the gate level without retry or `mini gate attempt N failed` wrapping.
- `[agent].max_mini_transport_retries` in `~/.malvin_home/config.toml` (default **3**) caps retries for all non-billing OpenRouter HTTP failures (429, 5xx, 4xx, auth, JSON decode, reqwest transport, provider capacity). Billing/payment failures (402/403) fail immediately. `--mini-max-http-retries` is deprecated and ignored by the mini retry loop.
- Cost estimates from OpenRouter `usage.cost` appear in `run_timing.json` and on a separate `COST:` finalize line after `TIMING:` (`total_cost`, `mean_cost_per_tx`, …).
- `trace.jsonl` uses the same ACP-shaped `direction` / `message` records as non-mini runs (synthetic, not JSON-RPC wire capture). Each OpenRouter HTTP attempt also records a `miniHttpExchange` audit field (status, body capped at 64 KiB, error when present); raw HTTP is never teed to stdout.
- Bash tool summaries on stdout use the same Read / Search / Edit / Run vocabulary as ACP when heuristics match.

Environment variables (mini only):

| Variable | Required | Purpose |
|----------|----------|---------|
| `OPENROUTER_API_KEY` | yes | Bearer token |
| `OPENROUTER_HTTP_REFERER` | no | OpenRouter attribution header |
| `OPENROUTER_BASE_URL` | no | Override API base (testing) |
| `OPENROUTER_REQUEST_TIMEOUT` | no | HTTP timeout in seconds (default 120) |

`malvin models` uses the Cursor agent CLI by default. Use the subcommand flag `malvin models --mini` to list OpenRouter models (no Cursor CLI required). Global `--mini` on other subcommands does not affect `malvin models`.

### `--mini-max-http-turns <N>` (default: 32)

Maximum HTTP completion rounds inside one `run_coder_prompt` when `--mini`. Each round may execute multiple ` ```bash ` blocks before the next OpenRouter call. `--mini-max-bash-turns` is a deprecated hidden alias for the same flag.

### `--mini-max-bash-execs <N>` (default: 128)

Maximum bash fence executions across all HTTP turns in one `run_coder_prompt` when `--mini`.

### `--name <NAME>`

Optional session name for `do`, `code`, `tidy`, `delight`, and bare `malvin REQUEST` (not the hidden `kpop` subcommand). When omitted on those invocations, malvin assigns a unique five-character id (`[a-z0-9]`). Every command that accepts `--name` acquires a session name lock before substantive work.

Malvin registers the top-level process under this name in a per-user registry at `~/.malvin_home/names/<NAME>` (one line: holder PID). If another live malvin process already holds the same name, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin_home/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live ACP session per lock slot in a workspace). Two malvin processes with different `--name` values may both register names and hold live ACP sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/checks/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--help`, `--version`, and bare `malvin` with no `REQUEST` parse `--name` but do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --doc` — this overview.
- `malvin <COMMAND> --doc` — documentation for that subcommand.
- `malvin revise doc.md --doc` — `revise` requires a placeholder `DOC_PATH` (any existing or dummy filename) even with `--doc`.

Other subcommand arguments (for example `<REQUEST>` or `init` languages) are not required when `--doc` is set, except `revise` as noted above.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Bare `malvin REQUEST` (kpop) options

When no subcommand is given, these global flags apply to the kpop workflow (same semantics as `malvin kpop`):

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | How many separate kpop agent runs (each with its own experiment log); code/tidy use config `max_loops_code` (default 3) when unset |
| `--max-hypotheses` | 5 (CLI default; overridden by `[agent].max_hypotheses` in `~/.malvin_home/config.toml` when the flag is omitted) | `## Step … — KPOP` budget per agent run |
| `--tenacious` | on | Sets `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

## Run directories and logs

Every agent-backed command creates `~/.malvin_home/logs/<hash>/<timestamp>_<token>/`. Typical files:

| File | Role |
|------|------|
| `plan.md` or `request.md` | Copy of user input for this run |
| `kpop.log`, `do.log`, `inspire.log`, … | Per-prompt transcripts |
| `stdout.log` | Tee of agent stdout (unless `--no-tee`) — **narrative** channel |
| `trace.jsonl` | ACP-shaped audit record — **authoritative** for semantics (tool results, shrink/fork, LLM usage) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `_kpop/exp_log_*.md` | KPop experiment logs (gate-loop and investigation commands) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Narrative vs audit (trust rule)

Each run writes two parallel channels with different contracts:

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, …). Use for skimming a run and vocabulary/ordering checks.
- **`trace.jsonl` (audit):** machine-authoritative ACP-shaped JSONL (`agent_message_chunk`, `tool_call`, mini-only fields like `miniTerminal`, `miniHttpExchange`). Use for tool exit codes, shrink/fork events, and gate-loop audit tooling.

Consumers must know which file to trust for which question. Named types live in `src/observability/` (`ObservabilityChannel`, `AuditEventKind`).

## Deferred stdout logging

During live ACP sessions, malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log`. Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched from Cursor’s local `store.db` while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral.

## Home config (`~/.malvin_home/config.toml`)

Top-level keys include `mem_limit_gb`, `theme`, and `mpc` (default **true**). When `mpc = true`, agent-backed KPop entry points (`code`, `tidy`, `delight`, `explain`, `revise`, init discovery, bare `malvin REQUEST`, and `kpop`) run an MPC planning-agent session at the start of each outer gate-loop iteration. The planning agent may edit the on-disk user brief at `user_request_path`; when the brief contains `## MPC_DONE` and quality gates pass (for workflows that already run `.malvin/checks`), malvin exits the outer loop without running the KPop implementer for that iteration. Per-iteration logs go to `mpc_planner_{n}.log` and append to `_kpop/mpc_planner_log.md` in the run directory. While `mpc = true`, the consecutive `## KPOP_SOLVED` early-exit path is disabled; with `mpc = false`, behavior is unchanged.

## Log retention

Before most agent-backed commands create a new run directory, malvin may prune older directories under `~/.malvin_home/logs/<hash>/` according to `~/.malvin_home/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). Set `max_count = 0` for unlimited run count (byte and age caps still apply). `malvin init` skips pruning. Use `malvin logs status` to inspect retention state and `malvin logs gc` (with optional `--dry-run`) to prune manually without starting an agent session. `malvin init` and agent-backed commands (including `malvin do`, `code`, and `tidy`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command or `malvin logs gc` may delete excess oldest runs once.

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for `malvin models` without `--mini`, and for agent subcommands).
- **OpenRouter** (when `--mini` or `malvin models --mini`): `OPENROUTER_API_KEY` for completions; listing works without a key. Network access required for live fetches.
- **`bash` on `PATH`** (when `--mini`): required on Linux and macOS; Windows native is not supported in v1 (use WSL).
- **kiss**: required before `code` and `tidy` start; installed/configured by `init`.
- **pre-commit**: installed and hooked by `init`.

## Request syntax

Several commands accept a positional request. `<REQUEST>` is always exactly **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| `code`, `do`, `kpop`, `inspire`, bare `malvin` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

### Sequential requests

`malvin` and `malvin code` accept **multiple** positional arguments. Malvin runs each request as a separate invocation in order, waiting for each to finish before starting the next. Each run gets its own directory under `~/.malvin_home/logs/<hash>/`. This matches calling `malvin` (or `malvin code`) once per argument from the shell.

Examples:

```text
malvin do "fix the typo"
malvin code plan.md
malvin code plan_1.md plan_2.md plan_3.md
malvin "Why does the cache miss?"          # bare kpop
malvin req_1.md req_2.md req_3.md          # bare kpop, sequential
malvin kpop notes/question.md
```

## Gate-loop commands (shared pattern)

`code`, `tidy`, `delight`, `explain`, and `revise` share an outer **gate loop** implemented in `kpop_engine`:

1. For each outer iteration (budget: `effective_max_loops(--max-loops) + 1` iterations), malvin may run one KPop agent session scoped by that command’s constraints file (`code_constraints.md`, `tidy_constraints.md`, `delight_constraints.md`, `explain_constraints.md`, or `revise_constraints.md`) rendered through `kpop_program.md`.
2. The agent records hypotheses in `~/.malvin_home/logs/<hash>/<run>/_kpop/exp_log_<n>.md`.
3. Malvin exits early when **two consecutive** sessions write `## KPOP_SOLVED` and workspace quality gates pass (`code` / `tidy`). Document workflows (`delight`, `explain`, `revise`) use the same loop machinery but do not require passing workspace gates for exit.
4. Otherwise the loop continues until the outer budget is exhausted; `code` rechecks gates after exhaustion, `tidy` may exit without recheck depending on configuration.

See `malvin code --doc`, `malvin tidy --doc`, `malvin delight --doc`, `malvin explain --doc`, and `malvin revise --doc` for command-specific behavior.
