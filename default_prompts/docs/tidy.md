# malvin tidy

Ensure the workspace passes **quality gates** (commands listed in `.malvin/checks`). If gates already pass, exit quickly without calling the agent. Otherwise run a tidy/review loop until LGTM and gates succeed, then optional learn and summary.

## Intention

Recover a dirty repo to a check-clean state—use after failed `code` pre-checks, CI drift, or local gate failures—without requiring a feature plan.

## Usage

```text
malvin tidy [OPTIONS]
```

No positional arguments. Session work directory is always `.` (cwd).

## Options

### `--max-loops <N>` (default: 3)

Maximum **outer iterations** of the tidy/review loop. Each iteration:

1. One coder turn (`tidy` on attempt 1, `tidy_concerns` on later attempts)
2. Reviewer fan-out + review write (same family as `code`)
3. On LGTM: run workspace gates; exit if gates pass

`0` is treated as `1`. If the last iteration gets LGTM but gates still fail, malvin may run a **bonus** recovery iteration (attempt `max_loops + 1`) with specialized gate-recovery logic.

### `--no-learn`

Skip the **learn** prompt after the loop (when elapsed time would allow it).

### Global options

See `malvin.md`. `--no-markdown` affects agent stdout when the tidy loop runs the agent (no effect on the fast path when gates already pass).

## Requirements

- `kiss` on PATH (CLI entry check)
- Cursor agent CLI (only when gates fail)

## Startup behavior

| Condition | Behavior |
|-----------|----------|
| Workspace gates **pass** immediately | **Skip agent** — emit startup, record timing, print `DONE`. No prompts. |
| Gates **fail** | Start agent session, enter tidy loop. |

Gate failure message on stderr points user at the failing commands before the agent runs.

## Prompt workflow (when agent runs)

### Main loop (per iteration, up to `--max-loops`)

| Step | Prompt role (effect) |
|------|----------------------|
| 1 | **Tidy** (iteration 1) or **Tidy concerns** (iteration 2+) — Fix issues blocking quality gates or review; concerns variant incorporates prior review feedback. |
| 2 | **Reviewers spawn** — Parallel reviewers (same mechanism as `code`). |
| 3 | **Review write** — Aggregate to LGTM or actionable review. |
| 4 | If not LGTM: continue loop (with recovery paths when `max_loops == 1`). |
| 5 | If LGTM: **workspace gates** — Must pass to finish; if fail, write gate failure artifact and continue or bonus-recover. |

### After loop succeeds

| Step | Prompt role (effect) | When |
|------|----------------------|------|
| 6 | **Learn** — Process reflection | Not `--no-learn`, elapsed ≥ 5 min |
| 7 | **Summary** — Header + summary body | Closing user message |

## Comparison to `code`

| | `tidy` | `code` |
|---|--------|--------|
| Input plan | None (implicit: fix checks) | User plan |
| Check plan | No | Yes (unless `--trust-the-plan`) |
| Implement | No (`tidy` prompts instead) | Yes |
| Pre-check at CLI | Self (gates decide skip/run) | Gates before session |

## Artifacts

- `./.malvin/logs/<run>/` with `plan.md` containing literal `"tidy"`
- `quality_gates.log`, review artifacts, phase logs
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
