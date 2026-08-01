# FT-27: Repair the R2 SQLite journal

Work only in this workspace.

## Incident

`journal/r2_sqlite.py` was reconstructed from an older branch after a bad
merge. Ordinary updates still appear in snapshots, but replicas disagree
after sequence wrap, repeated keys, deletes, failed batches, and calls made
inside a larger transaction. Replay lag also rises sharply when a batch
contains a long run of updates to the same few keys.

Restore the module to the conventions established by the neighboring journal
implementations. This package has two revisions and two backends; the three
complete modules are the specification. Some behavior in the starter is
inherited correctly, while some is not. Determine what R2 changed and what the
SQLite backend must preserve.

The public test is deliberately only a smoke test. `apply` also receives
one-shot iterables in production, and callers may already have an open
transaction when they call it. The replay worker uses batches with tens of
thousands of events; the staging pattern in the memory backend is intentional.

## Constraints

- Edit only `journal/r2_sqlite.py`; keep its public signatures unchanged.
- Use only the Python standard library.
- Do not import or delegate to the memory implementation at runtime.
- Do not special-case the public test values.

## Done when

`python -m pytest -q` passes and R2 SQLite agrees with the revision and backend
patterns represented by `r1_memory.py`, `r1_sqlite.py`, and `r2_memory.py`,
including their failure and transaction behavior.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
