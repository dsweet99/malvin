# malvin (default route)

Two-prompt agent sessions with autonomous routing: `header.md` + `router.md` + user request, then bare `router_b.md` on the same session. Outer `--max-loops` restarts with a new agent when `router_b` replies `CONTINUE_ROUTER`.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `kpop` / `tidy`) |
| Logs | `router.log` and `router_b.log` under `~/.malvin_home/logs/<hash>/<run>/` |
| Requires | No `.malvin/checks` at startup |

## Intention

Let the agent read the user request and decide how to satisfy it — including invoking other malvin commands such as `kpop` or `inspire`. Suitable when the right workflow is not known upfront.

After the work prompt, `router_b.md` asks for either an evidence report or the literal token `CONTINUE_ROUTER` to request another agent session (until `--max-loops` is exhausted).

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
| Literal | `.` (cwd) | `plan.md` in run dir |
| `path/to/file.md` | Parent of file | `plan.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--max-loops` | Outer router sessions (default `1`; tenacious expands to `9999`) |
| `--no-tenacious` | Keep default `--max-loops=1` and normal `--max-acp-retries` |
| `--no-tee` | Disables live streaming |
| `--verbose` | Full prompt bodies in `prompts.log` |

## Prompt workflow

Each outer loop iteration opens one coder session and sends two prompts:

| Turn | Piece | Role |
|------|-------|------|
| 1 | `header.md` | Standard Malvin coding context (log-reading, calibration, sandbox rules) |
| 1 | `router.md` | Autonomous routing brief; points agent at `kpop` / `inspire` |
| 1 | User request | Appended after headers |
| 2 | `router_b.md` | Bare follow-up: evidence report **or** `CONTINUE_ROUTER` |

When the agent’s `router_b` reply contains a line exactly equal to `CONTINUE_ROUTER` (or the whole reply trimmed is `CONTINUE_ROUTER`), malvin tears down the session and starts a new outer loop with the same `router.md` assembly. Otherwise the run finishes.

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `logs_dir` | `header.md` | `malvin_logs_root(work_dir)` |
| `current_state` | `header.md` | `format_current_state(...)` |
| `user_request_path` | `router.md` | `format_prompt_path(plan_path, work_dir)` |
| `malvin_command` | metadata | literal `"router"` |

No implement, review, concerns, learn, or summary phases.

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `do`).
- Backs up `.gitignore`, `.malvin/checks`, `.malvin/config.toml`, and `~/.malvin_home/config.toml` before each outer loop iteration; restores session dotfiles after each iteration and at run end.
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
