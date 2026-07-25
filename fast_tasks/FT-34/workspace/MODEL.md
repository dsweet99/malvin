# Continuum Hagen–Poiseuille

For a circular channel of radius `R`, length `L`, pressure drop `ΔP`, and
dynamic viscosity `μ`, the classical volumetric flow rate is

```text
Q_cont = π * R^4 * ΔP / (8 * μ * L)
```

Rarefaction / slip corrections are intentionally not tabulated here; apply the
first-order Maxwell slip model required by `plan.md` when `Kn > 0`.
