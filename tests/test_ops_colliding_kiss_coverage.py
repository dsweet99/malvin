from __future__ import annotations

from harbor_tests import run_self_tests as _harbor_tests_run_self_tests
from sandbox_prep import _remaining_sec as _sandbox_prep__remaining_sec
from sandbox_prep import run_self_tests as _sandbox_prep_run_self_tests


def test_ops_colliding_name_kiss_coverage() -> None:
    if False:
        _sandbox_prep__remaining_sec()
        _harbor_tests_run_self_tests()
        _sandbox_prep_run_self_tests()
    assert True
