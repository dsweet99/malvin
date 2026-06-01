# malvin do

One **single-turn** agent session: no gate loop, no KPop experiment log, no review fan-out.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or `@file` |
| Output | Plain stdout (no markdown styling) |
| Log | `do.log` under `./.malvin/logs/<run>/` |

## Intention

Answer a question, perform a one-off task, or continue informal work without the `code` pipeline. Suitable for terminals and pipes.

## Usage

```text
malvin do [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Literal text, or `@<path>` to read from a file.

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan.md` in run dir |
| `@file` | Parent of file | `plan.md` |

## Options

### `--repo-gates`

Before the agent runs, execute workspace quality gates from `.malvin/checks` (via `run_repo_workspace_gates_no_kiss_clamp`). Failure aborts before any prompt.

### `--thoughts`

Stream agent “thought” tokens to stdout in addition to normal output.

## Global options

See `malvin --doc`. Notable for `do`:

| Flag | Effect on `do` |
|------|----------------|
| `--no-markdown` | Ignored for styling — stdout is always plain |
| `--no-tee` | Disables live streaming |
| `--verbose` | Full prompt bodies in `prompts.log` |

## Prompt workflow

One coder prompt per invocation:

| Piece | Role |
|-------|------|
| `header_do.md` | Malvin coding context without log-reading mandates |
| `do_header.md` | Do-mode persona; direct answer only |
| User request | Appended after headers |

No implement, review, concerns, learn, or summary phases.

## Session behavior

- Backs up `.kissconfig`, `.kissignore`, `.malvin/checks`; restores after.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin code` | Multi-iteration plan implementation |
| `malvin kpop` | Hypothesis-driven investigation with `_kpop/` log |

## Examples

```text
malvin do "List failing tests and suggest fixes"
malvin do @notes/task.md
malvin do --repo-gates "Refactor foo.rs to use Result"
```
