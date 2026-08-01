# FT-30: Add public rolling_mean under kiss + pytest gates

Work only in this workspace.

## Incident

Callers need a public `rolling_mean(values: list[float], window: int) -> list[float]`
on the `statsutil` package. The workspace already has quality gates in
`.malvin/checks` (`kiss check`, then `pytest tests`). Both fail on the starter:
`kiss check` reports structural problems, and the public tests cannot import
`rolling_mean` from `statsutil`.

Add the missing feature so the public API exposes a correct simple moving
average. For input length n and window w with n >= w, return n - w + 1 means,
one per consecutive window. When n < w, return `[]`. Reject invalid windows
with an assertion (`window` must be an int >= 1). Use only the Python standard
library.

## Constraints

- Edit package code under `statsutil/` only; do not edit `tests/`,
  `.malvin/checks`, `.kissconfig`, or `pytest.ini`.
- Keep `kiss check` and `pytest tests` as the workspace quality gates.
- Do not special-case the public test values.
- Do not use the network or third-party packages.

## Done when

`kiss check` exits 0, `pytest tests` passes, and `rolling_mean` matches the
contract above (including empty inputs and window edge cases). Hidden tests
also run at grade time.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
