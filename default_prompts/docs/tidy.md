# malvin tidy

Bring the workspace back to a **gate-clean** state using the KPop gate loop scoped by `tidy_constraints.md`.

## Summary

| | |
|---|---|
| Input | None (implicit goal: pass `.malvin/checks`) |
| Fast path | If gates pass on first check, **no agent** — prints `DONE` |
| Loop | Outer gate iterations when gates fail |
| Requires | `kiss` on PATH; Cursor agent CLI only when gates fail |

## Intention

Recover after failed `code` pre-checks, CI drift, or local gate failures—without a feature plan.

## Usage

```text
malvin tidy [OPTIONS]
```

No positional arguments. Work directory is always `.` (cwd).

## Options

### `--max-loops <N>` (default: 3)

Outer gate-loop budget (`max(N, 1) + 1` iterations). `0` is treated as `1`.

### `--max-hypotheses <N>` (default: 10)

Hypothesis budget per KPop session inside the gate loop.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Workflow

| Phase | Behavior |
|-------|----------|
| First gate check | Run all commands in `.malvin/checks`; append output to `quality_gates.log` |
| Gates pass | Emit startup summary, print `DONE`, exit (no ACP session) |
| Gates fail | Print failure details to stderr; enter gate loop (`KPopHardConstraints::TIDY`) |

**Gate loop (when agent runs):**

1. Each outer iteration runs one KPop session with `tidy_constraints.md` + `kpop_program.md`.
2. Agent logs to `_kpop/exp_log_<iteration>.md`.
3. Early exit on two consecutive `## KPOP_SOLVED` with passing gates.
4. Unlike `code`, tidy does **not** recheck gates after a fully exhausted loop (`recheck_gates_after_exhausted: false`).

## Comparison to `code`

| | `tidy` | `code` |
|---|--------|--------|
| User plan | None | `plan.md` from request |
| Skip agent if gates pass | Yes | No |
| Constraints file | `tidy_constraints.md` | `code_constraints.md` |
| Recheck gates after exhaustion | No | Yes |

## Artifacts

- `./.malvin/logs/<run>/plan.md` — rendered tidy KPop request (not a user-authored plan)
- `quality_gates.log`, `_kpop/exp_log_*.md`, `kpop.log`, `stdout.log` (when agent runs)

## Examples

```text
malvin tidy
malvin tidy --max-loops 5 --max-hypotheses 20
malvin tidy && malvin code plan.md
```
