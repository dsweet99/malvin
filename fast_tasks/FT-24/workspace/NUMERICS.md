# Numerical contract

`roots(a, b, c)` solves `a*x*x + b*x + c = 0`.

Preconditions:

- `a`, `b`, and `c` are finite Python floats and `a != 0.0`.
- The polynomial, treating each binary float input as its exact value, has two
  real roots counting multiplicity.
- Each exact root is finite when rounded to a Python float.

Return `(small, large)` in numeric ascending order. Each result must be within
8 ULPs of the correctly rounded root. For a repeated root, return it twice.

Intermediate overflow, underflow, and cancellation do not relax the contract.
The implementation may use any Python standard-library arithmetic internally;
only the function interface and returned values are floats.

This function is also a hot path. A batch of 6,000 well-conditioned calls whose
coefficients are small integral floats and whose discriminants are perfect
squares must complete within 40 ms on the grading host. Robustness must
therefore be adaptive rather than imposing heavyweight arithmetic on every
call.
