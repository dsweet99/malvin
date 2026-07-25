# CODATA 2022 integer encodings

Recover digits from a NIST CODATA **2022** primary table (wallet card or the
CODATA 2022 constants paper). Do not use CODATA 2018.

## Inverse fine-structure constant

Let `α⁻¹` be the recommended central value. Return

```text
alpha_inv_times_1e12 = nearest integer to (α⁻¹ * 10**12)
```

as a Python `int`. The factor is exactly one trillion (`10**12`), not
`10**9`, `10**15`, or a digit-string concatenation.

## Rydberg frequency

Let `c R_∞` be the recommended Rydberg frequency in hertz. Return that value as
an integer number of hertz (`rydberg_frequency_hz`). Do not return the Rydberg
constant in inverse meters.
