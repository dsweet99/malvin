# malvin kpop

**KPOP** (Popperian investigation): hypothesis-driven exploration with an experiment log under `_kpop/`. Distinct from gate-loop `code` / `tidy`—focused on understanding, not shipping a pre-written plan.

## Summary

| | |
|---|---|
| Input | One or more investigation briefs → `request.md` per run dir |
| Loop | `--max-loops` separate agent **runs** (each with its own experiment log); stops early when a run's log contains `## KPOP_SOLVED` |
| Per run | Up to `--max-hypotheses` typed `## Step … — KPOP` lines |
| Lookup | `malvin kpop <KPOP_ID>` prints a prior log (no agent) |

## Intention

Explore questions or codebase behavior scientifically: falsifiable hypotheses, tests, recorded outcomes. For MBC2 creative ideation without evaluation, use **`malvin inspire`**.

## Usage

```text
malvin kpop [OPTIONS] <REQUEST>...
malvin kpop <KPOP_ID>                   # log lookup only
```

## Arguments

### `<REQUEST>...` (investigation brief, one or more)

Each shell argument is one request. Quote for internal spaces (e.g. `malvin kpop "Why does the cache miss?"`). Text or an existing `.md` file path. Stored as `request.md` in the run dir (not `plan.md`).

`malvin kpop REQUEST...` runs each request in sequence as a separate kpop invocation. Each gets its own run directory under `~/.malvin_home/logs/<hash>/`, equivalent to separate shell invocations.

### `<KPOP_ID>` (log lookup)

Short id: `M` plus five characters from `a-z` and `0-9` (example: `Ma3bx9`). Malvin searches `~/.malvin_home/logs/<hash>/` for `KPOP_LOG: <id>` and prints the experiment log. No agent session.

## Options

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | Separate kpop agent runs; stops early when a run's log contains `## KPOP_SOLVED` |
| `--max-hypotheses` | 5 | `## Step … — KPOP` budget **per** agent run |
| `--tenacious` | on | `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

## Global options

See `malvin --doc`. Does **not** require workspace quality gates at CLI entry (unlike `code` / `tidy`).

## Session architecture

Each agent session sends **one prompt**: `header.md` + `kpop_common.md` + `kpop_block.md` (Popper loop with a per-run hypothesis budget).

| Piece | Role |
|-------|------|
| **kpop_common** | Popper method: hypothesize → predict → falsify; log outcomes to the experiment log |
| **kpop_block** | Turn budget (`want` = `--max-hypotheses`); may append `## KPOP_SOLVED` |
| **User brief** | Inlined in `kpop_block.md` and on disk at `user_request_path` (`request.md`) |

Per-session artifacts under `_kpop/`:

| File | Role |
|------|------|
| `exp_log_<run>.md` | Experiment log — hypotheses, tests, and results (authoritative for investigation) |

Each outer `--max-loops` iteration gets its own experiment log (`_g2`, `_g3`, … suffix when applicable).

## KPOP_LOG line

At startup malvin prints:

```text
[malvin] KPOP_LOG: Ma3bx9 _kpop/exp_log_<run_id>.md
```

The printed path is work-dir-relative via `format_prompt_path` and may differ from the literal home path string (`~/.malvin_home/logs/<hash>/<run>/…`).

Use `malvin kpop Ma3bx9` later to dump that log.

## Termination

Stops when any of:

- A run's experiment log contains `## KPOP_SOLVED` (outer loop exits early)
- `--max-loops` runs complete
- Per-run `--max-hypotheses` budget is exhausted (no more `## Step … — KPOP` lines)
- Internal error

## Artifacts

- `request.md` — input brief
- `_kpop/exp_log_*.md` — experiment log (authoritative for hypotheses and test results)
- `kpop.log` — session transcript
- `quality_gates.log` when gates are embedded in prompts

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | Creative MBC2 ideas, not hypothesis testing |
| `malvin code` | Implement a plan with gate loop + `code_constraints.md` |
| `malvin do` | Single-turn task without KPop logging |

## Examples

```text
malvin kpop "Why does cache invalidation fail under load?"
malvin kpop req_1.md req_2.md req_3.md
malvin kpop questions/regression.md
malvin kpop Ma3bx9
```
