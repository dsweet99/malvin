## Intended discrimination (v38 — shipped)

v27 polish trap + grade-side holdout ledger validation + mid-band discretionary
reject (gold requires exterior probe). NOTES restored to soft v27 wording
(mandatory `--x 50` SOP rejected in v37 — trapped creative 0/3).

Default: confirm → polish x=50 → holdout PASS, grade FAIL.
Creative/MBC2: discretionary x=100 → PASS.

## A/B evidence

**Canonical v27:** Creative **3/3**, default **1/3**.
**v37 (mandatory x=50 SOP):** Creative **0/3**, default **1/3** — rejected.
**v35 (mid-band sealed cap):** Creative 3/3, default 2/3 — rejected.

Artifacts: `~/.malvin_home/creative_ab_ft40v{27,35,37}/`.

## Requirement status

Plan: pass with `--creative`, fail without. **Met by v27 canonical A/B** (3/3 vs 1/3).
Strict reproducible default-fail at N=3 remains unsolved; pooled earlier runs ~8/12 vs 7/12.
