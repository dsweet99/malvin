#!/usr/bin/env python3
"""Click CLI for fast_task — implementation in ``src/python/fast_task.py``."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("fast_task")

@click.group()
def fast_task_cli() -> None:
    """Run malvin on a fast task in local Docker; grade on the host."""

@fast_task_cli.command("tasks")
def fast_tasks_list_cmd() -> None:
    """List available fast task ids."""
    _lib.ft_cli_list_tasks()

@fast_task_cli.command(
    "solve",
    context_settings={"ignore_unknown_options": True, "allow_extra_args": True},
)
@click.argument("task_id")
@click.option(
    "--results-dir",
    type=click.Path(path_type=Path),
    default=None,
    help="Host results root (default: ~/.malvin_home/fast_task_results)",
)
@click.option(
    "--docker-image",
    default=None,
    help=f"Agent image tag (default: {_lib.DEFAULT_IMAGE})",
)
@click.option(
    "--base-image",
    default=_lib.DEFAULT_BASE_IMAGE,
    show_default=True,
    help="Base image used when building the agent image",
)
@click.option("--dry-run", is_flag=True, help="Stage + print cmds; skip docker run")
@click.option("--skip-grade", is_flag=True, help="Skip host grading after the agent")
@click.option(
    "--agent",
    type=click.Choice(list(_lib.AGENT_CHOICES), case_sensitive=False),
    default=_lib.AGENT_MALVIN,
    show_default=True,
    help=(
        "Agent to run in the container: malvin (default), or cursor "
        "(cursor-agent --force -p < plan.md)"
    ),
)
@click.option(
    "--main",
    "use_main",
    is_flag=True,
    help=(
        "Mount host malvin-main as the container malvin binary "
        "(do not rebuild or modify the local binary; only with --agent=malvin)"
    ),
)
@click.option(
    "--model",
    default=None,
    help=(
        "Model id passed through to malvin (ignored unless --agent=malvin); "
        "pi: models use the linked pi_agent_rust crate; codex: models bind-mount "
        "the Codex npm package and Node.js"
    ),
)
@click.option(
    "--creative",
    is_flag=True,
    help=(
        "Pass --creative through to malvin (creative router_b); "
        "incompatible with --cursor and --agent=cursor"
    ),
)
def fast_task_solve(
    task_id: str,
    creative: bool,
    **options: Any,
) -> None:
    """Run malvin on TASK_ID in Docker; report host-graded reward."""
    ctx = click.get_current_context()
    malvin_args = tuple(ctx.args)
    model = options.pop("model")
    if model is not None:
        malvin_args = ("--model", model, *malvin_args)
    if creative:
        malvin_args = ("--creative", *malvin_args)
    _lib.ft_cli_solve(
        task_id,
        malvin_args=malvin_args,
        timeout_sec=None,
        **options,
    )

@fast_task_cli.command("self-test")
def fast_task_selftest_cmd() -> None:
    """Run fast unit self-tests (no live agent)."""
    _lib.ft_cli_self_test()

__all__ = [
    "fast_task_cli",
    "fast_tasks_list_cmd",
    "fast_task_solve",
    "fast_task_selftest_cmd",
]

if __name__ == "__main__":
    fast_task_cli()
