# malvin explain

Produce a short, reader-friendly **LaTeX explanation** via the KPop gate loop scoped by `explain_constraints.md`. The agent writes `explain.tex` and compiles `explain.pdf` in the request work directory. On success, malvin automatically runs `malvin revise` on the same `--out-path` `.tex` file.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | `explain.tex` and `explain.pdf` in the request work directory (override with `--out-path`) |
| Loop | Full gate-kpop loop (`KPopHardConstraints::EXPLAIN`) |
| Fast path | **None** — always runs the agent (like `code` / `delight`, unlike `tidy`) |
| Exit policy | Two consecutive `## KPOP_SOLVED` markers in per-iteration exp logs; workspace gates need not pass |
| Requires | No `kiss` or `.malvin/checks` preflight (document workflow, like `delight` / `revise`) |

## Intention

Explain code or concepts for a reader who will not read the source. Typical use: `malvin explain "How does the gate loop exit?"` or `malvin explain notes/topic.md`.

## Usage

```text
malvin explain [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Exactly **one shell argument**. Quote for internal spaces. Topic as literal text, or an existing `.md` file path (same rules as `inspire` / `code`).

When `REQUEST` names an existing `.md` file, the work directory is that file's parent; otherwise the work directory is `.` (cwd). With the default `--out-path`, outputs land in that work directory. A custom `--out-path` resolves against the current working directory instead.

## Options

### `--out-path <PATH>` (default: `explain.tex`)

LaTeX output path. malvin derives the PDF path by replacing the `.tex` extension with `.pdf`. With the default `explain.tex`, if either default output already exists in the request work directory, malvin allocates the first free sibling pair (`explain_1.tex` / `explain_1.pdf`, …). For any other `--out-path`, preflight refuses to run when either resolved path already exists.

With the default basename `explain.tex`, outputs stay in the request work directory (for example `notes/explain.tex` when `REQUEST` is `notes/topic.md`). Any other value resolves against the process cwd, like `malvin delight --out-path`.

### `--max-loops <N>` (default: 3)

Outer gate-loop budget (`max(N, 1) + 1` iterations). `0` is treated as `1`.

### `--max-hypotheses <N>` (default: 5)

Hypothesis budget per KPop session inside the gate loop.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Success criteria

All of the following must hold:

1. Two consecutive outer gate-loop iterations each declared `## KPOP_SOLVED` in their own exp log.
2. After the session, the resolved `--out-path` and its derived `.pdf` exist and each has size &gt; 0.
3. The decoupled `malvin revise` workflow runs automatically on the same `--out-path` `.tex` file (prose clarity pass via `revise_constraints.md`).

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation |
| `malvin delight` | Author a feature pitch |
| `malvin revise` | Prose clarity pass on an existing document (runs automatically after `explain`; also available standalone) |
| `malvin code` | Implement a plan via the gate loop |

## Examples

```text
malvin explain "How does malvin tidy exit the gate loop?"
malvin explain docs/notes.md
malvin explain "topic" --out-path docs/paper.tex
```
