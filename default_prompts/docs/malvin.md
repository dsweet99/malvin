# malvin (top-level CLI)

malvin is a non-interactive CLI agent that drives the Cursor ACP (`cursor-agent` or `agent`) against a workspace. Each agent-backed invocation creates an isolated run directory under `~/.malvin_home/logs/<hash>/` and records prompts, stdout, and artifacts there.

## How to read this documentation

- **Humans:** skim **Commands**, then open `malvin <COMMAND> --doc` for the workflow you need.
- **Agents:** treat each `--doc` file as a self-contained contract for that command; global flags and run-directory rules live in this file.
- **Help vs doc:** `malvin --help` lists flags; `--doc` explains behavior, logs, and when to use each command.

## Usage

```text
malvin [OPTIONS] [REQUEST]
malvin [OPTIONS] <COMMAND>
```

Bare `malvin REQUEST` runs autonomous routing (decides among `kpop` and `inspire`). Use subcommands for named workflows. For KPop investigation, use `malvin kpop REQUEST`.

Use subcommands: `kpop`, `do`, `inspire`, `tidy`, `delight`, `priors`, `explain`, `revise`, `models`, `logs`.

## Commands

| Command | Purpose |
|---------|---------|
| *(default)* | Bare `malvin REQUEST` — autonomous routing among `kpop` / `inspire` (`--max-loops`, `CONTINUE_ROUTER`) |
| `kpop` | KPop investigation (Popperian hypothesis loop) |
| `do` | One-shot agent turn (non-looping) |
| `inspire` | One-shot MBC2 boundary exploration (batch ideation) |
| `tidy` | Fix quality gates via the KPop gate loop (`tidy_constraints.md`) |
| `delight` | Author a user-delighting feature pitch via the KPop gate loop |
| `priors` | Ground a request in good priors via the KPop gate loop |
| `explain` | Explain code or concepts as a LaTeX PDF via the KPop gate loop |
| `revise` | Revise an existing document in place via the KPop gate loop |
| `models` | List models via the Cursor agent CLI |
| `logs` | Inspect and prune run-log retention under `~/.malvin_home/logs/` |

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

By default the bare route and gate-loop commands (`kpop`, `tidy`, `delight`, `priors`, `explain`, `revise`) expand to `--max-loops=9999` and `--max-acp-retries=9999`. `--no-tenacious` restores normal loop/retry budgets.

### `--no-tee`

By default malvin tees agent stdout to the terminal (and `stdout.log` in the run dir). `--no-tee` suppresses live streaming; logs are still written under `~/.malvin_home/logs/`.

### `--no-markdown`

Disable styled markdown rendering of agent stdout for agent-backed subcommands that use the shared ACP client (`kpop`, `tidy` when the agent runs, `delight`, `priors`, `explain`, `revise`, `inspire`). No effect on `models`. **`do` uses plain stdout** on a TTY regardless of this flag; piped `do` output is always plain.

### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown.

### `--max-acp-retries <N>` (default: 3)

Maximum bounded attempts per ACP spawn or `session/prompt`, with 1s / 3s backoff between tries. `--tenacious` on gate-loop commands sets this to 9999.

### `--name <NAME>`

Optional session name for bare `malvin REQUEST`, `kpop`, `do`, `tidy`, `delight`, and `priors`. When omitted on those invocations, malvin assigns a unique five-character id (`[a-z0-9]`). Every command that accepts `--name` acquires a session name lock before substantive work.

Malvin registers the top-level process under this name in a per-user registry at `~/.malvin_home/names/<NAME>` (one line: holder PID). If another live malvin process already holds the same name, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin_home/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live ACP session per lock slot in a workspace). Two malvin processes with different `--name` values may both register names and hold live ACP sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/checks/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--help`, `--version`, and `malvin` with no subcommand parse `--name` but do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --doc` — this overview.
- `malvin <COMMAND> --doc` — documentation for that subcommand.
- `malvin revise doc.md --doc` — `revise` requires a placeholder `DOC_PATH` (any existing or dummy filename) even with `--doc`.

Other subcommand arguments (for example `<REQUEST>`) are not required when `--doc` is set, except `revise` as noted above.

## Quality gates (`.malvin/checks`)

Only **`malvin tidy`** requires `.malvin/checks` at gate-loop time. Use **`malvin init`** to discover and write `.malvin/checks` explicitly (KPop session from `init_constraints.md`). At `tidy` startup, when the checks file is missing or contains no command lines, malvin runs the same checks-discovery session, then aborts if the agent did not write a checks file with at least one command. Delete `.malvin/checks` to trigger discovery again on the next `init` or `tidy` run.

`tidy` runs workspace quality gates from `.malvin/checks` at the repo git root (one shell command per non-empty, non-comment line). Full-line comments starting with `#` are ignored. Mid-loop gate iterations do **not** run discovery; they error if checks are absent.

Other commands (`do`, bare `malvin REQUEST`, `kpop`, `inspire`, `delight`, `priors`, `explain`, `revise`) do not require `.malvin/checks` at startup and may run outside a git repo. `header.md` notes about checks lines are advisory when a workspace happens to have gates; they are not a startup requirement for those commands.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## `kpop` options

See `malvin kpop --doc`. Key flags:

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | How many separate kpop agent runs (each with its own experiment log); tidy uses config `max_loops_code` (default 3) when unset |
| `--tenacious` | on | Sets `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

## Run directories and logs

Every agent-backed command creates `~/.malvin_home/logs/<hash>/<timestamp>_<token>/`. Typical files:

| File | Role |
|------|------|
| `plan_<random>.md` or `request.md` | Copy of user input for this run |
| `kpop.log`, `do.log`, `router_1.log`, `router_2.log`, `inspire.log`, … | Per-iteration or per-prompt transcripts |
| `stdout.log` | Tee of agent stdout (unless `--no-tee`) — **narrative** channel |
| `trace.jsonl` | ACP-shaped audit record — **authoritative** for semantics (tool results, shrink/fork, LLM usage) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `_kpop/exp_log_*.md` | KPop experiment logs (gate-loop and investigation commands) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Narrative vs audit (trust rule)

Each run writes two parallel channels with different contracts:

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, …). Use for skimming a run and vocabulary/ordering checks.
- **`trace.jsonl` (audit):** machine-authoritative ACP-shaped JSONL (`agent_message_chunk`, `tool_call`, and other audit fields). Use for tool exit codes, shrink/fork events, and gate-loop audit tooling.

Consumers must know which file to trust for which question. Named types live in `src/observability/` (`ObservabilityChannel`, `AuditEventKind`).

## Deferred stdout logging

During live ACP sessions, malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log`. Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched from Cursor’s local `store.db` while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral.

## Home config (`~/.malvin_home/config.toml`)

Top-level keys include `mem_limit_gb` and `theme`.

## Log retention

Before most agent-backed commands create a new run directory, malvin may prune older directories under `~/.malvin_home/logs/<hash>/` according to `~/.malvin_home/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). Set `max_count = 0` for unlimited run count (byte and age caps still apply). Use `malvin logs status` to inspect retention state and `malvin logs gc` (with optional `--dry-run`) to prune manually without starting an agent session. Agent-backed commands (including `malvin do` and `tidy`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command or `malvin logs gc` may delete excess oldest runs once.

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for `malvin models` and agent subcommands).
- **pre-commit**: optional; malvin does not install hooks automatically.

## Request syntax

Several commands accept a positional request. `<REQUEST>` is always exactly **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| bare `malvin REQUEST`, `do`, `kpop`, `inspire` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

### Sequential requests

`malvin kpop` accepts **multiple** positional arguments. Malvin runs each request as a separate invocation in order, waiting for each to finish before starting the next. Each run gets its own directory under `~/.malvin_home/logs/<hash>/`. This matches calling `malvin kpop` once per argument from the shell.

Examples:

```text
malvin do "fix the typo"
malvin kpop "Why does the cache miss?"
malvin kpop req_1.md req_2.md req_3.md
malvin kpop notes/question.md
```

## Gate-loop commands (shared pattern)

`tidy`, `delight`, `priors`, `explain`, and `revise` share an outer **gate loop** implemented in `kpop_engine`:

1. For each outer iteration (budget: `effective_max_loops(--max-loops) + 1` iterations), malvin may run one KPop agent session. Scope comes from that command’s constraints file (`tidy_constraints.md`, etc.) rendered through `kpop_program.md` into `request.md`. Within the session, malvin sends one prompt: `header.md` + `kpop_common.md` (Popper loop).
2. Hypotheses and test results go to `~/.malvin_home/logs/<hash>/<run>/_kpop/exp_log_<n>.md`.
3. Malvin exits early when workspace quality gates pass (`tidy`). Document workflows (`delight`, `priors`, `explain`, `revise`) run until `--max-loops` is exhausted. `kpop` investigation runs until its own loop budget is exhausted.
4. Otherwise the loop continues until the outer budget is exhausted; `tidy` may exit without recheck depending on configuration.

See `malvin tidy --doc`, `malvin delight --doc`, `malvin priors --doc`, `malvin explain --doc`, and `malvin revise --doc` for command-specific behavior.
