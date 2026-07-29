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

Bare `malvin REQUEST` runs autonomous routing (requirements, multi-group KPop, optional work). Use `--do` for a one-shot turn, or subcommands for named workflows.

Use `--do` for a one-shot turn. Use subcommands: `inspire`, `init`, `tidy`, `delight`, `explain`, `models`.

## Commands

| Command | Purpose |
|---------|---------|
| *(default)* | Bare `malvin REQUEST` — requirements JSON → one multi-group KPop → optional work; outer `--max-loops` sessions |
| `--do` | One-shot agent turn (non-looping) |
| `inspire` | One-shot MBC2 boundary exploration (batch ideation) |
| `init` | Discover quality gates and write `.malvin/checks` |
| `tidy` | Fix quality gates via the default router with fixed request `Get the gates to pass.` and `--gates` forced on |
| `delight` | Author a user-delighting feature pitch via a composed default-router request |
| `explain` | Explain code or concepts as a LaTeX PDF via a composed default-router request |
| `models` | List models via the Cursor agent CLI |

Per-command documentation: `malvin <COMMAND> --doc` (embedded from `default_prompts/docs/<command>.md`); for the one-shot workflow use `malvin --do --doc`. The default-route contract (`router.md`) is printed after this overview when you run `malvin --doc`.

## Global options

These flags are **global**: they may appear before or after the subcommand name.


### `-b` / `--background`

Suppress all stdout from malvin and the agent. Run logs under `~/.malvin_home/logs/` are unchanged.

### `-q` / `--quiet`

On the **default router** (bare `malvin REQUEST`, and wrappers that call it: `tidy`, `delight`, `explain`), print only the text between `MALVIN_DM_START` and `MALVIN_DM_END` fences to process stdout. Startup chrome, ACP stream, heartbeats, prompt-name lines, fence markers, and TIMING/COST lines are omitted from stdout. Run-dir logs and stderr are unchanged.

This is **not** the same as `-b` / `--background` (which suppresses all stdout, including DM bodies). It is also **not** required for plain `malvin --do`: without `--verbose`, `--do` is already DM-body-only on stdout. With `--verbose`, `--do` tees the same live agent log classes as the default workflow (see `-v` / `--verbose` below).

### `--model <MODEL>`

Model id passed to the Cursor agent for subcommands that spawn a session. Default: `cursor:auto` (see `malvin models`).

### `--max-loops <N>` (default: 1)

Outer agent-session budget for bare `malvin REQUEST` (`effective_max_loops`). `0` is treated as `1`. Gate-loop wrappers (`tidy`, `delight`, `explain`) expose their own `--max-loops` with a default of `3`.

### `--no-force`

By default malvin passes `--force` to `cursor-agent` so tool calls proceed without interactive approval. `--no-force` disables that (the agent may wait for IDE approval).

### `--no-tenacious`

By default gate-loop commands (`tidy`, `delight`, `explain`) expand to `--max-loops=9999` and `--max-acp-retries=9999`. The bare default route expands both `--max-loops=9999` and `--max-acp-retries=9999` unless the matching flag was set explicitly on the command line. `--no-tenacious` restores normal budgets.

### `--gates`

Inject workspace check command text into agent prompts and, for workflows that use harness gates as loop criteria, treat failures as loop or exit criteria. Off by default. On bare `malvin REQUEST`, `--gates` also runs workspace `.malvin/checks` after each outer agent session: pass stops success; fail continues the outer loop (even when KPop chat said no work remaining); exhausted budget with failing gates fails the run. When work runs, check text is still injected into the work prompt. `malvin tidy` always forces `--gates` on (same harness criteria as bare `malvin REQUEST --gates`). Agent prompts may still include available `.malvin/checks` guidance when this option is off.



### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown. For `malvin --do`, also unlock the same live agent stdout log classes as the default workflow (thought tokens and narrative tee); without `--verbose`, `--do` stays DM-body-only.

### `--max-acp-retries <N>` (default: 3)

Maximum bounded attempts per ACP spawn or `session/prompt`, with 1s / 3s backoff between tries. `--tenacious` on gate-loop commands sets this to 9999.

### `--no-download`

Do not auto-download `local:` models on first use. If the GGUF is missing from `~/.malvin_home/model_cache/`, the run fails instead of fetching it. Use `malvin models download local:<id>` to fetch explicitly.

### `--git`

Allow the agent to run `git commit` by setting `{{ git_extra }}` in prompt templates. Off by default (agents are otherwise steered away from committing).

### `--name <NAME>`

Optional session name for bare `malvin REQUEST`, `--do`, `tidy`, and `delight`. When omitted on those invocations, malvin assigns a unique five-character id (`[a-z0-9]`). Every command that accepts `--name` acquires a session name lock before substantive work.

Malvin registers the top-level process under this name in a per-user registry at `~/.malvin_home/names/<NAME>` (one line: holder PID). If another live malvin process already holds the same name, the new invocation exits immediately with status 1. Stale or abandoned name files left by crashes, `SIGKILL`, or partial writes are reclaimed automatically on the next acquire — no manual cleanup under `~/.malvin_home/names/`.

Session names are independent of the workspace-scoped `.malvin/acp_spawn/<slot>.lock` files (one live ACP session per lock slot in a workspace). Two malvin processes with different `--name` values may both register names and hold live ACP sessions in the same workspace concurrently; only one process may hold each lock slot at a time.

`.malvin/acp_spawn/` holds ephemeral PID lock files at the workspace **git root** when `cwd` is inside a git work tree; outside git, locks and quality-gate lists live under `~/.malvin/acp_spawn/` and `~/.malvin/checks/` (shared). Advice and workspace config copies remain `{cwd}/.malvin/advice.md` and `{cwd}/.malvin/config.toml`. Legacy `{cwd}/.malvin/checks` files are read as a fallback until migrated; new writes always target the resolved root.

Any lock whose holder PID is dead (or whose contents are not a valid PID) is safe to delete manually. Lock files are not version-controlled; if they were accidentally committed, run `git rm -r --cached .malvin/acp_spawn/`. Malvin reclaims stale locks automatically on startup in a workspace (directory sweep after early-exit paths such as `--doc`, bare help, and missing-request short help) and when a slot is acquired; live sessions are never disturbed.

`--doc`, `--help`, `--version`, and `malvin` with no subcommand parse `--name` but do not acquire or release a name lock.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a run directory under `~/.malvin_home/logs/`.

- `malvin --doc` — this overview, then the default-route contract (`router.md`).
- `malvin <COMMAND> --doc` — documentation for that subcommand.
- `malvin --do --doc` — documentation for the one-shot `--do` workflow.

Other subcommand arguments (for example `<REQUEST>`) are not required when `--doc` is set.

## Quality gates (`.malvin/checks`)

Use **`malvin init`** to discover and write `.malvin/checks` explicitly (KPop session from `init_constraints.md`). Delete `.malvin/checks` to trigger discovery again on the next `init` run.

With `--gates` (and always for `malvin tidy`), malvin runs workspace quality gates from `.malvin/checks` at the repo git root (one shell command per non-empty, non-comment line). Full-line comments starting with `#` are ignored.

Other invocations (`--do`, bare `malvin REQUEST`, `inspire`, `delight`, `explain`) do not require `.malvin/checks` at startup and may run outside a git repo. With `--gates`, bare `malvin REQUEST` and `malvin tidy` run workspace gates after each outer session and continue that outer loop when they fail (see the default-route section of `malvin --doc`). Without `--gates` (the default for non-tidy commands), malvin does not run those checks directly on the default route. `header.md` notes about checks lines remain advisory when a workspace happens to have gates; they are not a startup requirement for those commands.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Run directories and logs

Every agent-backed command creates `~/.malvin_home/logs/<hash>/<timestamp>_<token>/`. Typical files:

| File | Role |
|------|------|
| `plan_<random>.md` or `request.md` | Copy of user input for this run |
| `kpop.log`, `do.log`, `router_1.log`, `router_2.log`, `inspire.log`, … | Per-iteration or per-prompt transcripts |
| `stdout.log` | Tee of agent stdout — **narrative** channel |
| `trace.jsonl` | ACP-shaped audit record — **authoritative** for semantics (tool results, shrink/fork, LLM usage) |
| `prompts.log` | Outgoing prompts (names only, or full bodies with `--verbose`) |
| `quality_gates.log` | Workspace gate commands and output when gates run |
| `_kpop/exp_log_*.md` | KPop experiment logs (gate-loop and related workflows) |
| `result.md` | `ABORT:` prefix stops workflows that check it |

### Narrative vs audit (trust rule)

Each run writes two parallel channels with different contracts:

- **`stdout.log` (narrative):** lossy, human-oriented lines with who-tags (`m|`, `t|`, `u|`, `b|`, …). Use for skimming a run and vocabulary/ordering checks.
- **`trace.jsonl` (audit):** machine-authoritative ACP-shaped JSONL (`agent_message_chunk`, `tool_call`, and other audit fields). Use for tool exit codes, shrink/fork events, and gate-loop audit tooling.

Consumers must know which file to trust for which question. Named types live in `src/observability/` (`ObservabilityChannel`, `AuditEventKind`).

## Deferred stdout logging

During live ACP sessions, malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log`. Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched from Cursor’s local `store.db` while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral.

## Home config (`~/.malvin_home/config.toml`)

Top-level keys include `mem_limit_gb`, `context_size` (local llama.cpp `n_ctx`, default 8192), and `theme`. Sections include `[agent]`, `[review]` (legacy explain hypothesis budget; unused by the router wrapper), `[default_workflow]` (`max_hypotheses` for bare `malvin REQUEST` multi-group KPop, default 5), and `[logs]`.

## Log retention

Before most agent-backed commands create a new run directory, malvin may prune older directories under `~/.malvin_home/logs/<hash>/` according to `~/.malvin_home/config.toml` `[logs]` settings (`max_count`, `max_age_days`, `max_bytes`). Set `max_count = 0` for unlimited run count (byte and age caps still apply). Agent-backed commands (including `malvin --do` and `tidy`) ensure the home config file exists with defaults. After upgrading to a build with default `max_count = 1000`, the next GC-enabled command may delete excess oldest runs once.

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for `malvin models` and agent subcommands).
- **pre-commit**: optional; malvin does not install hooks automatically.

## Request syntax

Several commands accept a positional request. `<REQUEST>` is always exactly **one shell argument**; quote it when the text contains spaces. Malvin does not join multiple unquoted shell words into a single request.

| Command | Path argument | Work directory |
|---------|---------------|----------------|
| bare `malvin REQUEST`, `--do`, `inspire` | Existing `.md` file path (no whitespace; case-sensitive `.md` suffix) reads that file; nonexistent `.md` paths are literal text | Parent of the file, or `.` for literal text |

Examples:

```text
malvin --do "fix the typo"
malvin inspire "explore API boundaries"
```

## Gate-loop and router-backed commands

`malvin tidy`, `malvin delight`, and `malvin explain` are thin wrappers: each composes a request and invokes the **default router** (same engine as bare `malvin REQUEST`). Tidy uses the fixed request `Get the gates to pass.` and forces `--gates` on. See `malvin tidy --doc`, `malvin delight --doc`, `malvin explain --doc`, and the default-route section of `malvin --doc`.

