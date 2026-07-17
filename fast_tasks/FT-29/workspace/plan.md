# FT-29: Restore the R2 heap dispatcher

Work only in this workspace.

## Incident

`dispatch/r2_heap.py` was reconstructed from an older implementation after a
bad merge. Basic jobs still come out in priority order, but production workers
disagree about replacement jobs, cancelled jobs, wrapped generations, and
keys with more than one spelling. Some jobs can also return twice after their
priority changes.

Restore the module to the conventions established by the neighboring
dispatchers. This package has two revisions and two storage strategies; the
three complete modules are the specification. Compare what the linear
dispatcher changed between revisions with what the heap dispatcher must
retain from R1. There is intentionally no separate dispatch specification,
and not every behavior inherited by the starter is wrong.

The public test is deliberately an ordinary smoke test. Real workers reuse
keys, cancel queued jobs, retry old generations after counter wrap, submit
equal-generation corrections, and mix Unicode compatibility characters with
spacing and case variants. A job that has already been taken still matters
when deciding whether a delayed update is stale.

## Constraints

- Edit only `dispatch/r2_heap.py`; keep its public signatures unchanged.
- Use only the Python standard library.
- Do not import or delegate to `r2_linear.py` at runtime.
- Preserve the heap storage strategy; do not replace it with a linear scan.
- Do not special-case the public test values.

## Done when

`python -m pytest -q` passes and R2 heap behavior agrees with the revision and
storage patterns represented by `r1_linear.py`, `r1_heap.py`, and
`r2_linear.py`, including replacement, cancellation, ordering, and validation
behavior.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
