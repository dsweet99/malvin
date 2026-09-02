"""Oracle response law for FT-39 (saturating response in x)."""


def predict(x: float) -> float:
    return 2.0 * x / (3.0 + x) + 1.0
