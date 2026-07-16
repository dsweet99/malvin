# FT-20: unique_sorted must beat O(n²)

Edit only files in this workspace.

## Task
Implement `unique_sorted(xs: list[int]) -> list[int]` in `uniq.py` returning sorted unique ints.

Functional hidden tests include negatives and empties.

Timing: on `list(range(5000)) + list(range(5000))`, must finish in ≤ 20 ms median of 3 runs (grader). A correct O(n²) list-membership solution will fail the clock.

Stdlib only; edit `uniq.py` only.

## Rules
- No third-party packages.
- Do not defeat the timer by skipping work / hardcoding.

## Done when
Correctness and timing both pass.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
