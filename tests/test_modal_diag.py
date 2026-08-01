"""Pytest entrypoints for src/python/modal_diag_selftest (kiss via ops/.kissconfig)."""

from __future__ import annotations

from modal_diag_selftest import run_modal_diag_selftests


def test_modal_diag_selftests() -> None:
    run_modal_diag_selftests()
