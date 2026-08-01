# FT-34: Microchannel gas mass-flow with rarefaction

Work only in this workspace.

## Incident

`flow.poiseuille.mass_flow_rate` estimates steady volumetric flow (m³/s) of a
dilute gas through a long circular microchannel under a small pressure drop.
Callers pass geometry, viscosity, pressure drop, and a Knudsen number
`Kn = λ / R` (mean free path over radius).

The public smoke tests currently pass. Vacuum-calibration runs at moderate
Knudsen number still disagree with the bench: the continuum Hagen–Poiseuille
branch is not enough by itself when `Kn > 0`.

## Public API (keep signature)

```python
def mass_flow_rate(
    radius_m: float,
    length_m: float,
    delta_p_pa: float,
    viscosity_pa_s: float,
    knudsen: float = 0.0,
) -> float: ...
```

Semantics:

- Continuum branch (`knudsen == 0`): classical Hagen–Poiseuille as in
  `MODEL.md`.
- Rarefied branch (`knudsen > 0`): isothermal **first-order Maxwell slip** for a
  circular tube with tangential-momentum accommodation **σ = 0.9** (not 1.0),
  Knudsen number `Kn = λ/R`. Use the standard round-tube leading correction that
  follows from Maxwell’s slip velocity with that σ. Do not use a
  hydraulic-diameter Knudsen definition, second-order slip, σ = 1 shortcuts, or a
  continuum answer when `Kn > 0`.
- Reject non-finite or non-positive geometry / viscosity / pressure-drop inputs
  with `ValueError`. `knudsen < 0` is also `ValueError`.

## Constraints

- Edit package code under `flow/` only. Do not edit `tests/`, `plan.md`,
  `MODEL.md`, or `pytest.ini`.
- Use only the Python standard library.
- Do not special-case the public smoke-test numbers.
- Do not read parent directories (grader / goldens).

## Done when

`python -m pytest -q` passes, and the hidden grader accepts continuum cases,
slip-regime cases (including a mid-check that the slip multiplier itself is
correct), and input validation.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
