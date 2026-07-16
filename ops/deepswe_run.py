#!/usr/bin/env python3
"""Click CLI for deepswe_run — implementation in ``src/python/deepswe_run.py``."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

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
def deepswe_run_solve(ctx: click.Context, **kwargs: Any) -> None:
    """Run malvin and Harbor grade (Modal by default; --local for Docker; --task for path-based)."""
    use_cursor = bool(kwargs.pop("use_cursor", False))
    test_harness = bool(kwargs.get("test_harness", False))
    malvin_command = kwargs.get("malvin_command", "route")
    if use_cursor and test_harness:
        raise click.ClickException("Use either --test or --cursor, not both")
    if use_cursor:
        if malvin_command not in ("route", "code"):
            raise click.ClickException(
                f"--cursor only applies to route/code, not --command {malvin_command}"
            )
        kwargs["malvin_command"] = "cursor"
    _lib.dispatch_solve(ctx, **kwargs)


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
