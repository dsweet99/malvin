# malvin kpop

**KPOP** (Popperian scientific investigator): a multiturn hypothesis-driven workflow. The agent maintains an experiment log under `_kpop/` in the run directory, emitting structured `## Step` hypotheses until success, budget exhaustion, or `## KPOP_SOLVED`.

## Intention

Scientifically explore a question or codebase behavior: formulate falsifiable hypotheses, test them, record outcomes, and optionally interleave **MBC2** “creative” turns that stress-test assumptions. Distinct from `code`—focused on investigation, not shipping a pre-written plan.

## Usage

```
malvin kpop [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Investigation brief as text or `@<path>`. Stored as `request.md` in the run dir (not `plan.md`).

## Options

### `--max-hypotheses <N>` (default: 10)

Stop after this many typed step lines exist in the experiment log: `## Step <n> — KPOP …` or `## Step <n> — MBC2 …` (em dash, en dash, or hyphen before the kind). Both kinds count toward the same budget. Alias: `--max-loops`.

### `--p-creative <P>` (default: 0.1)

Controls KPOP block sizing and MBC2 interleaving:

- Higher `P` → smaller mean KPOP blocks, more frequent MBC2 turns.
- Non-finite or ≤ 0 → **pure KPOP** (no MBC2 prompts; only `kpop_block` multiturn).

### `--no-learn`

Skip the **learn** prompt at the end of the multiturn session (if elapsed time meets the learn threshold).

### Global options

See `malvin.md`. `--no-markdown` styles agent stdout. `--no-force` disables agent `--force`.

## Requirements

- Cursor agent CLI
- Does **not** require kiss at CLI entry (unlike `code`)

## Multiturn architecture

KPOP alternates two **phases** driven by state machine + experiment log on disk:

### KPOP block phase

| Prompt role (effect) |
|----------------------|
| **KPOP block** — Agent adds up to *N* new `## Step` hypotheses in one turn (*N* sampled from a Poisson distribution around mean derived from `--p-creative`). Catches up if the block under-filled, with a cap on catch-up attempts. |

When the block completes, control passes to MBC2 (if creative mode enabled) or starts a new block.

### MBC2 phase (when `--p-creative` enables creative mode)

| Prompt role (effect) |
|----------------------|
| **MBC2 pure** — Creative / adversarial turn; up to two sends per MBC2 phase. Logged separately from KPOP steps. |

After MBC2 updates the log, a new KPOP block begins (credit from oversized blocks can carry forward).

### Shared preamble (first turn)

| Prompt role (effect) |
|----------------------|
| **KPOP common** — Shared rules, quality-gates markdown for the workspace, and request text. Coding rules prepended once. |

Templates `kpop_block.md` / `mbc2_pure.md` (and `mbc2.md` when loaded) are selected by the progression engine, not as a fixed linear list.

## Termination

Stops when any of:

- Experiment log contains a line exactly `## KPOP_SOLVED` (agent-declared success)
- Typed step line count ≥ `--max-hypotheses` (same `## Step <n> — KPOP|MBC2` format as above; malformed `## Step` lines do not count)
- Internal error (e.g. block catch-up exhausted)

## Post-run (optional)

| Prompt role (effect) | When |
|----------------------|------|
| **Learn** — Session reflection | End of multiturn, if not `--no-learn` and elapsed ≥ 5 min |

## Artifacts

- `_malvin/<run>/request.md` — input brief
- `_malvin/<run>/_kpop/exp_log_<run>.md` — experiment log (authoritative)
- `kpop.log` — multiturn transcript
- `quality_gates.log` when gates are embedded in prompts

## Examples

```
malvin kpop "Why does cache invalidation fail under load?"
malvin kpop @questions/regression.md --max-hypotheses 20
malvin kpop --p-creative 0 --no-learn @brief.md
```

Pure KPOP (`--p-creative 0`) disables MBC2 entirely.
