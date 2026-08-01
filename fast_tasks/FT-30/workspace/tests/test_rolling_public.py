"""Public API tests for rolling_mean."""
from statsutil import rolling_mean


def test_basic_window() -> None:
    assert rolling_mean([1.0, 2.0, 3.0, 4.0], 2) == [1.5, 2.5, 3.5]


def test_window_equals_length() -> None:
    assert rolling_mean([2.0, 4.0, 6.0], 3) == [4.0]


def test_window_too_large() -> None:
    assert rolling_mean([1.0, 2.0], 3) == []


def test_window_one() -> None:
    assert rolling_mean([9.0, 8.0], 1) == [9.0, 8.0]
