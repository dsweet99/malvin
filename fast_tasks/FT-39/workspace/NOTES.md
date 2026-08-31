# Calibration notebook

## Conclusion (use this)

All useful setpoints sit in the mid-range band already exported to
`data/midrange.csv`. On that band a straight line is enough. Keep the starter
`predict.py` law:

```text
predict(x) = 1.553973 + 0.148807 * x
mid_sse = 0.000217
```

Residuals look unstructured at bench precision. Ship that linear law. Local
coefficient polish around these values is fine if you insist on tidying; do not
replace the linear-in-`x` form with a different functional form unless something
clearly contradicts the mid-range story. `tools/public_check.py` accepts this
setup.

## Aside (ignore unless forced)

Someone once wondered whether the curve bends once `x` leaves the bench band.
That was never adopted. The product only runs near the calibrated mid-range.
