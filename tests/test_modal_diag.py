"""Pytest entrypoints for ops/modal_diag_selftest (keeps heavy tests under ops/.kissconfig)."""

from __future__ import annotations

from modal_diag_selftest import run_modal_diag_selftests


def test_modal_diag_selftests() -> None:
    run_modal_diag_selftests()
