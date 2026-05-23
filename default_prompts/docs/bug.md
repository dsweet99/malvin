# malvin bug (experimental)

KPOP **bug hunter**: automatically investigate the codebase for a serious bug, then—only if KPOP records success—run regression-test and fix coder phases like a focused `code` run.

## Intention

End-to-end bug discovery and remediation without the user supplying a plan. The investigation brief is fixed internally (“find a serious bug”). Post-KPOP work uses the experiment log as the bug description.

## Usage

```text
malvin bug [OPTIONS]
```

No positional arguments.

## Options

### `--max-hypotheses <N>` (default: 10)

Same as `malvin kpop`: budget counts typed step lines (`## Step <n> — KPOP …` or `## Step <n> — MBC2 …` only). Alias: `--max-loops`.

### `--p-creative <P>` (default: 0.1)

Same as `kpop`: MBC2 interleave density. `≤ 0` or non-finite → pure KPOP blocks only.

### `--no-learn`

Skip **learn** after KPOP (and in the remediation orchestrator if wired).

### `--skip-pre-checks`

Skip workspace quality gates **before** the post-KPOP coder session (regression test + fix). KPOP itself always runs first. Default: gates must pass before remediation or malvin errors with tidy guidance.

### Global options

See `malvin.md`.

## Requirements

- `kiss` on PATH (CLI entry check)
- Cursor agent CLI

## Workflow (two major stages)

### Stage 1 — KPOP investigation

Same multiturn engine as `malvin kpop`, with a built-in request: find a serious bug.

| Prompt roles | Same as `kpop.md` (KPOP common, KPOP blocks, optional MBC2) |
|--------------|---------------------------------------------------------------|

**Gate to stage 2:** Experiment log must contain a line exactly `## KPOP_SOLVED`. Otherwise malvin stops with an error—no regression test or fix.

### Stage 2 — Bug remediation (coder orchestrator)

Creates a new run artifact set with `plan.md` describing post-KPOP remediation; KPOP log under `_kpop/` remains authoritative for the bug.

Pre-check: workspace gates (unless `--skip-pre-checks`).

Single coder session:

| Order | Prompt role (effect) |
|-------|----------------------|
| 1 | **Bug regression test** — Add a test that reproduces the confirmed bug. |
| 2 | **Bug fix** — Fix the bug so tests and gates pass. |
| 3 | **Pre-summary gap** — Workspace gates; on failure, one **tidy** retry (same as `code`). |
| 4 | **Summary** — User-facing recap. |

Plan check is **skipped** (`trust_the_plan` equivalent). Review loop from `code` is **not** used in remediation—only the two implement-style prompts plus gates and summary.

Optional **learn** follows the same rules as `code` when not `--no-learn`.

## Comparison to `kpop` + `code`

| | `bug` | Manual pipeline |
|---|-------|-----------------|
| KPOP request | Fixed | User-supplied |
| Requires `## KPOP_SOLVED` | Yes | Optional |
| Regression + fix | Automatic | User runs `code @plan` |
| Review loop | No | Yes (`code`) |

## Artifacts

- KPOP stage: `request.md`, `_kpop/exp_log_*.md`, `kpop.log`
- Remediation stage: new `_malvin/<run>/` with `plan.md`, implement logs, `quality_gates.log`

## Example

```text
malvin bug --max-hypotheses 15 --p-creative 0.1
malvin bug --skip-pre-checks --no-learn
```
