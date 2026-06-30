"""Fast deepswe_run self-tests (solve / CLI routing)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_solve_dry_run() -> None:
    deepswe_run._test_solve_dry_run()


def test_deepswe_solve_modal_dry_run() -> None:
    deepswe_run._test_solve_modal_dry_run()


def test_deepswe_solve_modal_full_dry_run() -> None:
    deepswe_run._test_solve_modal_full_dry_run()


def test_deepswe_solve_resets_workspace_for_agent_runs() -> None:
    deepswe_run._test_solve_resets_workspace_for_agent_runs()


def test_deepswe_solve_local_dry_run_passes_reset() -> None:
    deepswe_run._test_solve_local_dry_run_passes_reset()


def test_deepswe_solve_command_in_help() -> None:
    deepswe_run._test_solve_command_in_help()


def test_deepswe_task_name_shorthand_routes_to_solve() -> None:
    deepswe_run._test_task_name_shorthand_routes_to_solve()


def test_deepswe_bare_invocation_shows_usage() -> None:
    deepswe_run._test_bare_invocation_shows_usage()


def test_deepswe_tasks_command() -> None:
    deepswe_run._test_tasks_command()


def test_deepswe_is_modal_spend_limit_error() -> None:
    deepswe_run._test_is_modal_spend_limit_error()


def test_deepswe_solve_modal_spend_limit_falls_back_to_local_dry_run() -> None:
    deepswe_run._test_solve_modal_spend_limit_falls_back_to_local_dry_run()


def test_deepswe_reset_workspace_removes_user_pycache() -> None:
    deepswe_run._test_reset_workspace_removes_user_pycache()


def test_deepswe_run_malvin_uses_plan_name_not_at_notation() -> None:
    deepswe_run._test_run_malvin_uses_plan_name_not_at_notation()


def test_deepswe_grade_only_apply_solution_fast_stub() -> None:
    deepswe_run._test_local_grade_only_apply_solution()


def test_deepswe_prepare_task_sandbox_dry_run() -> None:
    deepswe_run._test_prepare_task_sandbox_dry_run()
