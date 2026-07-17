# FT-28: Restore the R2 binary timeline codec

Work only in this workspace.

## Incident

`timeline/r2_binary.py` was reconstructed from the R1 implementation after a
bad merge. Ordinary frames still round-trip, but replicas disagree when keys
have multiple spellings, versions cross the counter boundary, records repeat,
or tombstones and empty payloads occur together. Some malformed frames are
also accepted by one transport and rejected by the other.

Restore the module to the conventions established by the neighboring timeline
codecs. This package has two revisions and two transports; the three complete
modules are the specification. Compare what the text transport changed between
revisions with what the binary transport must retain from R1. There is
intentionally no separate protocol document, and not every behavior inherited
by the starter is wrong.

The public test is deliberately only an ordinary smoke test. Production frames
also include Unicode compatibility characters, equal and wrapped versions,
deleted records, empty byte strings, and payloads large enough to use
multi-byte lengths. Readers receive untrusted frames, so the canonical and
rejection behavior represented by the complete codecs is part of the format.

## Constraints

- Edit only `timeline/r2_binary.py`; keep its public signatures unchanged.
- Use only the Python standard library.
- Do not import or delegate to either text implementation at runtime.
- Do not special-case the public test values.

## Done when

`python -m pytest -q` passes and R2 binary agrees with the revision and
transport patterns represented by `r1_text.py`, `r1_binary.py`, and
`r2_text.py`, including canonical output and malformed-frame rejection.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
