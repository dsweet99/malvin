# malvin (top-level CLI)

malvin is a non-interactive CLI agent that drives the Cursor ACP (`cursor-agent` or `agent`) against a workspace. Each agent-backed invocation creates an isolated run directory under `./.malvin/logs/` in the workspace (or target path) and records prompts, stdout, and artifacts there.

## How to read this documentation

- **Humans:** skim **Commands**, then open `malvin <COMMAND> --doc` for the workflow you need.
- **Agents:** treat each `--doc` file as a self-contained contract for that command; global flags and run-directory rules live in this file.
- **Help vs doc:** `malvin --help` lists flags; `--doc` explains behavior, logs, and when to use each command.

## Usage

```text
malvin [OPTIONS] [<COMMAND> | REQUEST]
```

Bare invocation (no subcommand):

- `malvin REQUEST` — KPop investigation (same as `malvin kpop REQUEST`). `<REQUEST>` is exactly **one shell argument**; quote it when the text contains spaces (e.g. `malvin "Why does the cache miss?"`). Bare `malvin` does **not** join multiple unquoted words into a single request.

Use subcommands for other workflows: `init`, `do`, `inspire`, `plan`, `code`, `tidy`, `models`.

## Commands

| Command | Purpose |
|---------|---------|
| `init` | Bootstrap a repo with malvin templates and tooling |
| `do` | One-shot agent turn (non-looping) |
| `inspire` | One-shot MBC2 boundary exploration (batch ideation) |
| `plan` | Four-prompt planning workflow on a persistent `plan.md` |
| `code` | Implement a plan via the KPop gate loop (`code_constraints.md`) |
| `tidy` | Fix quality gates via the KPop gate loop (`tidy_constraints.md`) |
| `models` | List models available from the Cursor agent CLI |

Hidden (backward compatible): `kpop` — prefer bare `malvin REQUEST` for investigation.

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`).

## Global options

These flags are **global**: they may appear before or after the subcommand name.

### `--no-color`

Disable ANSI color on malvin’s own status and error lines. Does not change the agent’s raw stream.

### `-b` / `--background`

Suppress all stdout from malvin and the agent. Run logs under `./.malvin/logs/` are unchanged.

### `--model <MODEL>`

Model id passed to the Cursor agent for subcommands that spawn a session. Default: `auto` (see `malvin models`).

### `--no-force`

By default malvin passes `--force` to `cursor-agent` so tool calls proceed without interactive approval. `--no-force` disables that (the agent may wait for IDE approval).

### `--no-tenacious`

By default gate-loop commands (`code`, `kpop`, `tidy`, bare `malvin REQUEST`) expand to `--max-loops=9999` and `--max-acp-retries=9999`. `--no-tenacious` restores normal loop/retry budgets.

### `--no-tee`

By default malvin tees agent stdout to the terminal (and `stdout.log` in the run dir). `--no-tee` suppresses live streaming; logs are still written under `./.malvin/logs/`.

### `--no-markdown`

Disable styled markdown rendering of agent stdout for agent-backed subcommands that use the shared ACP client (`code`, `kpop`, `tidy` when the agent runs, `inspire`, and the `init` summary phase). No effect on `models`. **`do` uses plain stdout** on a TTY regardless of this flag; piped `do` output is always plain.

### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown.

### `--max-acp-retries <N>` (default: 3)

Maximum bounded attempts per ACP spawn or `session/prompt`, with 1s / 3s backoff between tries. `--tenacious` on gate-loop commands sets this to 9999.

### `--name <NAME>`

Optional session name for workflow invocations (`init`, `do`, `inspire`, `plan`, `code`, `tidy`, `models`, and bare `malvin REQUEST`). When omitted, malvin assigns a unique five-character id (`[a-z0-9]`).

Malvin registers the top-level process under this name in a per-user registry at `~/.malvin/names/<NAME>` (one line: holder PID). If another live malvin process already holds the same name, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn.lock` (one live ACP session per workspace). Two malvin processes with different `--name` values may both register names in the same workspace; only one may hold a live ACP session there at a time.

`--doc`, `--help`, `--version`, and bare `malvin` with no `REQUEST` parse `--name` but do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a `./.malvin/logs` run directory.

- `malvin --doc` — this overview.
- `malvin <COMMAND> --doc` — documentation for that subcommand.

Other subcommand arguments (for example `<REQUEST>` or `init` languages) are not required when `--doc` is set.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Bare `malvin REQUEST` (kpop) options

When no subcommand is given, these global flags apply to the kpop workflow (same semantics as `malvin kpop`):

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | How many separate kpop agent runs (each with its own experiment log); code/tidy use config `max_loops_code` (default 3) when unset |
| `--max-hypotheses` | 10 | `## Step … — KPOP` budget per agent run |
| `--tenacious` | on | Sets `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

## Run directories and logs

Every agent-backed command creates `./.malvin/logs/<timestamp>_<token>/` under the session work directory. Typical files:

| File | Role |
|------|------|
| `plan.md` or `request.md` | Copy of user input for this run |
| `kpop.log`, `do.log`, `ideas.log`, … | Per-prompt transcripts |
| `stdout.log` | Tee of agent stdout (unless `--no-tee`) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `_kpop/exp_log_*.md` | KPop experiment logs (gate-loop and investigation commands) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

## Deferred stdout logging

During live ACP sessions, malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log`. Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched from Cursor’s local `store.db` while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral.

## Log retention

Before most agent-backed commands create a new run directory, malvin may prune older directories under `~/.malvin/logs/<hash>/` according to `~/.malvin/config.toml` `[logs]` settings (`max_age_days`, `max_bytes`). `malvin init` and `malvin do` skip pruning. `malvin init` ensures the home config file exists with defaults.

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for agent subcommands and `models`).
- **kiss**: required before `code` and `tidy` start; installed/configured by `init`.
- **pre-commit**: installed and hooked by `init`.

## Request syntax

Several commands accept a positional request. `<REQUEST>` is always exactly **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| `code`, `plan`, `do`, `kpop`, `inspire`, bare `malvin` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

Examples:

```text
malvin do "fix the typo"
malvin code plan.md
malvin "Why does the cache miss?"          # bare kpop
malvin kpop notes/question.md
```

## Gate-loop commands (shared pattern)

`code` and `tidy` share an outer **gate loop** implemented in `gate_kpop_workflow`:

1. For each outer iteration (budget: `effective_max_loops(--max-loops) + 1` iterations), malvin may run one KPop agent session scoped by that command’s constraints file (`code_constraints.md` or `tidy_constraints.md`) rendered through `kpop_program.md`.
2. The agent records hypotheses in `./.malvin/logs/<run>/_kpop/exp_log_<n>.md`.
3. Malvin exits early when **two consecutive** sessions write `## KPOP_SOLVED` and workspace quality gates pass.
4. Otherwise the loop continues until the outer budget is exhausted; `code` rechecks gates after exhaustion, `tidy` may exit without recheck depending on configuration.

See `malvin code --doc` and `malvin tidy --doc` for command-specific behavior.
