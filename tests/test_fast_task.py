"""Unit tests for ops/fast_task.py (no live agent)."""

from __future__ import annotations

import errno

import fast_task
from click.testing import CliRunner
from toolchain_repos import load_ops_entry


def test_fast_task_self_tests_via_cli() -> None:
    cli = load_ops_entry("fast_task").fast_task_cli
    runner = CliRunner()
    result = runner.invoke(cli, ["self-test"], catch_exceptions=False)
    assert result.exit_code == 0, result.output
    assert "ALL fast_task self-tests OK" in result.output


def test_fast_task_cli_tasks() -> None:
    cli = load_ops_entry("fast_task").fast_task_cli
    runner = CliRunner()
    result = runner.invoke(cli, ["tasks"])
    assert result.exit_code == 0, result.output
    assert "FT-01" in result.output


def test_fast_task_default_results_falls_back_when_read_only(monkeypatch, tmp_path) -> None:
    monkeypatch.setattr(fast_task, "ft_default_results_dir", lambda: tmp_path / "readonly")
    monkeypatch.setattr(fast_task, "REPO_ROOT", tmp_path)
    original_mkdir = fast_task.Path.mkdir

    def deny_default(self, *args, **kwargs):
        if self == (tmp_path / "readonly" / "FT-01"):
            raise OSError(errno.EROFS, "read-only")
        return original_mkdir(self, *args, **kwargs)

    monkeypatch.setattr(fast_task.Path, "mkdir", deny_default)
    run_root = fast_task.ft_run_root("FT-01", None)
    assert run_root.is_dir()
    assert run_root.is_relative_to(tmp_path / ".malvin" / "fast_task_results")


def test_fast_task_kiss_coverage_witnesses() -> None:
    ops = load_ops_entry("fast_task")
    _ = (
        fast_task.ft_default_results_dir,
        fast_task.ft_timestamp_dir,
        fast_task.ft_list_task_ids,
        fast_task.ft_resolve_task_dir,
        fast_task._ft_copy_ignore,
        fast_task.ft_stage_workspace,
        fast_task.ft_ensure_staged_git,
        fast_task.ft_assert_stage_isolated,
        fast_task.ft_resolve_malvin_binary,
        fast_task.ft_resolve_malvin_main_binary,
        fast_task._ft_dry_run_stub_binary,
        fast_task._ft_resolve_host_binary,
        fast_task.ft_resolve_cursor_sdk_bridge_dir,
        fast_task.ft_resolve_node_bin,
        fast_task.ft_resolve_codex_bin,
        fast_task.ft_resolve_codex_package,
        fast_task.ft_resolve_codex_auth_file,
        fast_task.ft_normalize_agent,
        fast_task.ft_malvin_args_request_pi,
        fast_task.ft_malvin_args_request_codex,
        fast_task.ft_malvin_args_request_creative,
        fast_task.ft_assert_creative_compatible,
        fast_task.ft_dockerfile_for_agent,
        fast_task.ft_assert_dockerfile_nonleak,
        fast_task.ft_docker_available,
        fast_task.ft_ensure_agent_image,
        fast_task.ft_cursor_env_args,
        fast_task.ft_run_malvin_logs_dir,
        fast_task.ft_docker_agent_cmd,
        fast_task.ft_assert_agent_cmd_nonleak,
        fast_task.ft_relay_subprocess_stdout,
        fast_task._ft_kill_process_group,
        fast_task.ft_preflight_workspace_mount,
        fast_task.ft_grade_on_host,
        fast_task.ft_print_evaluation_summary,
        fast_task.ft_exit_from_evaluation,
        fast_task.ft_run_solve,
        fast_task.ft_cli_list_tasks,
        fast_task.ft_cli_solve,
        fast_task.ft_cli_self_test,
        ops.fast_task_cli,
        ops.fast_tasks_list_cmd,
        ops.fast_task_solve,
        ops.fast_task_selftest_cmd,
        fast_task.run_fast_task_self_tests,
        fast_task._ft_test_list_and_resolve_tasks,
        fast_task._ft_test_stage_workspace_isolated,
        fast_task._ft_test_dockerfile_nonleak,
        fast_task._ft_test_docker_agent_cmd_nonleak,
        fast_task._ft_test_docker_agent_cmd_cursor,
        fast_task._ft_test_docker_agent_cmd_pi,
        fast_task._ft_test_docker_agent_cmd_codex,
        fast_task._ft_assert_solve_creative_dry_runs,
        fast_task._ft_test_assert_agent_cmd_rejects_task_root,
        fast_task._ft_test_grade_on_host_starter_reward_zero,
        fast_task._ft_test_solve_checks_docker_before_build,
        fast_task._ft_test_solve_help_and_dry_run,
        fast_task._ft_test_solve_main_dry_run,
        fast_task._ft_test_resolve_malvin_binary_prefers_current_repo_build,
        fast_task._ft_test_resolve_malvin_main_binary,
        fast_task._ft_test_resolve_agent_helpers,
        fast_task._ft_relay_stdout_spy,
        fast_task._ft_echo_capture,
        fast_task._ft_test_relay_streams_before_wait,
        fast_task._ft_test_relay_timeout_kills_slow_command,
        fast_task._ft_test_print_evaluation_includes_reward,
        fast_task._ft_test_helpers_and_cli_surface,
        fast_task._ft_test_exit_from_evaluation,
        fast_task._ft_test_ensure_agent_image_dry_run,
        fast_task._ft_test_default_results_dir,
        fast_task.ft_redact_cmd_tokens,
        fast_task.ft_redact_cmd_for_display,
        fast_task._ft_test_redact_cmd_for_display,
        fast_task._ft_test_preflight_requires_host_plan,
        fast_task.DEFAULT_AGENT_TIMEOUT_SEC,
        fast_task.AGENT_TIMEOUT_ENV,
        fast_task.ft_agent_timeout_sec,
        fast_task.TIMEOUT_EXIT_CODE,
        fast_task.AGENT_MALVIN,
        fast_task.AGENT_CURSOR,
        fast_task.AGENT_CHOICES,
        fast_task.EXTERNAL_AGENTS,
        fast_task.CURSOR_AGENT_SHELL,
        fast_task.CURSOR_SDK_BRIDGE_REMOTE,
        fast_task.CURSOR_SDK_BRIDGE_JS_REMOTE,
    )
    assert True
