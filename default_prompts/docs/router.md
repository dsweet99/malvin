# malvin (default route)

Four-prompt agent sessions with autonomous routing: `header.md` + `router_a_1.md` + user request, then bare `router_a_2.md`, then bare `router_b_simple.md` or `router_b_complex.md`, then bare `router_c.md` on the same session. Outer `--max-loops` restarts with a new agent when `router_c` replies `CONTINUE_ROUTER`.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `kpop` / `tidy`) |
| Logs | `router_1.log`, `router_2.log`, … under `~/.malvin_home/logs/<hash>/<run>/` (all turns in one file per outer loop) |
| Requires | No `.malvin/checks` at startup |

## Intention

Let the agent read the user request and decide how to satisfy it — including invoking other malvin commands such as `kpop` or `inspire`. Suitable when the right workflow is not known upfront.

Turn 1 (`router_a_1`) classifies complexity. Turn 2 (`router_a_2`) classifies whether the task is coding. When `CODING_TASK: YES` and `.malvin/checks` is missing, malvin runs `init` (separate kpop subprocess) before turn 3. After the work prompts, `router_c.md` asks for either an evidence report or the literal token `CONTINUE_ROUTER` to request another agent session (until `--max-loops` is exhausted).

## Usage

```text
malvin [OPTIONS] <REQUEST>
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow.

## Arguments

### `<REQUEST>` (required)

Exactly **one shell argument**. Quote for internal spaces. Literal text, or an existing `.md` file path (same rules as `do`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--max-loops` | Outer router sessions (default `1`; tenacious expands to `9999`) |
| `--no-tenacious` | Keep default `--max-loops=1` and normal `--max-acp-retries` |
| `--no-tee` | Disables live streaming |
| `--verbose` | Full prompt bodies in `prompts.log` |

## Prompt workflow

Each outer loop iteration opens one coder session and sends four prompts:

| Turn | Piece | Role |
|------|-------|------|
| 1 | `header.md` | Standard Malvin coding context (log-reading, calibration, sandbox rules) |
| 1 | `router_a_1.md` | Classify complexity (`COMPLEXITY_SCORE: 1-10`) |
| 1 | User request | Appended after headers |
| 2 | `router_a_2.md` | Bare classify coding (`CODING_TASK: YES\|NO`) |
| 3 | `router_b_simple.md` or `router_b_complex.md` | Bare work brief (`COMPLEXITY_SCORE <= 3` → simple; `> 3` → complex) |
| 4 | `router_c.md` | Bare follow-up: evidence report **or** `CONTINUE_ROUTER` |

After turn 1, malvin parses `COMPLEXITY_SCORE` from the agent response (alone on its own line). Parse failure aborts the run immediately. After turn 2, malvin parses `CODING_TASK` the same way. Dotfile backup runs after that parse, before turn 3.

When the agent’s `router_c` reply contains a line exactly equal to `CONTINUE_ROUTER` (or the whole reply trimmed is `CONTINUE_ROUTER`), malvin tears down the session and starts a new outer loop with the same `router_a_1` assembly. Otherwise the run finishes.

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `logs_dir` | `header.md` | `malvin_logs_root(work_dir)` |
| `current_state` | `header.md` | `format_current_state(...)` |
| `user_request_path` | `router_a_1.md` | `format_prompt_path(plan_path, work_dir)` |
| `malvin_command` | metadata | `malvin --model=<active_model>` (e.g. `malvin --model=cursor:auto`) |

No implement, review, concerns, learn, or summary phases.

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `do`).
- Backs up `.gitignore`, `.malvin/checks`, `.malvin/config.toml`, and `~/.malvin_home/config.toml` after `router_a_2` parsing (before turn 3); restores session dotfiles after each iteration and at run end.
- When `CODING_TASK: YES` and checks are missing, runs `malvin init` via a separate kpop agent between turns 2 and 3 (coder session stays open).
- Checks `result.md` for `ABORT:` after the outer loop completes.

## Related commands

| Command | When |
|---------|------|
| `malvin do` | One-turn direct answer without routing brief |
| `malvin kpop` | Hypothesis-driven investigation with `_kpop/` log |
| `malvin inspire` | Creative ideation without routing |

## Examples

```text
malvin "Figure out why tests fail and fix them"
malvin --max-loops 3 notes/task.md
malvin --no-tenacious "Quick one-shot route"
```
