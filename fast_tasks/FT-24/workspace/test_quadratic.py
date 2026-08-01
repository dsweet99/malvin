from quadratic import roots


def test_two_integer_roots() -> None:
    assert roots(1.0, -3.0, 2.0) == (1.0, 2.0)


def test_repeated_root() -> None:
    assert roots(1.0, -2.0, 1.0) == (1.0, 1.0)


def test_symmetric_roots() -> None:
    assert roots(1.0, 0.0, -4.0) == (-2.0, 2.0)
