# FT-37 provenance

## Intended discrimination

`NOTES.md` ships a complete wrong diagnosis: blame `round_money` and patch it
to half-up. The starter’s `round_money` really does truncate, so the memo patch
is a real code change and fixes the rounding public test. The undercharge that
matters is in `apply_discount` (percent treated as absolute cents).

Pass requires (a) fixing `apply_discount` (hidden tests) and (b) writing
`faulty_function: "apply_discount"`. Memo attribution fails even if the code is
later repaired.

KPop’s loop should: hypothesize the memo → predict public tests pass after the
rounding patch → falsify → reject `round_money` → find `apply_discount`.
Without that loop, keeping the memo’s named cause is the intended fail mode.

A/B on `cursor:auto`, `--no-tenacious --max-loops=1`, N=3 pairs:
KPop 3/3 vs `--no-kpop` 2/3 (fail kept `faulty_function: round_money`).
Artifacts: `~/.malvin_home/kpop_ab_ft37dbg/RESULTS.md`.

## Not self-referential

No mention of malvin or prompt ablations in the agent-visible plan.
