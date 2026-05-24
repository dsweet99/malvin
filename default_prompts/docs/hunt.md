# malvin hunt (experimental)

KPOP **bug hunter**: investigate the codebase for a serious bug, optionally record a durable **BUG_ID**, and optionally run regression-test and fix coder phases.

## Intention

Three modes: discover only, discover and fix in one invocation, or fix a bug found in an earlier discover run. Post-KPOP remediation uses the experiment log as the bug description. Discovery and remediation share one `./.malvin/logs/<run_id>/` tree (not separate run directories).

## Usage

```text
malvin hunt [OPTIONS]
malvin hunt --fix [OPTIONS]
malvin hunt <BUG_ID> [OPTIONS]
```

| Invocation | Behavior |
|------------|----------|
| `malvin hunt` | **Discover only:** KPOP investigation; on `## KPOP_SOLVED`, log `BUG_ID` + `BUG_LOG`, then `DONE`. No regression test or fix. |
| `malvin hunt --fix` | **Discover + fix:** same discovery, then regression test and fix (former default). |
| `malvin hunt <BUG_ID>` | **Fix by id:** skip KPOP; locate the originating run under `{cwd}/.malvin/logs/**` from log lines; run remediation. |

`<BUG_ID>` must match `M` plus five characters from `a-z` and `0-9` (example: `Ma3bx9`). `--fix` and `<BUG_ID>` cannot be combined.

## BUG_ID and BUG_LOG

After KPOP records success (`## KPOP_SOLVED` in the experiment log), discover modes emit:

```text
[malvin] BUG_ID: Ma3bx9
[malvin] BUG_LOG: Ma3bx9 ./.malvin/logs/<run_id>/_kpop/exp_log_<run_id>.md
```

These lines are written to stdout and the run’s `stdout.log`. Fix-by-id searches for `BUG_LOG: <id> ` first, then falls back to `BUG_ID: <id>` and the standard `_kpop/exp_log_*.md` path for that run.

## Options

### `--fix`

Discover and remediate in one invocation. Mutually exclusive with `<BUG_ID>`.

### `--max-hypotheses <N>` (default: 10)

Same as `malvin kpop`: budget counts `## Step <n> — KPOP …` headings in the experiment log. Alias: `--max-loops`.

### `--no-learn`

Skip **learn** in the remediation orchestrator when not `--no-learn`.

### Global options

See `malvin.md`.

## Requirements

- `kiss` on PATH (CLI entry check)
- Cursor agent CLI

## Workflow

### KPOP investigation (discover and `--fix`)

Same multiturn engine as `malvin kpop`, with the built-in request from `default_prompts/hunt_request.md` (embedded in the binary; overridable via a custom prompt root).

| Prompt roles | Same as `kpop.md` (KPOP common, KPOP blocks) |
|--------------|---------------------------------------------------------------|

**Success gate:** Experiment log must contain a line exactly `## KPOP_SOLVED`. Otherwise malvin stops with an error—no `BUG_ID`, no remediation.

### Bug remediation (`--fix` or `<BUG_ID>`)

Uses the same run directory as discovery. Writes or reuses `plan.md` describing post-KPOP remediation; the KPOP log under `_kpop/` remains authoritative.

Single coder session:

| Order | Prompt role (effect) |
|-------|----------------------|
| 1 | **Bug regression test** — Add a test that reproduces the confirmed bug. |
| 2 | **Bug fix** — Fix the bug so tests and gates pass. |
| 3 | **Summary** — User-facing recap. |

Plan check is **skipped** (`trust_the_plan` equivalent). Review loop from `code` is **not** used in remediation.

Optional **learn** follows the same rules as `code` when not `--no-learn`.

## Comparison to `kpop` + `code`

| | `hunt` | Manual pipeline |
|---|-------|-----------------|
| KPOP request | Fixed | User-supplied |
| Requires `## KPOP_SOLVED` | Yes (for BUG_ID / fix) | Optional |
| Regression + fix | `--fix` or `<BUG_ID>` | User runs `code @plan` |
| Review loop | No | Yes (`code`) |

## Artifacts

- Discover: `request.md`, `_kpop/exp_log_*.md`, `stdout.log` (`BUG_ID` / `BUG_LOG` lines)
- Remediation (same run dir): `plan.md`, implement logs, `quality_gates.log`

## Examples

```text
malvin hunt
malvin hunt --fix --max-hypotheses 15 
malvin hunt Ma1b2c --no-learn
```
