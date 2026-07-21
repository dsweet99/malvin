"""Hagen–Poiseuille microchannel flow."""
from __future__ import annotations

import math


def mass_flow_rate(
    radius_m: float,
    length_m: float,
    delta_p_pa: float,
    viscosity_pa_s: float,
    knudsen: float = 0.0,
) -> float:
    """Return volumetric flow rate [m^3/s] for a circular microchannel.

    Starter bug: ignores knudsen (continuum only). Public smoke tests use Kn=0,
    so they pass while slip-regime callers see large relative error.
    """
    for name, value in (
        ("radius_m", radius_m),
        ("length_m", length_m),
        ("delta_p_pa", delta_p_pa),
        ("viscosity_pa_s", viscosity_pa_s),
    ):
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            raise ValueError(f"{name} must be a real number")
        if not math.isfinite(float(value)) or float(value) <= 0.0:
            raise ValueError(f"{name} must be finite and positive")
    if not isinstance(knudsen, (int, float)) or isinstance(knudsen, bool):
        raise ValueError("knudsen must be a real number")
    if not math.isfinite(float(knudsen)) or float(knudsen) < 0.0:
        raise ValueError("knudsen must be finite and >= 0")

    q_cont = math.pi * float(radius_m) ** 4 * float(delta_p_pa) / (
        8.0 * float(viscosity_pa_s) * float(length_m)
    )
    # BUG: slip multiplier omitted.
    return q_cont
