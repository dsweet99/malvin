"""Docker-backed deepswe_run self-tests (skipped when daemon unavailable)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_docker_purge_root_owned_ephemeral_caches() -> None:
    deepswe_run._test_purge_root_owned_ephemeral_caches_docker()


def test_deepswe_docker_local_grade_only_apply_solution() -> None:
    deepswe_run._test_local_grade_only_apply_solution()
