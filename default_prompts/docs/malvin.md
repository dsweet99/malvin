# malvin (top-level CLI)

malvin is a non-interactive CLI agent that drives the Cursor ACP (`cursor-agent` or `agent`) against a workspace. Each invocation creates an isolated run directory under `_malvin/` in the workspace (or target path) and logs prompts, stdout, and artifacts there.

## Usage

```text
malvin [OPTIONS] <COMMAND>
```

## Commands

| Command | Purpose |
|---------|---------|
| `init` | Bootstrap a repo with malvin templates and tooling |
| `do` | One-shot agent turn for a single user request |
| `invent` | One-shot MBC2 boundary exploration (batch ideation from `mbc2.md`) |
| `code` | Implement a plan with review and optional learn loop |
| `kpop` | Popperian scientific investigation (hypothesis-driven experiment log) |
| `hunt` | Find a serious bug via KPOP, then regression-test and fix (experimental) |
| `tidy` | Fix workspace until quality gates pass |
| `plan` | Write or review a plan file (experimental) |
| `models` | List models available from the Cursor agent CLI |

See the matching doc in this directory: `init.md`, `do.md`, `invent.md`, `code.md`, `kpop.md`, `bug.md`, `tidy.md`, `plan.md`, `models.md`.

## Global options

These flags are **global**: they may appear before or after the subcommand name.

### `--no-color`

Disable ANSI color on malvin’s own status and error lines. Does not change the agent’s raw stream.

### `--model <MODEL>`

Model id passed to the Cursor agent for subcommands that spawn an agent session. Default: `auto` (see `models` for the CLI default malvin prints).

### `--no-force`

By default malvin passes `--force` to `cursor-agent` so tool calls proceed without interactive approval. `--no-force` disables that (agent may wait for user approval in the IDE).

### `--no-tee`

By default malvin tees agent stdout to the terminal (and `stdout.log` in the run dir). `--no-tee` suppresses live streaming; logs are still written under `_malvin/`.

### `--no-markdown`

Disable styled markdown rendering of agent stdout for agent-backed subcommands that use the shared ACP client (`code`, `kpop`, `hunt`, `plan`, `tidy` when the agent runs, and the `init` summary phase). No effect on `models` (no agent session). Note: **`do` and `invent` always use plain stdout** regardless of this flag.

### `-v` / `--verbose`

Log **full** outgoing prompt bodies to stdout and `prompts.log`. Default: only the prompt filename is shown.

### `--doc`

Print built-in documentation and exit. Does not spawn an agent or create a `_malvin` run directory.

- `malvin --doc` — top-level overview (`default_prompts/docs/malvin.md`, this file).
- `malvin <COMMAND> --doc` — documentation for that subcommand (`default_prompts/docs/<command>.md`).

Other subcommand arguments (for example `<REQUEST>` or `init` languages) are not required when `--doc` is set.

### `-h` / `--help`

Print help for the top-level CLI or a subcommand (`malvin <COMMAND> --help`).

### `-V` / `--version`

Print malvin’s version.

## Run directories and logs

Every agent-backed command creates `_malvin/<timestamp>_<token>/` under the session work directory. Typical files:

- `plan.md` or `request.md` — copy of the user input for this run
- `do.log`, `code` phase logs, `kpop.log`, etc. — per-prompt transcripts
- `stdout.log` — tee of agent stdout (unless `--no-tee`)
- `prompts.log` — outgoing prompts (names only, or full bodies with `--verbose`)
- `quality_gates.log` — workspace gate commands and output when gates run
- `review.md`, `review_prep.md`, `result.md` — review and abort artifacts for coding workflows

## External dependencies

- **Cursor agent CLI**: `agent` or `cursor-agent` on `PATH` (required for all agent subcommands and `models`).
- **kiss**: required before `code`, `tidy`, `plan`, and `hunt` start; also installed/configured by `init`.
- **pre-commit**: installed and hooked by `init`.

## Request syntax

Several commands accept a positional request.

- **`code` and `plan`:** pass an existing `.md` file path (no whitespace; case-sensitive `.md` suffix) to read that file; work dir is its parent. Otherwise the argument is literal text (including nonexistent `.md` paths).
- **Other commands (`do`, `kpop`, …):** prefix with `@` to read text from a file; work dir is the file’s parent.

Examples:

- `malvin do "fix the typo"` — work dir `.`, request is literal text
- `malvin code plan.md` — read `plan.md`, work dir is its parent
- `malvin kpop @notes/request.md` — KPOP stores copy as `request.md` in the run dir
