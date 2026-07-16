# FT-13: Write tests that catch a planted whitespace bug

Edit only files in this workspace.

## Task
`impl/normalize.py` is frozen (do not edit it). Implement `tests/test_normalize.py` so that:

1. All tests PASS against a correct reference implementation of `normalize_ws` (wired only at grade time).
2. At least one test FAILS against the planted buggy `impl/normalize.py`.

API: `normalize_ws(s: str) -> str` collapses internal whitespace and strips ends.

Planted bug: tabs are not treated as whitespace.

Use pytest.

## Rules
- Do not edit `impl/normalize.py`.
- No network.

## Done when
Your tests distinguish buggy vs correct behavior under the dual-run grader.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
