# malvin kpop

**KPOP** (Popperian scientific investigator): a multiturn hypothesis-driven workflow. The agent maintains an experiment log under `_kpop/` in the run directory, emitting structured `## Step` hypotheses until success, budget exhaustion, or `## KPOP_SOLVED`.

## Intention

Scientifically explore a question or codebase behavior: formulate falsifiable hypotheses, test them, and record outcomes. Distinct from `code`—focused on investigation, not shipping a pre-written plan. For MBC2 creative interleave turns, use **`malvin invent`**.

## Usage

```text
malvin kpop [OPTIONS] <REQUEST>
malvin kpop <KPOP_ID>
```

## Arguments

### `<REQUEST>` (investigation brief)

Investigation brief as text or `@<path>`. Stored as `request.md` in the run dir (not `plan.md`).

### `<KPOP_ID>` (log lookup)

Short id `M` plus five characters from `a-z` and `0-9` (example: `Ma3bx9`). Malvin searches `{cwd}/.malvin/logs/**` for a tagged `KPOP_LOG: <id> …` line and prints the experiment log to stdout. No agent session.

## Options

### `--max-hypotheses <N>` (default: 10)

Stop after this many typed step lines exist in the experiment log: `## Step <n> — KPOP …` (em dash, en dash, or hyphen before the kind). Alias: `--max-loops`.

### `--no-learn`

Skip the **learn** prompt at the end of the multiturn session (if elapsed time meets the learn threshold).

### Global options

See `malvin.md`. `--no-markdown` styles agent stdout. `--no-force` disables agent `--force`.

## Requirements

- Cursor agent CLI
- Does **not** require kiss at CLI entry (unlike `code`)

## Multiturn architecture

Each turn uses the **KPOP block** prompt: the agent adds hypotheses as `## Step` lines in the experiment log.

| Prompt role (effect) |
|----------------------|
| **KPOP common** — Shared rules, quality-gates markdown for the workspace, and request text. Coding rules prepended once. |
| **KPOP block** — Agent adds new `## Step` hypotheses in one turn. |

## KPOP_LOG

At the start of a normal run, malvin prints:

```text
[malvin] KPOP_LOG: Ma3bx9 ./.malvin/logs/<run_id>/_kpop/exp_log_<run_id>.md
```

Use `malvin kpop Ma3bx9` later to dump that log.

## Termination

Stops when any of:

- Experiment log contains a line exactly `## KPOP_SOLVED` (agent-declared success)
- Typed step line count ≥ `--max-hypotheses`
- Internal error

## Post-run (optional)

| Prompt role (effect) | When |
|----------------------|------|
| **Learn** — Session reflection | End of multiturn, if not `--no-learn` and elapsed ≥ 5 min |

## Artifacts

- `./.malvin/logs/<run>/request.md` — input brief
- `./.malvin/logs/<run>/_kpop/exp_log_<run>.md` — experiment log (authoritative)
- `kpop.log` — multiturn transcript
- `quality_gates.log` when gates are embedded in prompts

## Examples

```text
malvin kpop "Why does cache invalidation fail under load?"
malvin kpop @questions/regression.md --max-hypotheses 20
malvin kpop Ma3bx9
```
