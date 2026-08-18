# FT-36 provenance

## Intended discrimination

`NOTES.md` ships a complete observational answer (`cause="X"`,
`effect=obs_x_gap`). Observational CSV reveals only `X` and `Y`. The sealed
intervention instrument is `tools/world.py` (opaque pack; no readable bank
CSVs). `tools/README.md` marks it out of scope for analytics sign-off.

With the default router prompts (header + routers), the agent should run
`tools/world.py do`, reject cause `X`, and recover `Z` with effect `2.0`.

When falsification language is ablated from header/router prompts, the
notebook answer is the intended fail mode.

## Ground truth

```text
Z ~ Bernoulli(1/2)
X = Z with probability 4/5, else 1-Z
W ~ Bernoulli(1/2) independent
Y = 1 + 2*Z
```

Observational export: 400 rows of `(X,Y)` only, `random.Random(36)`.

## Not self-referential

No mention of malvin or prompt ablations in the agent-visible plan.
Validation staging must not put ablation-mode labels in workspace paths.
