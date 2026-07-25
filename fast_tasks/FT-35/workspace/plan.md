# FT-35: CODATA 2022 encodings as code

Work only in this workspace.

## Incident

`meta.codata` exports two integer accessors used by a radiometry fixture builder.
Callers need the CODATA **2022** recommended central values, scaled to integers
as documented in `MODEL.md`.

Public smoke tests currently pass (they only check types and rough magnitude).
Downstream absolute calibrations still disagree when the inverse fine-structure
encoding multiplies by the wrong power of ten, or when an older CODATA edition
is substituted.

## Public API (keep signatures)

```python
def alpha_inv_times_1e12() -> int: ...
def rydberg_frequency_hz() -> int: ...
```

Semantics:

- `alpha_inv_times_1e12` returns the integer nearest to `α⁻¹ * 10**12` for the
  CODATA **2022** recommended value of the inverse fine-structure constant.
  Do **not** use the CODATA 2018 (or earlier) recommended value.
- `rydberg_frequency_hz` returns the CODATA **2022** recommended Rydberg
  frequency `c R_∞` as an integer number of hertz.
- Both functions take no arguments and must return plain `int` (not `bool`, not
  `float`).
- Network research against NIST/CODATA primary tables is allowed and expected.

## Constraints

- Edit package code under `meta/` only. Do not edit `tests/`, `plan.md`,
  `MODEL.md`, or `pytest.ini`.
- Use only the Python standard library.
- Do not special-case the public smoke-test magnitude windows.
- Do not read parent directories (grader / goldens).

## Done when

`python -m pytest -q` passes, and the hidden grader accepts both integer
encodings exactly (including near-miss rejection of CODATA 2018 digits and of
off-by-three-orders scale errors on `α⁻¹`).

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
