# malvin tidy

Ensure the workspace passes **quality gates** (commands listed in `.malvin/checks`). If gates already pass, exit quickly without calling the agent. Otherwise run up to `--max-loops` **KPop tidy** sessions, re-checking gates between each session.

## Intention

Recover a dirty repo to a check-clean state—use after failed `code` pre-checks, CI drift, or local gate failures—without requiring a feature plan.

## Usage

```text
malvin tidy [OPTIONS]
```

No positional arguments. Session work directory is always `.` (cwd).

## Options

### `--max-loops <N>` (default: 3)

Maximum **outer iterations**. Each iteration:

1. Run **all** workspace quality gates (every non-empty line in `.malvin/checks`), appending full output to `./.malvin/logs/<run>/quality_gates.log`.
2. If **all** gates pass: stop (no agent on this iteration).
3. If any gate fails: run one **KPop tidy** multiturn session (hypothesis steps per `kpop_program.md` / `--max-loops` alias semantics for inner KPop budget).

`0` is treated as `1`. If the last iteration still fails gates after its KPop session, malvin exits with a gate-failure error.

### `--no-learn`

Skip the **learn** prompt after a KPop session (when elapsed time would allow it).

### Global options

See `malvin.md`. `--no-markdown` affects agent stdout when the tidy loop runs the agent (no effect on the fast path when gates already pass).

## Requirements

- `kiss` on PATH (CLI entry check)
- Cursor agent CLI (only when gates fail)

## Startup behavior

| Condition | Behavior |
|-----------|----------|
| Workspace gates **pass** on the first check | **Skip agent** — emit startup (command line, plan text, logs path), print `DONE`. No ACP session. |
| Gates **fail** | Print gate failure details to stderr, start agent on first failing iteration, enter KPop tidy. |

Gate failure messages on stderr use the standard malvin gate-failure format before the agent runs.

## Prompt workflow (when agent runs)

Each outer iteration that still fails gates runs **one** KPop multiturn session:

| Step | Effect |
|------|--------|
| 1 | Emit startup (first agent iteration only) and KPop log line |
| 2 | **KPop tidy** — Agent works through falsifiable hypotheses per `kpop_program.md` and `tidy_constraints.md`, logging to `./.malvin/logs/<run>/_kpop/exp_log_<token>.md` |
| 3 | Optional **learn** after session when not `--no-learn` and elapsed ≥ 5 min |

There is no separate reviewer fan-out or `tidy` / `tidy_concerns` coder loop; remediation is entirely the KPop session.

## Comparison to `code`

| | `tidy` | `code` |
|---|--------|--------|
| Input plan | None (implicit: fix checks) | User plan |
| Check plan | No | Yes (unless `--trust-the-plan`) |
| Implement | KPop tidy sessions | Coder + review loop |
| Pre-check at CLI | Self (gates decide skip/run) | Gates before session |

## Artifacts

- `./.malvin/logs/<run>/` with `plan.md` containing the rendered tidy KPop request
- `quality_gates.log`, KPop experiment log under `_kpop/`, phase logs
- `stdout.log` when agent runs

## Examples

```text
malvin tidy
malvin tidy --max-loops 5
malvin tidy --no-learn
```

Typical recovery flow after failed code:

```text
malvin tidy && malvin code @plan.md
```
