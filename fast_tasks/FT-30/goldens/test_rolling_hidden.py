"""Hidden behavioral tests for FT-30 rolling_mean (grader-only)."""
from statsutil import rolling_mean
import pytest


def test_empty_values() -> None:
    assert rolling_mean([], 1) == []


def test_negatives_and_floats() -> None:
    assert rolling_mean([-1.0, 1.0, -1.0, 1.0], 2) == [0.0, 0.0, 0.0]


def test_full_drain_length() -> None:
    values = [float(i) for i in range(10)]
    got = rolling_mean(values, 4)
    assert len(got) == 7
    assert got[0] == pytest.approx(1.5)
    assert got[-1] == pytest.approx(7.5)


def test_window_must_be_positive() -> None:
    with pytest.raises(AssertionError):
        rolling_mean([1.0, 2.0], 0)
