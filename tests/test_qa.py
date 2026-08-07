"""Unit tests for ops/qa.py (no live Cursor agent)."""

from __future__ import annotations

from click.testing import CliRunner

from toolchain_repos import load_ops_entry


def test_qa_self_tests_via_cli() -> None:
    cli = load_ops_entry("qa").qa_cli
    result = CliRunner().invoke(cli, ["self-test"], catch_exceptions=False)
    assert result.exit_code == 0, result.output
    assert "ALL qa self-tests OK" in result.output


def test_qa_cli_list() -> None:
    cli = load_ops_entry("qa").qa_cli
    result = CliRunner().invoke(cli, ["list"], catch_exceptions=False)
    assert result.exit_code == 0, result.output
    assert "sigkill-stdin-hold-abandons-bridge" in result.output
    assert "(local)" in result.output
    assert "tool-child-survives-close" not in result.output
    assert "hard-kill-agent-busy" not in result.output


def test_qa_sigkill_stdin_hold_command_registered() -> None:
    cli = load_ops_entry("qa").qa_cli
    result = CliRunner().invoke(cli, ["sigkill-stdin-hold-abandons-bridge", "--help"])
    assert result.exit_code == 0, result.output
    assert "stdin-hold" in result.output.lower() or "abandons" in result.output.lower()
