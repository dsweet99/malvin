# malvin revise

Revise an **existing document in place** via the KPop gate loop scoped by `revise_constraints.md`.

## Summary

| | |
|---|---|
| Input | Positional `DOC_PATH` — existing file to revise |
| Output | Same path, edited in place (no `--out-path`) |
| Loop | Full gate-kpop loop (`GateLoopBehavior::REVISE`) |
| Fast path | **None** — always runs the agent (like `code` / `delight`) |
| Exit policy | Two consecutive `## KPOP_SOLVED` markers in per-iteration exp logs; workspace gates need not pass |
| Requires | No `kiss` or `.malvin/checks` preflight (document workflow, like `explain` / `delight`) |

## Intention

Improve clarity and precision of an existing markdown or prose document — fixing mystifying synonymy, non-local references, hedgy language, and similar issues defined in `revise_constraints.md`.

## Usage

```text
malvin revise DOC_PATH [OPTIONS]
```

`DOC_PATH` must name an existing regular file.

## Options

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

1. Preflight passed (`DOC_PATH` existed as a regular file at start).
2. Two consecutive outer gate-loop iterations each declared `## KPOP_SOLVED` in their own exp log.
3. After the session, `DOC_PATH` is still a regular file with size &gt; 0.

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin plan` | Four-prompt refinement on an implementation plan |
| `malvin delight` | Author a new feature plan from scratch |
| `malvin explain` | Write a LaTeX explanation PDF |

## Examples

```text
malvin revise docs/guide.md
malvin revise README.md --max-loops 5
```
