#!/usr/bin/env python3
"""Click CLI for deepswe_run — implementation in ``src/python/deepswe_run.py``."""

from __future__ import annotations

from pathlib import Path

import click

from _ops_bootstrap import load_library

_lib = load_library("deepswe_run")


@click.group(cls=_lib.TaskAliasGroup)
def deepswe_run_cli() -> None:
    """Run malvin on a DeepSWE task and grade with Harbor ``tests/test.sh``."""


@deepswe_run_cli.command("tasks")
def deepswe_run_tasks_cmd() -> None:
    """List all available DeepSWE tasks."""
    _lib.cli_list_tasks()


@deepswe_run_cli.command("self-test")
def deepswe_run_self_test_cmd() -> None:
    """Run unit tests and exit (no task run)."""
    _lib.cli_self_test()


@deepswe_run_cli.command(
    "solve",
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.argument("task_name", required=False)
@_lib._task_kernel_options
@_lib._local_solve_options
@click.pass_context
def deepswe_run_solve(
    ctx: click.Context,
    *,
    task_name: str | None,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    runtime: str,
    skip_materialize: bool,
    grade_only: bool,
    smoke_grade: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    use_local_docker: bool,
    test_harness: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run malvin and Harbor grade (Modal by default; --local for Docker; --task for path-based)."""
    _lib.dispatch_solve(
        ctx,
        task_name=task_name,
        task_dir=task_dir,
        workspace=workspace,
        results_dir=results_dir,
        malvin_command=malvin_command,
        runtime=runtime,
        skip_materialize=skip_materialize,
        grade_only=grade_only,
        smoke_grade=smoke_grade,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
        docker_image=docker_image,
        dry_run=dry_run,
        use_local_docker=use_local_docker,
        test_harness=test_harness,
        malvin_args=malvin_args,
    )


cli = deepswe_run_cli
main = deepswe_run_cli
tasks_cmd = deepswe_run_tasks_cmd
self_test_cmd = deepswe_run_self_test_cmd
solve = deepswe_run_solve
run_self_tests = _lib.run_self_tests
materialize_workspace = _lib.materialize_workspace
parse_task_dir = _lib.parse_task_dir

__all__ = [
    "deepswe_run_cli",
    "deepswe_run_tasks_cmd",
    "deepswe_run_self_test_cmd",
    "deepswe_run_solve",
    "cli",
    "main",
    "tasks_cmd",
    "self_test_cmd",
    "solve",
    "run_self_tests",
    "materialize_workspace",
    "parse_task_dir",
]

if __name__ == "__main__":
    raise SystemExit(main())
