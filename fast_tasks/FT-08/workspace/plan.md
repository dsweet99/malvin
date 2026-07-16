# FT-08: Near-singular 2×2 solve

Edit only files in this workspace.

## Task
In `solve_system.py` implement `solve(a11,a12,a21,a22,b1,b2) -> (x,y)` for 2×2 linear systems using only Python floats.

Hidden tests include a near-singular case with true solution `(1.0, -1.0)` and a well-conditioned case.

Require max(|x−1|,|y+1|) ≤ 1e-9 on the near-singular case and ≤ 1e-12 on the well-conditioned case.

Cramer’s rule or explicit inverse OK; no numpy.

## Rules
- Edit `solve_system.py` only.
- Stdlib / pure Python floats only.

## Done when
Hidden numeric assertions pass.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
