# malvin write

Produce a short, reader-friendly **LaTeX explanation** by starting one agent session: an aggregated initial host prompt (`header.md` + `write_a.md`), then `write_b.md` for the paper.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | `write.tex` and `write.pdf` (or `--out-path`); paths are named in the `write_b` prompt |
| Session | One agent: aggregated `header.md` + `write_a.md` (research → `notes.tex` in the run log dir) → wait → `write_b.md` (LaTeX + PDF from those notes) |
| Exit policy | Both host prompts complete successfully |
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

Required to run. Exactly **one shell argument**. Quote for internal spaces. Topic as literal text, or an existing `.md` file path (same rules as bare `malvin REQUEST`).

When `REQUEST` names an existing `.md` file, the work directory is that file's parent; otherwise the work directory is `.` (cwd). With the default `--out-path`, outputs land in that work directory. A custom `--out-path` resolves against the current working directory instead.

## Prompt workflow

| Turn | Piece | Role |
|------|-------|------|
| 1 (aggregated, at spawn) | `header.md` + `write_a.md` | One host send via `start_coder_session`. Header: standard Malvin context (`--model`, `--git`). `write_a`: research notes → `notes.tex`. |
| 2 | `write_b.md` | Paper + PDF using `--out-path` (and derived `.pdf`) |

Shared agent flags such as `--creative`, `--gates`, and `--no-kpop` are accepted on `write` for CLI uniformity with other agent commands, but they do **not** change write's prompt pieces (those options apply to the default router). `--max-hypotheses` is kept for config/CLI compatibility and is not injected into write prompts.

## Options

### `--out-path <PATH>` (default: `write.tex`)

LaTeX output path. malvin derives the PDF path by replacing the `.tex` extension with `.pdf`. With the default `write.tex`, if either default output already exists in the request work directory, malvin allocates the first free sibling pair (`write_1.tex` / `write_1.pdf`, …) before composing the prompts. For any other `--out-path`, preflight refuses to run when either resolved path already exists.

### `--max-loops <N>` (default: 3)

Kept for CLI compatibility with other gate-loop wrappers. The write session is a fixed two-prompt sequence (aggregated initial + `write_b`) and does not use this budget.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` (and expands `--max-loops` for compatibility).

### `--no-tenacious`

Restore normal retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. `--quiet` / `-q` prints only `__MALVIN_DM_START__`/`END` bodies on stdout (not the same as `-b`).

## Success criteria

All of the following must hold:

1. Preflight passed (default outputs may have been auto-allocated; non-default paths must not have pre-existed).
2. The agent finished the aggregated initial turn (`header` + `write_a`) and then `write_b` without error.

## Related commands

| Command | When |
|---------|------|
| `malvin --creative REQUEST` | Default router with MBC2 creative turns |
| bare `malvin REQUEST` | Default router (multi-turn problem-solving) |

## Examples

```text
malvin write "How does malvin tidy force --gates on the default router?"
malvin write docs/notes.md
malvin write "topic" --out-path docs/paper.tex
```
