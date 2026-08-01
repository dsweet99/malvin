#!/usr/bin/env python3
"""Click + Modal entry for deepswe_modal — implementation in ``src/python/deepswe_modal.py``."""

from __future__ import annotations

import sys
from pathlib import Path

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("deepswe_modal")
app = _lib.app


@click.command(
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.option(
    "--self-test",
    is_flag=True,
    help="Run local unit tests without Modal credentials.",
)
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--workspace",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--results-dir",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    show_default="~/.malvin_home/deepswe-results",
)
@click.option(
    "--command",
    "malvin_command",
    type=click.Choice(["route", "do", "hello", "init-checks"]),
    default="route",
    show_default=True,
)
@click.option(
    "--grade-only",
    is_flag=True,
    help="Skip agent; grade current workspace on Modal.",
)
@click.option(
    "--apply-solution",
    is_flag=True,
    help=(
        "Apply reference solution.patch before grade. "
        "Only valid with --grade-only (agent+apply-solution is refused)."
    ),
)
@click.option(
    "--reset",
    "reset_flag",
    is_flag=True,
    help=(
        "Hard reset workspace to base_commit before run. "
        "Always-on for agent solve (flag accepted as no-op)."
    ),
)
@click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def deepswe_modal_main(
    ctx: click.Context,
    self_test: bool,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    grade_only: bool,
    apply_solution: bool,
    reset_flag: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """DeepSWE evaluation on Modal (agent optional, Harbor grade in sandbox)."""
    _lib.dispatch_main(
        ctx,
        self_test,
        task_dir,
        workspace,
        results_dir,
        malvin_command,
        grade_only,
        apply_solution,
        reset_flag,
        malvin_args,
    )


main = deepswe_modal_main


@app.local_entrypoint(name="entrypoint")
def deepswe_modal_entrypoint(*arglist: str) -> None:
    """``modal run ops/deepswe_modal.py -- [OPTIONS] [-- MALVIN_ARGS]``."""
    main.main(
        args=list(arglist),
        prog_name="modal run ops/deepswe_modal.py",
        standalone_mode=True,
    )


entrypoint = deepswe_modal_entrypoint
run_unit_tests = _lib.run_unit_tests
run_modal_eval = _lib.run_modal_eval
cidr_probe_image = _lib.cidr_probe_image
stream_process_output = _lib.stream_process_output
CURSOR_API_HOSTS = _lib.CURSOR_API_HOSTS
agent_sandbox_network_kwargs = _lib.agent_sandbox_network_kwargs
sandbox_network_kwargs = _lib.sandbox_network_kwargs

__all__ = [
    "app",
    "main",
    "entrypoint",
    "deepswe_modal_main",
    "deepswe_modal_entrypoint",
    "run_unit_tests",
    "run_modal_eval",
    "cidr_probe_image",
    "stream_process_output",
    "CURSOR_API_HOSTS",
    "agent_sandbox_network_kwargs",
    "sandbox_network_kwargs",
]

if __name__ == "__main__":
    raise SystemExit(main())
