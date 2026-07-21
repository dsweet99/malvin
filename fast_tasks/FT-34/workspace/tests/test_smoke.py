import math

import pytest

from flow import mass_flow_rate


def test_continuum_matches_hagen_poiseuille() -> None:
    r, L, dp, mu = 1e-5, 1e-2, 1000.0, 1.8e-5
    got = mass_flow_rate(r, L, dp, mu, knudsen=0.0)
    want = math.pi * r**4 * dp / (8.0 * mu * L)
    assert abs(got - want) / want < 1e-12


def test_rejects_non_positive_radius() -> None:
    with pytest.raises(ValueError):
        mass_flow_rate(0.0, 1.0, 1.0, 1.0, 0.0)
