# malvin kpop

**KPOP** (Popperian investigation): hypothesis-driven exploration with an experiment log under `_kpop/`. Distinct from gate-loop `code` / `tidy`—focused on understanding, not shipping a pre-written plan.

Prefer **bare** invocation when investigating: `malvin REQUEST` (same workflow, `kpop` subcommand is hidden but equivalent).

## Summary

| | |
|---|---|
| Input | One or more investigation briefs → `request.md` per run dir |
| Loop | `--max-loops` separate agent **runs** (each with its own experiment log) |
| Per run | Up to `--max-hypotheses` typed `## Step … — KPOP` lines |
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

Text or an existing `.md` file path. Stored as `request.md` in the run dir (not `plan.md`).

Bare `malvin REQUEST...` runs each request in sequence as a separate kpop invocation. Each gets its own run directory under `./.malvin/logs/`, equivalent to separate shell invocations. The hidden `kpop` subcommand accepts a single request only.

### `<KPOP_ID>` (log lookup)

Short id: `M` plus five characters from `a-z` and `0-9` (example: `Ma3bx9`). Malvin searches `{cwd}/.malvin/logs/**` for `KPOP_LOG: <id>` and prints the experiment log. No agent session.

## Options

| Flag | Default | Meaning |
|------|---------|---------|
| `--max-loops` | 1 | Separate kpop agent runs; stops early when a run’s log contains `## KPOP_SOLVED` |
| `--max-hypotheses` | 10 | `## Step … — KPOP` budget **per** agent run |
| `--tenacious` | on | `--max-acp-retries=9999` and `--max-loops=9999` |
| `--no-tenacious` | off | Restore normal loop/retry budgets |

Bare `malvin REQUEST` uses the same flags at the top level (see `malvin --doc`).

## Global options

See `malvin --doc`. Does **not** require `kiss` at CLI entry (unlike `code` / `tidy`).

## Multiturn architecture

Each agent run:

| Piece | Role |
|-------|------|
| **KPOP common** | Shared rules, workspace quality-gates markdown, request text |
| **KPOP block** | Agent adds new `## Step` hypotheses in one turn batch |
| Experiment log | `./.malvin/logs/<run>/_kpop/exp_log_<run>.md` (second run may use `_g2` suffix, etc.) |

## KPOP_LOG line

At startup malvin prints:

```text
[malvin] KPOP_LOG: Ma3bx9 ./.malvin/logs/<run_id>/_kpop/exp_log_<run_id>.md
```

Use `malvin kpop Ma3bx9` later to dump that log.

## Termination

Stops when any of:

- Experiment log contains a line exactly `## KPOP_SOLVED`
- Typed step count reaches `--max-hypotheses`
- `--max-loops` runs complete without early success
- Internal error

## Artifacts

- `request.md` — input brief
- `_kpop/exp_log_*.md` — experiment log (authoritative)
- `kpop.log` — multiturn transcript
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
malvin kpop questions/regression.md --max-hypotheses 20
malvin kpop Ma3bx9
```
