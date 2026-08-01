# FT-34 golden provenance (grader-only; not in agent workspace)

Continuum:

```text
Q_cont = π R^4 ΔP / (8 μ L)
```

First-order Maxwell slip for a circular tube with accommodation σ and
`Kn = λ/R`:

```text
u_s = ((2 − σ)/σ) · λ · (∂u/∂n)
Q = Q_cont * (1 + 4 * ((2 − σ)/σ) * Kn)
```

This task fixes **σ = 0.9**, so

```text
Q = Q_cont * (1 + 4.888… · Kn) = Q_cont * (1 + 4*(2-0.9)/0.9 * Kn)
```

Designed failure modes:

- Ignoring `knudsen` (continuum only) — public pytest still green
- σ = 1 shortcut → `Q = Q_cont * (1 + 4 Kn)`
- `(1 + 6 Kn)` / `(1 + 8 Kn)` diameter or parallel-plate confusions
- Second-order / Beskok–Karniadakis forms
- Failing validation on non-positive inputs / negative Kn
