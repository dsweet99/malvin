"""Run ops module self-tests (Modal-free unit checks)."""

from __future__ import annotations

import modal_sandbox_lifecycle
from malvin_modal import run_unit_tests as run_malvin_modal_unit_tests


def test_modal_sandbox_lifecycle_self_test() -> None:
    modal_sandbox_lifecycle._test_release_modal_sandbox()


def test_malvin_modal_unit_tests() -> None:
    run_malvin_modal_unit_tests()
