"""Fast deepswe_run self-tests for ``solve --test`` harness smoke flag."""

from __future__ import annotations

import deepswe_run


def test_deepswe_solve_test_flag_modal_dry_run() -> None:
    deepswe_run._test_solve_test_flag_modal_dry_run()


def test_deepswe_solve_test_flag_rejects_skip_grade() -> None:
    deepswe_run._test_solve_test_flag_rejects_skip_grade()


def test_deepswe_solve_test_flag_in_help() -> None:
    deepswe_run._test_solve_test_flag_in_help()


def test_deepswe_init_checks_cmd_uses_fail_fast_source() -> None:
    deepswe_run._test_init_checks_cmd_uses_fail_fast_source()


def test_deepswe_write_deepswe_agent_checks_preseeds_file() -> None:
    deepswe_run._test_write_deepswe_agent_checks_preseeds_file()


def test_deepswe_write_deepswe_agent_checks_ecosystem_fallback() -> None:
    deepswe_run._test_write_deepswe_agent_checks_ecosystem_fallback()


def test_deepswe_evaluation_smoke_allows_reward_zero() -> None:
    deepswe_run._test_evaluation_smoke_allows_reward_zero()


def test_deepswe_run_malvin_init_checks_preseeds_then_shells() -> None:
    deepswe_run._test_run_malvin_init_checks_preseeds_then_shells()


def test_deepswe_agent_phase_needs_cursor_credentials() -> None:
    deepswe_run._test_agent_phase_needs_cursor_credentials()


def test_deepswe_discover_precommit_id_only_meta() -> None:
    deepswe_run._test_discover_deepswe_checks_precommit_id_only_meta()


def test_deepswe_write_agent_checks_bandit_like_id_only() -> None:
    deepswe_run._test_write_deepswe_agent_checks_bandit_like_id_only()
