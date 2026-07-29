# malvin explain

Produce a short, reader-friendly **LaTeX explanation** by composing a request and running the **default router** workflow (same path as bare `malvin REQUEST`). The composed request embeds the user request and the `.tex` / `.pdf` output paths.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | `explain.tex` and `explain.pdf` (or `--out-path`); paths are named in the composed router request |
| Loop | Default router: requirements JSON → multi-group KPop → optional work; outer `--max-loops` sessions |
| Exit policy | Router success (agent fulfills the composed explain request) |
| Requires | No `.malvin/checks` preflight (document workflow) |

## Intention

Explain code or concepts for a reader who will not read the source. Typical use: `malvin explain "How does the gate loop exit?"` or `malvin explain notes/topic.md`.

## Usage

```text
malvin explain [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Exactly **one shell argument**. Quote for internal spaces. Topic as literal text, or an existing `.md` file path (same rules as `inspire`).

When `REQUEST` names an existing `.md` file, the work directory is that file's parent; otherwise the work directory is `.` (cwd). With the default `--out-path`, outputs land in that work directory. A custom `--out-path` resolves against the current working directory instead.

## Options

### `--out-path <PATH>` (default: `explain.tex`)

LaTeX output path. malvin derives the PDF path by replacing the `.tex` extension with `.pdf`. With the default `explain.tex`, if either default output already exists in the request work directory, malvin allocates the first free sibling pair (`explain_1.tex` / `explain_1.pdf`, …) before composing the router request. For any other `--out-path`, preflight refuses to run when either resolved path already exists.

### `--max-loops <N>` (default: 3)

Outer router session budget (`effective_max_loops`). `0` is treated as `1`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Success criteria

All of the following must hold:

1. Preflight passed (default outputs may have been auto-allocated; non-default paths must not have pre-existed).
2. The default router completed within the `--max-loops` budget.

On success, malvin follows the default router exit reporting.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation |
| `malvin delight` | Author a feature pitch (also a router request wrapper) |
| bare `malvin REQUEST` | Same router engine; explain is a thin request wrapper |

## Examples

```text
malvin explain "How does malvin tidy force --gates on the default router?"
malvin explain docs/notes.md
malvin explain "topic" --out-path docs/paper.tex
```
