"""Individual sandbox_prep unit tests (one pytest node per _test_*)."""
from __future__ import annotations

import tempfile
from pathlib import Path

import sandbox_prep

# Warm process-cached venvs during collection so call-phase durations stay under 1.5s.
_WARM = Path(tempfile.mkdtemp(prefix="malvin-venv-warm-"))
sandbox_prep._clone_cached_venv(_WARM / "empty")
sandbox_prep._clone_cached_venv(_WARM / "pytest80", ("pytest==8.0.0",))
sandbox_prep._clone_cached_venv(_WARM / "pytest834", ("pytest==8.3.4",))
sandbox_prep._clone_cached_venv(
    _WARM / "adaptix",
    (
        "typing-extensions==4.12.2",
        "typeguard==4.4.1",
        "pytest==8.3.4",
    ),
)


def test_sandbox_prep_parse_dockerfile_run_commands_multiline() -> None:
    sandbox_prep._test_parse_dockerfile_run_commands_multiline()

def test_sandbox_prep_workspace_sync_commands_bandit() -> None:
    sandbox_prep._test_workspace_sync_commands_bandit()

def test_sandbox_prep_workspace_sync_commands_fastapi() -> None:
    sandbox_prep._test_workspace_sync_commands_fastapi()

def test_sandbox_prep_bash_lc_pip_intents_ignore_shell_noise() -> None:
    sandbox_prep._test_bash_lc_pip_intents_ignore_shell_noise()

def test_sandbox_prep_requirement_inline_comments_stripped_for_pip() -> None:
    sandbox_prep._test_requirement_inline_comments_stripped_for_pip()

def test_sandbox_prep_pep508_extras_preserved_in_pip_install_spec() -> None:
    sandbox_prep._test_pep508_extras_preserved_in_pip_install_spec()

def test_sandbox_prep_requirements_editable_and_constraints_declared() -> None:
    sandbox_prep._test_requirements_editable_and_constraints_declared()

def test_sandbox_prep_poetry_extra_and_runtime_deps_declared() -> None:
    sandbox_prep._test_poetry_extra_and_runtime_deps_declared()

def test_sandbox_prep_fixture_imports_not_unmapped_for_workspace_project() -> None:
    sandbox_prep._test_fixture_imports_not_unmapped_for_workspace_project()

def test_sandbox_prep_editable_pip_segment_ignores_dirty_equals() -> None:
    sandbox_prep._test_editable_pip_segment_ignores_dirty_equals()

def test_sandbox_prep_infra_abort_dockerfile_sync_is_offline() -> None:
    sandbox_prep._test_infra_abort_dockerfile_sync_is_offline()

def test_sandbox_prep_dockerfile_image_build_commands_fastapi() -> None:
    sandbox_prep._test_dockerfile_image_build_commands_fastapi()

def test_sandbox_prep_hybrid_poetry_runtime_sync_skipped() -> None:
    sandbox_prep._test_hybrid_poetry_runtime_sync_skipped()

def test_sandbox_prep_hybrid_pnpm_runtime_sync_skipped() -> None:
    sandbox_prep._test_hybrid_pnpm_runtime_sync_skipped()

def test_sandbox_prep_tox_lint_check_commands() -> None:
    sandbox_prep._test_tox_lint_check_commands()

def test_sandbox_prep_just_and_tox_runner_install_commands() -> None:
    sandbox_prep._test_just_and_tox_runner_install_commands()

def test_sandbox_prep_workspace_lint_tool_install_command() -> None:
    sandbox_prep._test_workspace_lint_tool_install_command()

def test_sandbox_prep_precommit_install_hooks_command() -> None:
    sandbox_prep._test_precommit_install_hooks_command()

def test_sandbox_prep_precommit_pin_from_workspace_pyproject() -> None:
    sandbox_prep._test_precommit_pin_from_workspace_pyproject()

def test_sandbox_prep_uv_sync_dev_command() -> None:
    sandbox_prep._test_uv_sync_dev_command()

def test_sandbox_prep_uv_pip_build_system_command() -> None:
    sandbox_prep._test_uv_pip_build_system_command()

def test_sandbox_prep_uv_editable_install_command() -> None:
    sandbox_prep._test_uv_editable_install_command()

def test_sandbox_prep_default_pip_editable_seed_for_offline_sync() -> None:
    sandbox_prep._test_default_pip_editable_seed_for_offline_sync()

def test_sandbox_prep_editable_seed_reads_monorepo_build_backends() -> None:
    sandbox_prep._test_editable_seed_reads_monorepo_build_backends()

def test_sandbox_prep_editable_target_project_deps_enter_declared() -> None:
    sandbox_prep._test_editable_target_project_deps_enter_declared()

def test_sandbox_prep_uv_offline_smoke_commands() -> None:
    sandbox_prep._test_uv_offline_smoke_commands()

def test_sandbox_prep_setuptools_extra_requirement_files_not_extra_keys() -> None:
    sandbox_prep._test_setuptools_extra_requirement_files_not_extra_keys()

def test_sandbox_prep_workspace_declared_repin_command() -> None:
    sandbox_prep._test_workspace_declared_repin_command()

def test_sandbox_prep_workspace_image_warm_commands() -> None:
    sandbox_prep._test_workspace_image_warm_commands()

def test_sandbox_prep_registry_image_cache_bust_commands() -> None:
    sandbox_prep._test_registry_image_cache_bust_commands()

def test_sandbox_prep_registry_image_cache_bust_pydantic_v1_legitimate() -> None:
    sandbox_prep._test_registry_image_cache_bust_pydantic_v1_legitimate()

def test_sandbox_prep_declared_deps_skip_marker_gated_backports() -> None:
    sandbox_prep._test_declared_deps_skip_marker_gated_backports()

def test_sandbox_prep_mandatory_probe_no_crash_on_dotted_import_name() -> None:
    sandbox_prep._test_mandatory_probe_no_crash_on_dotted_import_name()

def test_sandbox_prep_run_post_prep_probes_structured_error() -> None:
    sandbox_prep._test_run_post_prep_probes_structured_error()

def test_sandbox_prep_run_post_prep_probes_multi_violation_errors() -> None:
    sandbox_prep._test_run_post_prep_probes_multi_violation_errors()

def test_sandbox_prep_run_post_prep_probes_mixed_import_and_violation_errors() -> None:
    sandbox_prep._test_run_post_prep_probes_mixed_import_and_violation_errors()

def test_sandbox_prep_mandatory_probe_prefers_metadata_over_stale_module_version() -> None:
    sandbox_prep._test_mandatory_probe_prefers_metadata_over_stale_module_version()

def test_sandbox_prep_mandatory_probe_runtime_metadata_wins_over_stale_version() -> None:
    sandbox_prep._test_mandatory_probe_runtime_metadata_wins_over_stale_version()

def test_sandbox_prep_mandatory_probe_fails_on_invalid_version_string() -> None:
    sandbox_prep._test_mandatory_probe_fails_on_invalid_version_string()

def test_sandbox_prep_mandatory_probe_accepts_single_char_version_ops() -> None:
    sandbox_prep._test_mandatory_probe_accepts_single_char_version_ops()

def test_sandbox_prep_mandatory_probe_strips_pep508_extras_before_specifier() -> None:
    sandbox_prep._test_mandatory_probe_strips_pep508_extras_before_specifier()

def test_sandbox_prep_precommit_warm_soft_fails_install_hooks() -> None:
    sandbox_prep._test_precommit_warm_soft_fails_install_hooks()

def test_sandbox_prep_pythonpath_dockerfile_skips_synthetic_editable() -> None:
    sandbox_prep._test_pythonpath_dockerfile_skips_synthetic_editable()

def test_sandbox_prep_effective_spec_prefers_pyproject_constraint_over_lockfile() -> None:
    sandbox_prep._test_effective_spec_prefers_pyproject_constraint_over_lockfile()

def test_sandbox_prep_effective_spec_exact_pyproject_beats_lockfile() -> None:
    sandbox_prep._test_effective_spec_exact_pyproject_beats_lockfile()

def test_sandbox_prep_mandatory_probe_fails_when_version_unknown() -> None:
    sandbox_prep._test_mandatory_probe_fails_when_version_unknown()

def test_sandbox_prep_httpx_drift_probe_script_write_roundtrip() -> None:
    sandbox_prep._test_httpx_drift_probe_script_write_roundtrip()

def test_sandbox_prep_probe_import_name_phonenumberslite() -> None:
    sandbox_prep._test_probe_import_name_phonenumberslite()

def test_sandbox_prep_mandatory_probe_uses_metadata_before_import() -> None:
    sandbox_prep._test_mandatory_probe_uses_metadata_before_import()

def test_sandbox_prep_registry_image_cache_bust_reconciles_twice_after_httpx_fix() -> None:
    sandbox_prep._test_registry_image_cache_bust_reconciles_twice_after_httpx_fix()

def test_sandbox_prep_mandatory_probe_script_commands_builder_safe() -> None:
    sandbox_prep._test_mandatory_probe_script_commands_builder_safe()

def test_sandbox_prep_mandatory_probe_script_write_roundtrip() -> None:
    sandbox_prep._test_mandatory_probe_script_write_roundtrip()

def test_sandbox_prep_registry_image_cache_bust_adaptix_pydantic_pin() -> None:
    sandbox_prep._test_registry_image_cache_bust_adaptix_pydantic_pin()

def test_sandbox_prep_pydantic_pins_for_cache_bust_reads_requirements() -> None:
    sandbox_prep._test_pydantic_pins_for_cache_bust_reads_requirements()

def test_sandbox_prep_collect_pip_install_intents_bash_lc() -> None:
    sandbox_prep._test_collect_pip_install_intents_bash_lc()

def test_sandbox_prep_dockerfile_bulk_pip_commands_fastapi() -> None:
    sandbox_prep._test_dockerfile_bulk_pip_commands_fastapi()

def test_sandbox_prep_workspace_sync_commands_fastapi_task_dockerfile() -> None:
    sandbox_prep._test_workspace_sync_commands_fastapi_task_dockerfile()

def test_sandbox_prep_should_replay_skips_apt_and_git() -> None:
    sandbox_prep._test_should_replay_skips_apt_and_git()

def test_sandbox_prep_discover_verifier_spec_public_vs_grade() -> None:
    sandbox_prep._test_discover_verifier_spec_public_vs_grade()

def test_sandbox_prep_verifier_venv_materialize_public_no_patch_only_names() -> None:
    sandbox_prep._test_verifier_venv_materialize_public_no_patch_only_names()

def test_sandbox_prep_verifier_grade_closure_commands_include_mapped() -> None:
    sandbox_prep._test_verifier_grade_closure_commands_include_mapped()

def test_sandbox_prep_probe_verifier_env_plugin_conflict_reports_verifier_prep() -> None:
    sandbox_prep._test_probe_verifier_env_plugin_conflict_reports_verifier_prep()

def test_sandbox_prep_prepare_verifier_grade_materialize_when_missing() -> None:
    sandbox_prep._test_prepare_verifier_grade_materialize_when_missing()

def test_sandbox_prep_probe_verifier_env_unmapped_imports_fail_closed() -> None:
    sandbox_prep._test_probe_verifier_env_unmapped_imports_fail_closed()

def test_sandbox_prep_prepare_task_sandbox_does_not_call_probe_verifier() -> None:
    sandbox_prep._test_prepare_task_sandbox_does_not_call_probe_verifier()

def test_sandbox_prep_probe_verifier_env_missing_collect_path_does_not_abort() -> None:
    sandbox_prep._test_probe_verifier_env_missing_collect_path_does_not_abort()

def test_sandbox_prep_probe_plugin_conflict_failed_collect_aborts() -> None:
    sandbox_prep._test_probe_plugin_conflict_failed_collect_aborts()

def test_sandbox_prep_modified_hunk_context_imports_in_verifier_spec() -> None:
    sandbox_prep._test_modified_hunk_context_imports_in_verifier_spec()

def test_sandbox_prep_adaptix_prepatch_materialize_catches_importerror() -> None:
    sandbox_prep._test_adaptix_prepatch_materialize_catches_importerror()

def test_sandbox_prep_verifier_pip_honors_spec_venv_path() -> None:
    sandbox_prep._test_verifier_pip_honors_spec_venv_path()

def test_sandbox_prep_prepare_verifier_grade_materialize_creates_real_venv() -> None:
    sandbox_prep._test_prepare_verifier_grade_materialize_creates_real_venv()

def test_sandbox_prep_discover_grade_closure_records_declared_harbor_imports() -> None:
    sandbox_prep._test_discover_grade_closure_records_declared_harbor_imports()

def test_sandbox_prep_editable_project_satisfies_harbor_import() -> None:
    sandbox_prep._test_editable_project_satisfies_harbor_import()

def test_sandbox_prep_probe_editable_roots_prefers_harbor_import_case() -> None:
    sandbox_prep._test_probe_editable_roots_prefers_harbor_import_case()

def test_sandbox_prep_non_pytest_test_sh_skips_collect_probe() -> None:
    sandbox_prep._test_non_pytest_test_sh_skips_collect_probe()

def test_sandbox_prep_unpinned_dockerfile_package_declared() -> None:
    sandbox_prep._test_unpinned_dockerfile_package_declared()

def test_sandbox_prep_cargo_and_go_mod_skipped_in_offline_sync() -> None:
    sandbox_prep._test_cargo_and_go_mod_skipped_in_offline_sync()

def test_sandbox_prep_collect_import_error_editable_feature_gap() -> None:
    sandbox_prep._test_collect_import_error_editable_feature_gap()

def test_sandbox_prep_bare_pyproject_deps_become_unpinned() -> None:
    sandbox_prep._test_bare_pyproject_deps_become_unpinned()

def test_sandbox_prep_adaptix_conflict_fixture_yields_plugin_policy_or_verifier_prep() -> None:
    sandbox_prep._test_adaptix_conflict_fixture_yields_plugin_policy_or_verifier_prep()

def test_sandbox_prep_adaptix_import_error_never_soft_succeeds_on_system_python() -> None:
    sandbox_prep._test_adaptix_import_error_never_soft_succeeds_on_system_python()

def test_sandbox_prep_plugin_policy_as_env_allowlist_wiring() -> None:
    sandbox_prep._test_plugin_policy_as_env_allowlist_wiring()

def test_sandbox_prep_plugin_disable_policy_lets_collect_boot() -> None:
    sandbox_prep._test_plugin_disable_policy_lets_collect_boot()

def test_sandbox_prep_verifier_prep_result_as_dict_excludes_secrets() -> None:
    sandbox_prep._test_verifier_prep_result_as_dict_excludes_secrets()

def test_sandbox_prep_leakage_public_view_excludes_patch_only_imports() -> None:
    sandbox_prep._test_leakage_public_view_excludes_patch_only_imports()

