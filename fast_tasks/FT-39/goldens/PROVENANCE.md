# FT-39 provenance

## Intended discrimination

`NOTES.md` and starter `predict.py` ship a complete mid-range linear law
(`predict(x) = 1.553973 + 0.148807 x`, `mid_sse = 0.000217`).
`tools/public_check.py` accepts that notebook setup.

Agent-visible `tools/holdout.py` is a **coarse** gate: it rejects clearly
unbounded large-`x` responses. The plan states that PASS is necessary but **not
sufficient**. A mid-range-only least-squares hyperbolic fit typically passes the
coarse gate yet fails host `HOLD_MSE_MAX = 0.002`. The sealed minimum is the
simple capacity-limited law

```text
predict(x) = 2.0 * x / (3.0 + x) + 1.0
```

Shipping the linear notebook law, or a mid-range-polished saturator with the wrong
 asymptotes, is the intended fail mode.

`--creative` selects `router_b_creative.md` (`malvin inspire` / MBC2: extremes,
negations, asymptotic scaling). Negating unbounded linearity and reading a clean
capacity limit (floor 1, capacity 3) is that move. There is no off-band CSV; the
agent must invent the form.

## Ground truth (generator)

```text
y = 2.0 * x / (3.0 + x) + 1.0
```

Mid-range: 7 points on `[2.7, 3.6]`, `random.gauss(0, 0.008)`, `random.Random(39)`.

## A/B (cursor:auto, --no-tenacious --max-loops=1, KPop on both arms)

### v2–v10 (superseded)

Packed tables, MSE oracles, `.so` disassembly, tight windows, and
`data/extremes.csv` all let agents recover the law under `kpop_common` without
`router_b_creative`. See `~/.malvin_home/creative_ab_ft39v*/RESULTS.md`.

### v11 (coarse holdout; aborted)

See `~/.malvin_home/creative_ab_ft39v11/RESULTS.md`.

### v12 (tight asymptote gate + 900s timeout)

See `~/.malvin_home/creative_ab_ft39v12/RESULTS.md` when present.

## Not self-referential

No mention of malvin, `--creative`, or prompt ablations in the agent-visible
plan. Validation staging must not put ablation-mode labels in workspace paths.
