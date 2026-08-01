# FT-09: CSV → Pearson correlation

Edit only files in this workspace.

## Task
Given `data/pairs.csv` columns `x,y` (n=40), compute the Pearson sample correlation

```text
r = cov(x,y) / (sx * sy)
```

using sample covariance and sample stddevs with Bessel correction (divide by n−1), matching `numpy.corrcoef` on 1-D arrays.

Write `answer.json`:

```json
{"n":40, "pearson_r": <float>, "mean_x": <float>, "mean_y": <float>}
```

Tolerances: |pearson_r − GOLD| ≤ 1e-6; means abs err ≤ 1e-9. Stdlib only. Do not plot.

## Rules
- No network / no numpy required.
- Overwrite or create `answer.json` at the workspace root.

## Done when
Numeric fields match within tolerances.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
