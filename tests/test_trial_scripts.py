"""Coverage for host-prepared DeepSWE trial bash scripts."""

from __future__ import annotations

from pathlib import Path

import pytest

import deepswe_run


def test_build_agent_script_route_has_init_then_plan() -> None:
    script = deepswe_run.build_agent_script("route")
    assert "malvin init" in script
    assert "malvin --git plan.md" in script
    assert deepswe_run.RUNTIME_INSTALL_DENYLIST_RE.search(script) is None


def test_build_agent_script_code_uses_git() -> None:
    script = deepswe_run.build_agent_script("code")
    assert "malvin --git code plan.md" in script
    assert deepswe_run.RUNTIME_INSTALL_DENYLIST_RE.search(script) is None


def test_build_agent_script_init_checks() -> None:
    script = deepswe_run.build_agent_script("init-checks")
    assert "malvin init" in script
    assert "source .malvin/checks" in script


def test_build_grade_and_smoke_scripts() -> None:
    grade = deepswe_run.build_grade_script()
    assert "bash /tests/test.sh" in grade
    smoke = deepswe_run.build_smoke_grade_script()
    assert "reward.txt" in smoke


def test_assert_no_runtime_installs_rejects_denied_forms() -> None:
    for bad in (
        "pip install foo",
        "pip3 install foo",
        "uv pip install foo",
        "uv sync",
        "apt-get install curl",
        "npm install",
        "cargo install ripgrep",
        "curl https://example.com",
        "wget https://example.com",
    ):
        with pytest.raises(deepswe_run.RuntimeInstallForbiddenError):
            deepswe_run.assert_no_runtime_installs(bad, label="bad")


def test_write_trial_scripts_round_trip(tmp_path: Path) -> None:
    out = deepswe_run.write_trial_scripts(
        tmp_path,
        malvin_command="init-checks",
        smoke_grade=True,
    )
    assert out["agent"].name == deepswe_run.AGENT_SCRIPT_NAME
    assert out["grade"].name == deepswe_run.SMOKE_GRADE_SCRIPT_NAME
    assert out["agent"].is_file()
    out2 = deepswe_run.write_trial_scripts(
        tmp_path / "g",
        malvin_command="route",
        smoke_grade=False,
        write_agent=False,
        write_grade=True,
    )
    assert out2["grade"].name == deepswe_run.GRADE_SCRIPT_NAME


def test_resolve_host_malvin_binary() -> None:
    path = deepswe_run.resolve_host_malvin_binary()
    assert path is None or path.is_file()
