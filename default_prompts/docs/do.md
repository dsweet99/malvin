# malvin --do

One **single-turn** agent session: no gate loop, no KPop experiment log, no review fan-out.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Default: plain stdout with only text between `MALVIN_DM_START` / `MALVIN_DM_END`. With `--verbose`: same agent log classes as the default workflow (thought tokens, narrative tee, full outgoing prompts). |
| Log | `do.log` under `~/.malvin_home/logs/<hash>/<run>/` |
| Requires | No `.malvin/checks` at startup |

## Intention

Answer a question, perform a one-off task, or continue informal work without a gate-loop pipeline. Suitable for terminals and pipes.

## Usage

```text
malvin --do [OPTIONS] [REQUEST]
```

If `REQUEST` is omitted (and `--doc` is not set), malvin prints short usage on stdout and exits 0.

## Arguments

### `[REQUEST]`

Required to run. Exactly **one shell argument**. Quote for internal spaces (e.g. `malvin --do "fix the typo"`). Literal text, or an existing `.md` file path (same rules as bare `malvin REQUEST`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for `--do`:

| Flag | Effect on `--do` |
|------|----------------|
| `--quiet` / `-q` | Not needed without `--verbose`: `--do` is already DM-body-only |
| `--verbose` / `-v` | Same stdout log classes as the default workflow (thoughts, narrative tee, full prompt bodies); also full bodies in `prompts.log` |
| `-b` / `--background` | Suppresses all stdout, including DM bodies |

## Prompt workflow

One coder prompt per invocation:

| Piece | Role |
|-------|------|
| `header.md` | Standard Malvin coding context (log-reading, calibration, sandbox rules) |
| `do_header.md` | Do-mode persona; direct answer only |
| User request | Appended after headers |

No implement, review, concerns, learn, or summary phases.

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `tidy`).
- Backs up `.gitignore`, `.malvin/checks`, and `.malvin/config.toml`; restores after the session.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin --do Hello` | One-turn agent connectivity smoke check |

## Examples

```text
malvin --do Hello
malvin --do "List failing tests and suggest fixes"
malvin --do notes/task.md
malvin --verbose --do "Show the full agent stream"
```
