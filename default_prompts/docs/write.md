# malvin write

Produce a short, reader-friendly **LaTeX explanation** by starting one agent session and sending two prompts in order: research notes from `write_a.md`, then the paper from `write_b.md`.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | `write.tex` and `write.pdf` (or `--out-path`); paths are named in the `write_b` prompt |
| Session | One agent: `write_a` (research → `notes.tex` in the run log dir) → wait → `write_b` (LaTeX + PDF from those notes) |
| Exit policy | Both prompts complete successfully |
| Requires | No `.malvin/gates` preflight (document workflow) |

## Intention

Write about code or concepts for a reader who will not read the source. Typical use: `malvin write "How does the gate loop exit?"` or `malvin write notes/topic.md`.

## Usage

```text
malvin write [OPTION]... [REQUEST]
```

If `REQUEST` is omitted (and `--doc` is not set), malvin prints short usage on stdout and exits 0.

## Arguments

### `[REQUEST]`

Required to run. Exactly **one shell argument**. Quote for internal spaces. Topic as literal text, or an existing `.md` file path (same rules as `inspire`).

When `REQUEST` names an existing `.md` file, the work directory is that file's parent; otherwise the work directory is `.` (cwd). With the default `--out-path`, outputs land in that work directory. A custom `--out-path` resolves against the current working directory instead.

## Options

### `--out-path <PATH>` (default: `write.tex`)

LaTeX output path. malvin derives the PDF path by replacing the `.tex` extension with `.pdf`. With the default `write.tex`, if either default output already exists in the request work directory, malvin allocates the first free sibling pair (`write_1.tex` / `write_1.pdf`, …) before composing the prompts. For any other `--out-path`, preflight refuses to run when either resolved path already exists.

### `--max-loops <N>` (default: 3)

Kept for CLI compatibility with other gate-loop wrappers. The write session is a fixed two-prompt sequence and does not use this budget.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` (and expands `--max-loops` for compatibility).

### `--no-tenacious`

Restore normal retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. `--quiet` / `-q` prints only `__MALVIN_DM_START__`/`END` bodies on stdout (not the same as `-b`).

## Success criteria

All of the following must hold:

1. Preflight passed (default outputs may have been auto-allocated; non-default paths must not have pre-existed).
2. The agent finished `write_a` and then `write_b` without error.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation |
| bare `malvin REQUEST` | Default router (multi-turn problem-solving) |

## Examples

```text
malvin write "How does malvin tidy force --gates on the default router?"
malvin write docs/notes.md
malvin write "topic" --out-path docs/paper.tex
```
