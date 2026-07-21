"""CODATA accessors — starter uses wrong edition / scale."""


def alpha_inv_times_1e12() -> int:
    # CODATA 2018 central value, correctly scaled — edition near-miss.
    return 137035999084


def rydberg_frequency_hz() -> int:
    # Placeholder magnitude-ish; not the 2022 integer encoding.
    return 3289841960355000
