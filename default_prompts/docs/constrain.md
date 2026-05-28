# malvin constrain

Implement a plan **test-first**: regression test, then code, using the same **KPop gate loop** as `code` but scoped by `constrain_constraints.md`.

## Summary

| | |
|---|---|
| Input | Plan text or `.md` path → `plan.md` |
| Loop | Same outer gate loop as `malvin code` |
| Scope | `constrain_constraints.md` (test before implementation) |
| Requires | `kiss` on PATH; Cursor agent CLI |

## Intention

Enforce constraint-driven development: the agent must satisfy explicit test-first rules in the constraints prompt while working through the KPop gate loop until gates pass.

## Usage

```text
malvin constrain [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Same rules as `malvin code`: plan text or path to an existing `.md` file (no whitespace; case-sensitive `.md` suffix). Stored as `plan.md`.

## Options

Same as `malvin code`:

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | Outer gate-loop budget |
| `--max-hypotheses` | 10 | Hypothesis steps per gate session |
| `--tenacious` | off | `--max-acp-retries=9999` and `--max-loops=9999` |

## Global options

See `malvin --doc`.

## Workflow

Identical gate-loop machinery to `code` (`GateLoopBehavior::CODE`):

1. Render `kpop_program.md` with **`constrain_constraints.md`** as scope (regression test first, then implementation).
2. Run KPop sessions until two consecutive `## KPOP_SOLVED` markers and passing workspace gates, or the outer budget is exhausted.

## Artifacts

Same layout as `code`: `plan.md`, `_kpop/exp_log_*.md`, `kpop.log`, `quality_gates.log`.

## Related commands

| Command | When |
|---------|------|
| `malvin code` | Feature implementation without test-first constraint prompt |
| `malvin tidy` | Gate failures without a plan |

## Examples

```text
malvin constrain plan.md
malvin constrain "Add regression test for widget bug, then fix"
```
