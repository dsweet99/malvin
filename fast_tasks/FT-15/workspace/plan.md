# FT-15: Merge half-open intervals

Edit only files in this workspace.

## Task
Implement `merge_intervals(intervals: list[list[int]]) -> list[list[int]]` in `merge.py`.

Intervals are half-open `[L,R)`. Merge overlaps and abutting ranges (`R == next.L` merges). Hidden adversarial cases include empty list, single point `[2,2)`, unsorted input, nested intervals, and negative bounds. Output must be sorted by `L`.

## Rules
- Edit `merge.py` only.
- Exact list equality is graded on hidden cases.

## Done when
All hidden adversarial cases match exactly.

Treat abutting intervals as mergeable. Preserve half-open semantics so an empty `[2,2)` remains valid input.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
