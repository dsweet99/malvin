# malvin tidy

Bring the workspace back to a **gate-clean** state using the KPop gate loop scoped by `tidy_constraints.md`.

## Summary

| | |
|---|---|
| Input | None (implicit goal: pass `.malvin/checks`) |
| Fast path | If gates pass on first check, **no agent** — prints `DONE` |
| Loop | Outer gate iterations when gates fail |
| Requires | Cursor agent CLI only when gates fail |

## Intention

Recover after CI drift or local gate failures—without a feature plan.

## Usage

```text
malvin tidy [OPTIONS]
```

No positional arguments. Work directory is always `.` (cwd).

## Options

### `--max-loops <N>` (default: 3)

Outer gate-loop budget (`max(N, 1) + 1` iterations). `0` is treated as `1`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. Tidy always runs workspace gates; the global `--gates` option does not change tidy behavior.

## Workflow

| Phase | Behavior |
|-------|----------|
| First gate check | Run all commands in `.malvin/checks`; append output to `quality_gates.log` |
| Gates pass | Emit startup summary, print `DONE`, exit (no ACP session) |
| Gates fail | Print failure details to stderr; enter gate loop (`KPopHardConstraints::TIDY`) |

**Gate loop (when agent runs):**

1. Each outer iteration renders `tidy_constraints.md` through `kpop_program.md` into `request.md`, then runs one KPop session (`header.md` + `kpop_common.md`).
2. Agent logs hypotheses and test results to `_kpop/exp_log_<iteration>.md`.
3. Early exit when workspace gates pass.
4. Tidy does **not** recheck gates after a fully exhausted loop (`recheck_gates_after_exhausted: false`).

## Prompt roles

| Artifact | Role |
|----------|------|
| `tidy_constraints.md` | Implicit goal: pass workspace quality gates |
| `kpop_program.md` | Rendered into `request.md` — scope + quality gates |
| `kpop_common.md` | Popper method; log to experiment log |
| `header.md` | Prepended on each session |

## Artifacts

- `~/.malvin_home/logs/<hash>/<run>/request.md` — rendered tidy KPop request (not a user-authored plan)
- `quality_gates.log`, `_kpop/exp_log_*.md`, `kpop.log`, `stdout.log` (when agent runs)

## Examples

```text
malvin tidy
malvin tidy --max-loops 5
```
