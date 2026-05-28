# malvin (top-level CLI)

malvin is a non-interactive CLI agent that drives the Cursor ACP (`cursor-agent` or `agent`) against a workspace. Each invocation creates an isolated run directory under `./.malvin/logs/` in the workspace (or target path) and logs prompts, stdout, and artifacts there.

## Usage

```text
malvin [OPTIONS] [<COMMAND> | REQUEST]
```

Bare invocation (no subcommand):

- `malvin REQUEST` — KPop investigation (same as legacy `malvin kpop REQUEST`)

Use subcommands for other workflows: `do`, `code`, `constrain`, `tidy`, etc.

## Commands

| Command | Purpose |
|---------|---------|
| `init` | Bootstrap a repo with malvin templates and tooling |
| `do` | One-shot agent turn (non-looping) |
| `invent` | One-shot MBC2 boundary exploration (batch ideation from `mbc2.md`) |
| `code` | Implement a plan with review loop |
| `constrain` | Regression test first, then implementation |
| `tidy` | Fix quality gates and related workspace issues |
| `models` | List models available from the Cursor agent CLI |

Hidden (backward compatible): `kpop`.

See the matching doc in this directory: `init.md`, `do.md`, `invent.md`, `code.md`, `constrain.md`, `kpop.md`, `tidy.md`, `models.md`.

## Global options

These flags are **global**: they may appear before or after the subcommand name.

### `--no-color`

Disable ANSI color on malvin’s own status and error lines. Does not change the agent’s raw stream.

### `--model <MODEL>`

Model id passed to the Cursor agent for subcommands that spawn an agent session. Default: `auto` (see `models` for the CLI default malvin prints).

### `--no-force`

By default malvin passes `--force` to `cursor-agent` so tool calls proceed without interactive approval. `--no-force` disables that (agent may wait for user approval in the IDE).

### `--no-tee`

By default malvin tees agent stdout to the terminal (and `stdout.log` in the run dir). `--no-tee` suppresses live streaming; logs are still written under `./.malvin/logs/`.

### `--no-markdown`

Disable styled markdown rendering of agent stdout for agent-backed subcommands that use the shared ACP client (`code`, `kpop`, `tidy` when the agent runs, `invent`, and the `init` summary phase). No effect on `models` (no agent session). Note: **`do` always uses plain stdout** regardless of this flag.

### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a `./.malvin/logs` run directory.

- `malvin --doc` — top-level overview (`default_prompts/docs/malvin.md`, this file).
- `malvin <COMMAND> --doc` — documentation for that subcommand (`default_prompts/docs/<command>.md`).

Other subcommand arguments (for example `<REQUEST>` or `init` languages) are not required when `--doc` is set.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Run directories and logs

Every agent-backed command creates `./.malvin/logs/<timestamp>_<token>/` under the session work directory. Typical files:

- `plan.md` or `request.md` — copy of the user input for this run
- `do.log`, `code` phase logs, `kpop.log`, etc. — per-prompt transcripts
- `stdout.log` — tee of agent stdout (unless `--no-tee`)
- `prompts.log` — outgoing prompts (names only, or full bodies with `--verbose`)
- `quality_gates.log` — workspace gate commands and output when gates run
- `review.md`, `review_prep.md`, `result.md` — review and abort artifacts for coding workflows

## Deferred stdout logging

During live ACP sessions (`code`, `kpop`, `tidy`, and similar agent-backed flows), malvin may defer agent stdout lines briefly before writing them to the terminal and `stdout.log`. Each line waits until it has been queued for at least **`max_age`** (default **1000ms**, env `MALVIN_DEFER_LOG_MAX_AGE_MS`) so tool summaries can be enriched from Cursor’s local `store.db` while preserving FIFO order. Set `MALVIN_DEFER_LOG=0` to disable deferral. Heartbeats during an active defer session go through the same defer sink (including the wall-clock poller) so `stdout.log` and the terminal stay in FIFO order with agent output.

## Log retention

Before most agent-backed commands create a new run directory, malvin may prune older directories under `./.malvin/logs/` according to `.malvin/config.toml` `[logs]` settings (`max_age_days`, `max_runs`, `max_bytes`). `malvin init` and `malvin do` skip this pruning. `malvin init` seeds the config file with defaults.

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for all agent subcommands and `models`).
- **kiss**: required before `code` and `tidy` start; also installed/configured by `init`.
- **pre-commit**: installed and hooked by `init`.

## Request syntax

Several commands accept a positional request.

- **`code`:** pass an existing `.md` file path (no whitespace; case-sensitive `.md` suffix) to read that file; work dir is its parent. Otherwise the argument is literal text (including nonexistent `.md` paths).
- **Other commands (`do`, `kpop`, …):** prefix with `@` to read text from a file; work dir is the file’s parent.

Examples:

- `malvin do "fix the typo"` — work dir `.`, request is literal text
- `malvin code plan.md` — read `plan.md`, work dir is its parent
- `malvin @notes/request.md` — KPop stores copy as `request.md` in the run dir (bare kpop)
