"""Fast deepswe_run self-tests (discovery / plan / patch surface)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_malvin_repo_root() -> None:
    deepswe_run._test_malvin_repo_root()


def test_deepswe_kiss_repo_root() -> None:
    deepswe_run._test_kiss_repo_root()


def test_deepswe_default_deepswe_tasks_root() -> None:
    deepswe_run._test_default_deepswe_tasks_root()


def test_deepswe_resolve_local_task_dir() -> None:
    deepswe_run._test_resolve_local_task_dir()


def test_deepswe_local_agent_image_tag() -> None:
    deepswe_run._test_local_agent_image_tag()


def test_deepswe_docker_local_eval_cmd() -> None:
    deepswe_run._test_docker_local_eval_cmd()


def test_deepswe_list_deepswe_tasks() -> None:
    deepswe_run._test_list_deepswe_tasks()


def test_deepswe_read_task_language() -> None:
    deepswe_run._test_read_task_language()


def test_deepswe_list_deepswe_tasks_with_language() -> None:
    deepswe_run._test_list_deepswe_tasks_with_language()


def test_deepswe_discover_deepswe_checks_minimal() -> None:
    deepswe_run._test_discover_deepswe_checks_minimal()


def test_deepswe_discover_deepswe_checks_python_repo() -> None:
    deepswe_run._test_discover_deepswe_checks_python_repo()


def test_deepswe_discover_deepswe_checks_precommit() -> None:
    deepswe_run._test_discover_deepswe_checks_precommit()


def test_deepswe_discover_deepswe_checks_existing_malvin_checks() -> None:
    deepswe_run._test_discover_deepswe_checks_existing_malvin_checks()


def test_deepswe_write_plan_and_checks_discovers() -> None:
    deepswe_run._test_write_plan_and_checks_discovers()
