# FT-03: csvcut column select

Edit only files in this workspace.

## Task
Implement `bin/csvcut` (Python stdlib, executable) such that:

```text
./bin/csvcut -f b,a data/input.csv
```

prints CSV to stdout with header `b,a` and rows preserving that column order.

Missing columns must exit with code 2 and stderr exactly `missing: <name>` for the first missing name (no extra text).

## Rules
- Use `data/input.csv` and this plan / `task.md` only.
- Write the script at `bin/csvcut`.
- No network / no third-party installs.

## Done when
Column selection and missing-column errors behave as specified.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
