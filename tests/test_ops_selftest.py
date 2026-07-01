"""Run ops module self-tests (Modal-free unit checks)."""

from __future__ import annotations

import modal_sandbox_lifecycle
from deepswe_modal import (
    run_agent_toolchain_unit_tests,
    run_allowlist_refresh_unit_tests,
    run_cidr_allowlist_unit_tests,
    run_harbor_modal_probe_unit_tests,
    run_harvest_sandbox_unit_tests,
    run_modal_lifecycle_eval_unit_tests,
)
from malvin_modal import run_unit_tests as run_malvin_modal_unit_tests
from sandbox_prep import run_self_tests as run_sandbox_prep_self_tests


def test_modal_sandbox_lifecycle_self_test() -> None:
    modal_sandbox_lifecycle._test_release_modal_sandbox()


def test_sandbox_prep_self_tests() -> None:
    run_sandbox_prep_self_tests()


def test_deepswe_modal_cidr_allowlist_unit_tests() -> None:
    run_cidr_allowlist_unit_tests()


def test_deepswe_modal_allowlist_refresh_unit_tests() -> None:
    run_allowlist_refresh_unit_tests()


def test_deepswe_modal_agent_toolchain_unit_tests() -> None:
    run_agent_toolchain_unit_tests()


def test_deepswe_modal_harvest_sandbox_unit_tests() -> None:
    run_harvest_sandbox_unit_tests()


def test_deepswe_modal_harbor_modal_probe_unit_tests() -> None:
    run_harbor_modal_probe_unit_tests()


def test_deepswe_modal_lifecycle_eval_unit_tests() -> None:
    run_modal_lifecycle_eval_unit_tests()


def test_malvin_modal_unit_tests() -> None:
    run_malvin_modal_unit_tests()
