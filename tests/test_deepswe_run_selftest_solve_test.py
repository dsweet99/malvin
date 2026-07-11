"""Fast deepswe_run self-tests for ``solve --test`` harness smoke flag."""

from __future__ import annotations

import deepswe_run


def test_deepswe_solve_test_flag_modal_dry_run() -> None:
    deepswe_run._test_solve_test_flag_modal_dry_run()


def test_deepswe_solve_test_flag_rejects_skip_grade() -> None:
    deepswe_run._test_solve_test_flag_rejects_skip_grade()


def test_deepswe_solve_test_flag_in_help() -> None:
    deepswe_run._test_solve_test_flag_in_help()
