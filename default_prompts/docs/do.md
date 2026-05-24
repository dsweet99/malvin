# malvin do

Respond to a **single** user request in one agent session. This is malvin’s lightweight, non-looping mode: no plan check, no review fan-out, no implement/review cycle.

## Intention

Answer a question, perform a one-off task, or continue an informal conversation without the full `code` pipeline. Output is plain text on stdout (no markdown styling), suitable for terminals and pipes.

## Usage

```text
malvin do [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

The user’s task as literal text, or `@<path>` to read from a file.

- Literal: work directory is `.` (cwd).
- `@file`: file contents become the request; work directory is the file’s parent.
- Run copy is stored as `plan.md` under `./.malvin/logs/<run>/`.

## Options

### `--repo-gates`

Before the agent runs, execute workspace quality gates (commands from `.malvin/checks`, with `kiss clamp` preparation). Uses the no-clamp variant for gates only in the sense that this path calls `run_repo_workspace_gates_no_kiss_clamp` so gates do not implicitly create `.kissconfig`. Failure aborts before any prompt.

### `--thoughts`

When the agent emits “thought” tokens, stream them to stdout as well (in addition to normal output).

### Global options

See `malvin.md`. **`do` ignores `--no-markdown`** for agent output: stdout is always raw/plain. `--verbose` logs full prompt bodies. `--no-tee` disables live streaming.

## Prompt workflow

Exactly **one** coder prompt per invocation.

| Step | Prompt role (effect) | Log |
|------|----------------------|-----|
| 1 | **Coding header** (`header.md`) — Full malvin coding context (identity, history/memory guidance, repo rules). | `do.log` |
| 2 | **Do header** (`do_header.md`) — Tells the agent it is malvin in do mode; plaintext replies; points at `./.malvin/logs/.../do.log` for prior do sessions. | `do.log` |
| — | **User request** — Appended after both headers (not a separate file). | `do.log` |

No `implement`, `review`, `concerns`, `learn`, or `summary` phases.

## Session behavior

- Backs up `.kissconfig`, `.kissignore`, `.malvin/checks` before the session; restores after.
- Writes `do.log`, `stdout.log`, `prompts.log`, timing JSON as applicable.
- Checks `result.md` for ABORT after the session (coding-style abort protocol).

## When to use

- Quick questions or small edits without a written plan.
- Piped/redirected workflows that need clean plaintext.
- Inspecting or continuing prior work via recent `./.malvin/logs/*/do.log` (agent is steered to look there in the do header).

## Examples

```text
malvin do "List failing tests and suggest fixes"
malvin do @notes/task.md
malvin do --repo-gates "Refactor foo.rs to use Result"
```
