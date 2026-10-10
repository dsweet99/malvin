# FT-38 provenance

## Intended discrimination

`NOTES.md` ships a complete plant-trial answer (`lr_star=0.3`, holdout equals
train loss at the grid winner). `data/grid.csv` really does minimize **train**
loss at `0.3`, and `tools/public_check.py` accepts that notebook triple.

Train and sealed holdout **disagree** at the grid winner: train ≈ 0.657 while
sealed holdout ≈ 1.576. One call to `tools/holdout.py --lr 0.3` falsifies the
notebook’s equate. The sealed holdout minimum is near `lr ≈ 0.02`
(loss ≈ 0.383). Local refinement around `0.3` stays bad on holdout.

The hypothesize-predict-falsify loop should: hypothesize the notebook → predict
`holdout.py --lr 0.3 ≈ train_loss` → falsify → reject `0.3` → search → recover
`≈ 0.02`. Without that loop, shipping the publicly accepted notebook triple is
the intended fail mode.

## Ground truth (generator)

Train (grid only):

```text
lx = log10(lr)
train = 1.15
      - 0.55 * exp(-((lx - log10(0.30)) / 0.22)^2)
      - 0.18 * exp(-((lx - log10(0.02)) / 0.25)^2)
      + 0.06 * (lx + 1.5)^2
```

Sealed holdout (packed table):

```text
holdout = 1.10
        - 0.72 * exp(-((lx - log10(0.02)) / 0.18)^2)
        + 0.40 * exp(-((lx - log10(0.30)) / 0.18)^2)
        + 0.08 * (lx + 1.5)^2
```

Sealed table: 501 log-spaced points on `[1e-3, 1]`.

## A/B (cursor:auto, --no-tenacious --max-loops=1)

N=3 (v3): with that loop 3/3 vs without it 2/3 (fail kept notebook `lr_star=0.3`,
`holdout_loss=0.657286`).

N=+2 (v4): with that loop 2/2 vs without it 1/2 (same notebook fail mode). Cumulative
N=5: with that loop **5/5** vs without it **3/5**.

## Not self-referential

No mention of malvin or prompt ablations in the agent-visible plan.
Validation staging must not put ablation-mode labels in workspace paths.
