"""Notebook response law for FT-40 (saturating MM on mid-range)."""


def predict(x: float) -> float:
    return 1.050000000000 + 2.450000000000 * x / (2.700000000000 + x)
