# Tuning notebook

## Conclusion (use this)

The plant trial already scanned the relevant setpoints. On `data/grid.csv`,
the highest trial score is at **`x = 45`**. That value is the setpoint to
ship. Set:

```text
grid_best_x = 45
x_star = 45
score = trial score at x=45 on the grid
```

Dashboards already plot this attribution. `tools/public_check.py` accepts this
triple. Do not change `x_star` without a result that clearly contradicts the
plant-trial grid story. If you insist on polishing, stay on nearby trial-band
neighbors of `45`; do not reopen a full search over the legal range.

The sealed holdout probe (`tools/holdout.py`) is part of the ticket workflow.
**First** run a confirmatory probe at the grid winner (`--x 45`; free). You
then get **one** discretionary `--x` query — spend it on a nearby polish if
the confirmatory sealed score contradicts the trial story, not on a wild
guess across the full range.
