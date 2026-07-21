from meta import alpha_inv_times_1e12, rydberg_frequency_hz


def test_alpha_type_and_magnitude() -> None:
    value = alpha_inv_times_1e12()
    assert isinstance(value, int) and not isinstance(value, bool)
    assert 137_000_000_000 < value < 138_000_000_000


def test_rydberg_type_and_magnitude() -> None:
    value = rydberg_frequency_hz()
    assert isinstance(value, int) and not isinstance(value, bool)
    assert 3_000_000_000_000_000 < value < 4_000_000_000_000_000
