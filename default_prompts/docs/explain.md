# malvin explain

Produce a short, reader-friendly **LaTeX explanation** via an outer **Review → Plan → Work** loop. Review and Plan are one-shot in-process KPop sessions (separate experiment logs). Work is a coder turn. Success is chat `LGTM` from Review, with non-empty `.tex` and `.pdf`.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | `explain.tex` and `explain.pdf` in the request work directory (override with `--out-path`) |
| Loop | Outer `effective_max_loops(--max-loops)` iterations of Review → (LGTM stop \| Plan → Work) |
| Review / Plan | In-process KPop (exactly one session each); soft constraints from `explain_constraints.md` via `kpop_program_creative.md` for Review |
| Work | Coder session with `explain_work.md` |
| Exit policy | Review chat is exactly `LGTM`, then non-empty tex/pdf; workspace gates need not pass |
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

LaTeX output path. malvin derives the PDF path by replacing the `.tex` extension with `.pdf`. With the default `explain.tex`, if either default output already exists in the request work directory, malvin allocates the first free sibling pair (`explain_1.tex` / `explain_1.pdf`, …). For any other `--out-path`, preflight refuses to run when either resolved path already exists.

With the default basename `explain.tex`, outputs stay in the request work directory (for example `notes/explain.tex` when `REQUEST` is `notes/topic.md`). Any other value resolves against the process cwd, like `malvin delight --out-path`.

### `--max-loops <N>` (default: 3)

Outer review/plan/work iteration budget (`max(N, 1)`). This is **not** the KPop engine `N+1` gate budget. `0` is treated as `1`. Exhausting the budget without Review `LGTM` is failure.

### `--max-hypotheses <N>` (default: 10)

Hypothesis budget for **each** Review and Plan KPop session (fresh experiment log per phase, so budgets do not share). Precedence: CLI flag &gt; config `review.max_hypotheses` &gt; built-in **10**. Does **not** use `agent.max_hypotheses`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999` (outer review/plan/work budget).

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Loop contract

Each outer iteration:

1. **Review** (in-process KPop, once): judge lack-of-satisfaction of `explain_constraints.md`. Chat is exactly `LGTM` or a failure-focused gap list. Missing/empty products ⇒ never `LGTM`. Empty chat ⇒ fail.
2. If Review chat is `LGTM` → validate non-empty tex/pdf → success (skip Plan and Work).
3. **Plan** (in-process KPop, once): consume the review; chat is the work plan. Empty chat ⇒ fail.
4. **Work** (coder): execute `explain_work.md` with `{{ review }}` and `{{ plan }}`.

Review and Plan each use a distinct `exp_log` path every outer iteration.

## Success criteria

All of the following must hold:

1. Review returned exactly `LGTM` within the `--max-loops` outer budget.
2. The resolved `--out-path` and its derived `.pdf` exist and each has size &gt; 0.

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation |
| `malvin delight` | Author a feature pitch |

## Examples

```text
malvin explain "How does malvin tidy exit the gate loop?"
malvin explain docs/notes.md
malvin explain "topic" --out-path docs/paper.tex
```
