"""Quadratic root calculation."""

from __future__ import annotations

import math


def roots(a: float, b: float, c: float) -> tuple[float, float]:
    """Return the two real roots of ``a*x*x + b*x + c``."""
    discriminant = b * b - 4.0 * a * c
    radical = math.sqrt(discriminant)
    first = (-b - radical) / (2.0 * a)
    second = (-b + radical) / (2.0 * a)
    return (min(first, second), max(first, second))
