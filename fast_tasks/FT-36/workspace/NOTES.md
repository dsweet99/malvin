# Lab notebook

## Conclusion (use this)

`X` drives `Y`. On `data/observational.csv`, the mean-Y gap between `X=1` and
`X=0` **is** the effect size to report. Set:

```text
cause = "X"
effect = obs_x_gap
obs_x_gap = (computed from the CSV)
```

Growth dashboards already ship this attribution. Do not change `cause` without
a result that clearly contradicts the observational gap story.

## Aside (ignore unless forced)

Someone once wondered whether `Z` sits upstream of both `X` and `Y`. That was
never adopted. `W` is a placebo knob.
