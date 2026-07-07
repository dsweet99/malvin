# malvin (default route)

One **single-turn** agent session with autonomous routing brief: no gate loop, no KPop experiment log.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `kpop` / `tidy`) |
| Log | `router.log` under `~/.malvin_home/logs/<hash>/<run>/` |

## Intention

Let the agent read the user request and decide how to satisfy it — including invoking other malvin commands such as `kpop` or `inspire`. Suitable when the right workflow is not known upfront.

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
| `--no-tee` | Disables live streaming |
| `--verbose` | Full prompt bodies in `prompts.log` |

## Prompt workflow

One coder prompt per invocation:

| Piece | Role |
|-------|------|
| `header.md` | Standard Malvin coding context (log-reading, calibration, sandbox rules) |
| `router.md` | Autonomous routing brief; points agent at `kpop` / `inspire` |
| User request | Appended after headers |

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
- Backs up `.gitignore`, `.malvin/checks`, `.malvin/config.toml`, and `~/.malvin_home/config.toml`; restores after the session.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin do` | One-turn direct answer without routing brief |
| `malvin kpop` | Hypothesis-driven investigation with `_kpop/` log |
| `malvin inspire` | Creative ideation without routing |

## Examples

```text
malvin "Figure out why tests fail and fix them"
malvin notes/task.md
```
