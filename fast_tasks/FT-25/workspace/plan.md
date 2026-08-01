# FT-25: Restore the R2 binary field codec

Work only in this workspace.

## Incident

The binary R2 codec was replaced by a partial implementation during a merge.
Restore `wire/r2_binary.py` so that it follows the field ordering,
normalization, framing, null handling, and rejection behavior established by
the neighboring codecs.

This repository supports two revisions and two transports. The three complete
modules are the specification: compare the revision change in the text
transport with the transport change in R1, then apply the same conventions to
R2 binary. There is intentionally no separate protocol document.

Public tests cover an ordinary round trip and currently pass. Production
reports only say that some R2 binary frames disagree with the other
implementations. Determine which inherited R1 behaviors are no longer valid in
R2; do not assume every apparent difference is a bug.

## Constraints

- Edit only `wire/r2_binary.py`; keep its public signatures unchanged.
- Use only the Python standard library.
- Do not import or delegate to the text codec at runtime.
- Do not special-case the public test data.

## Done when

`python -m pytest -q` passes, and the implementation handles the edge cases
implied by `r1_text.py`, `r1_binary.py`, and `r2_text.py`.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
