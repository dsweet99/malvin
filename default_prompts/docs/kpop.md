# malvin kpop

**KPOP** (Popperian investigation): hypothesis-driven exploration with an experiment log under `_kpop/`. Distinct from gate-loop `code` / `tidy`—focused on understanding, not shipping a pre-written plan.

Prefer **bare** invocation when investigating: `malvin REQUEST` (same workflow, `kpop` subcommand is hidden but equivalent).

## Summary

| | |
|---|---|
| Input | One or more investigation briefs → `request.md` per run dir |
| Loop | `--max-loops` separate agent **runs** (each with its own experiment log) |
| Lookup | `malvin kpop <KPOP_ID>` prints a prior log (no agent) |

## Intention

Explore questions or codebase behavior scientifically: falsifiable hypotheses, tests, recorded outcomes. For MBC2 creative ideation without evaluation, use **`malvin inspire`**.

## Usage

```text
malvin [OPTIONS] <REQUEST>...           # bare kpop
malvin kpop [OPTIONS] <REQUEST>         # hidden alias (single request)
malvin kpop <KPOP_ID>                   # log lookup only
```

## Arguments

### `<REQUEST>...` (investigation brief, one or more for bare invocation)

Exactly **one shell argument**. Quote for internal spaces (e.g. `malvin "Why does the cache miss?"`). Text or an existing `.md` file path. Stored as `request.md` in the run dir (not `plan.md`).

Bare `malvin REQUEST...` runs each request in sequence as a separate kpop invocation. Each gets its own run directory under `~/.malvin_home/logs/<hash>/`, equivalent to separate shell invocations. The hidden `kpop` subcommand accepts a single request only.

### `<KPOP_ID>` (log lookup)

Short id: `M` plus five characters from `a-z` and `0-9` (example: `Ma3bx9`). Malvin searches `~/.malvin_home/logs/<hash>/` for `KPOP_LOG: <id>` and prints the experiment log. No agent session.

## Options

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | Separate kpop agent runs; stops early when mpc plan `DONE` and workspace gates pass |
| `--tenacious` | on | `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

Bare `malvin REQUEST` uses the same flags at the top level (see `malvin --doc`).

## Global options

See `malvin --doc`. Does **not** require `kiss` at CLI entry (unlike `code` / `tidy`).

## Multiturn architecture

Each agent session is a **single turn** assembled from three prompt layers (plus `header.md` on the first turn of a session):

| Piece | Role |
|-------|------|
| **kpop_common** | Popper method: hypothesize → predict → falsify; log outcomes to the experiment log |
| **mpc_block** | MPC workflow: write plan → KPop-review plan → revise plan → implement → write exactly `DONE` to mpc plan file |
| **User brief** | On disk at `user_request_path` (`request.md` in the run dir) |

Per-session artifacts under `_kpop/`:

| File | Role |
|------|------|
| `exp_log_<run>.md` | Experiment log — hypotheses, tests, and results (authoritative for investigation) |
| `mpc_plan.md` | Per-iteration MPC scratch plan; exactly `DONE` signals the agent finished this session |

Between outer `--max-loops` iterations, malvin clears `mpc_plan.md` (and may record a done marker for the prior iteration). Each outer iteration gets its own experiment log (`_g2`, `_g3`, … suffix when applicable).

## KPOP_LOG line

At startup malvin prints:

```text
[malvin] KPOP_LOG: Ma3bx9 _kpop/exp_log_<run_id>.md
```

The printed path is work-dir-relative via `format_prompt_path` and may differ from the literal home path string (`~/.malvin_home/logs/<hash>/<run>/…`).

Use `malvin kpop Ma3bx9` later to dump that log.

## Termination

Stops when any of:

- mpc plan file (`_kpop/mpc_plan.md`) contains exactly `DONE` and workspace quality gates pass
- `--max-loops` runs complete without early success
- Internal error

## Artifacts

- `request.md` — input brief
- `_kpop/mpc_plan.md` — per-iteration MPC plan scratch file (exactly `DONE` for early exit)
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
malvin "Why does cache invalidation fail under load?"
malvin req_1.md req_2.md req_3.md
malvin kpop questions/regression.md
malvin kpop Ma3bx9
```
