#!/usr/bin/env python3
"""Click CLI for Cursor SDK shutdown QA repros — implementation in ``src/python/qa.py``."""

from __future__ import annotations

import sys
from pathlib import Path

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("qa")

@click.group()
def qa_cli() -> None:
    """Regression checks for Cursor SDK process-management fixes."""

@qa_cli.command("list")
def qa_list_cmd() -> None:
    """List available shutdown QA scenarios."""
    _lib.list_scenarios()

@qa_cli.command("sigkill-stdin-hold-abandons-bridge")
def qa_sigkill_stdin_hold_abandons_bridge() -> None:
    """Local: stdin-hold + SIGKILL parent abandons cursor-sdk-bridge."""
    raise SystemExit(_lib.run_scenario("sigkill-stdin-hold-abandons-bridge"))

@qa_cli.command("all")
@click.option(
    "--code-only",
    is_flag=True,
    help="Skip live Cursor SDK scenarios (no API / no long agent runs).",
)
def qa_all_cmd(code_only: bool) -> None:
    """Run every scenario (exit 0 only if each reports FIXED)."""
    raise SystemExit(_lib.run_all(include_live=not code_only))

@qa_cli.command("self-test")
def qa_selftest_cmd() -> None:
    """Fast offline self-tests (no Cursor API)."""
    _lib.qa_cli_self_test()

__all__ = [
    "qa_cli",
    "qa_list_cmd",
    "qa_sigkill_stdin_hold_abandons_bridge",
    "qa_all_cmd",
    "qa_selftest_cmd",
]

if __name__ == "__main__":
    qa_cli()
