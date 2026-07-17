# FT-24: Recover both quadratic roots

Work only in this workspace.

## Incident

`quadratic.roots()` receives finite Python floats and returns the two real roots
in ascending order. Telemetry shows plausible coefficients producing zero,
infinity, or two indistinguishable roots. A proposed arbitrary-precision fix
then made this hot utility too slow. The ordinary public tests pass.

Investigate `NUMERICS.md`, the implementation, and the tests. Repair
`quadratic.py` so it meets the documented float-level contract. Hidden tests
include badly scaled coefficients, a root many orders of magnitude smaller
than the other, and a positive discriminant much smaller than either term used
to form it. The grader also measures a batch of ordinary small-integral
polynomials described in `NUMERICS.md`.

## Constraints

- Edit only `quadratic.py`; keep the public signature unchanged.
- Use only the Python standard library.
- Inputs satisfy the preconditions in `NUMERICS.md`.
- Do not special-case listed coefficients.

## Done when

`python -m pytest -q` passes and both returned floats agree with roots of the
exact values of the input floats to the documented tolerance.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
