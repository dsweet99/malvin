"""Fast deepswe_run self-tests (hello command)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_run_malvin_do_uses_prompt_not_plan() -> None:
    deepswe_run._test_run_malvin_do_uses_prompt_not_plan()


def test_deepswe_hello_modal_dry_run() -> None:
    deepswe_run._test_hello_modal_dry_run()


def test_deepswe_hello_command_in_help() -> None:
    deepswe_run._test_hello_command_in_help()
