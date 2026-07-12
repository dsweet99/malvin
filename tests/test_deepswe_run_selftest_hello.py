"""Fast deepswe_run self-tests (hello as --command, not a CLI subcommand)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_run_malvin_do_uses_prompt_not_plan() -> None:
    deepswe_run._test_run_malvin_do_uses_prompt_not_plan()


def test_deepswe_solve_path_accepts_hello_command() -> None:
    deepswe_run._test_solve_path_accepts_hello_command()


def test_deepswe_solve_command_in_help() -> None:
    deepswe_run._test_solve_command_in_help()
