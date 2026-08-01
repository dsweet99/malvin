"""Fast deepswe_run self-tests (patch surface / config)."""

from __future__ import annotations

import deepswe_run


def test_deepswe_scan_pytest_monkeypatch_hooks() -> None:
    deepswe_run._test_scan_pytest_monkeypatch_hooks()


def test_deepswe_scan_class_level_attributes() -> None:
    deepswe_run._test_scan_class_level_attributes()


def test_deepswe_patch_surface_targets_prefers_config_style_classes() -> None:
    deepswe_run._test_patch_surface_targets_prefers_config_style_classes()


def test_deepswe_render_patch_surface_probe_roundtrip() -> None:
    deepswe_run._test_render_patch_surface_probe_roundtrip()


def test_deepswe_write_plan_and_checks_includes_patch_surface_probe() -> None:
    deepswe_run._test_write_plan_and_checks_includes_patch_surface_probe()


def test_deepswe_malvin_mem_limit_gb_from_task_memory() -> None:
    deepswe_run._test_malvin_mem_limit_gb_from_task_memory()


def test_deepswe_ensure_deepswe_malvin_config_seeds_home_config() -> None:
    deepswe_run._test_ensure_deepswe_malvin_config_seeds_home_config()


def test_deepswe_ensure_deepswe_malvin_config_skips_default_memory() -> None:
    deepswe_run._test_ensure_deepswe_malvin_config_skips_default_memory()


def test_deepswe_ephemeral_cache_find_expr() -> None:
    deepswe_run._test_ephemeral_cache_find_expr()


def test_deepswe_purge_root_owned_ephemeral_caches_docker_cmd() -> None:
    deepswe_run._test_purge_root_owned_ephemeral_caches_docker_cmd()


def test_deepswe_git_clean_would_remove_and_docker_purge_targets() -> None:
    deepswe_run._test_git_clean_would_remove_and_docker_purge_targets()


def test_deepswe_parse_task_dir_does_not_require_test_sh() -> None:
    deepswe_run._test_parse_task_dir_does_not_require_test_sh()


def test_deepswe_validate_verifier_paths_fails_without_test_sh() -> None:
    deepswe_run._test_validate_verifier_paths_fails_without_test_sh()
