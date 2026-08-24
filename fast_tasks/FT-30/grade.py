#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-30. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-30"
REQUIRED_CHECKS = ("kiss check", "pytest tests")

ORACLE_INIT = '''\
"""Public stats helpers for this workspace."""

from statsutil._rolling import rolling_mean

__all__ = ["rolling_mean"]
'''


def write_reward(path: Path, value: int) -> None:
    assert value in (0, 1)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{value}\n", encoding="utf-8")


def default_workspace() -> Path:
    return Path(__file__).resolve().parent / "workspace"


def default_reward_out() -> Path:
    env = os.environ.get("MALVIN_REWARD_PATH") or os.environ.get("HARBOR_REWARD_PATH")
    if env:
        return Path(env)
    return Path(__file__).resolve().parent / "reward.txt"


def _checks_ok(workspace: Path) -> bool:
    checks = workspace / ".malvin" / "checks"
    if not checks.is_file():
        return False
    lines = [
        line.strip()
        for line in checks.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    ]
    return list(lines) == list(REQUIRED_CHECKS)


def _run(cmd: list[str], *, cwd: Path, env: dict[str, str] | None = None) -> int:
    merged = {**os.environ, "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1", **(env or {})}
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        env=merged,
        check=False,
    )
    return int(proc.returncode)


def _kiss_available() -> bool:
    return shutil.which("kiss") is not None


def evaluate(workspace: Path) -> int:
    workspace = workspace.resolve()
    if not _checks_ok(workspace):
        return 0
    if not _kiss_available():
        return 0
    if _run(["kiss", "check"], cwd=workspace) != 0:
        return 0
    if _run(
        [sys.executable, "-m", "pytest", "-q", "tests", "-p", "no:cacheprovider"],
        cwd=workspace,
    ) != 0:
        return 0

    hidden = Path(__file__).resolve().parent / "goldens" / "test_rolling_hidden.py"
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        shutil.copytree(workspace / "statsutil", td_path / "statsutil")
        (td_path / "tests").mkdir()
        shutil.copy2(hidden, td_path / "tests" / "test_rolling_hidden.py")
        (td_path / "pytest.ini").write_text("[pytest]\npythonpath = .\n", encoding="utf-8")
        if _run(
            [sys.executable, "-m", "pytest", "-q", "tests", "-p", "no:cacheprovider"],
            cwd=td_path,
        ) != 0:
            return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    (workspace / "statsutil" / "__init__.py").write_text(ORACLE_INIT, encoding="utf-8")


def _with_stub_kiss_if_needed(td: Path) -> None:
    """Ensure ``kiss`` is on PATH for self-test when the host has no kiss-ai.

    Real grading still requires a real ``kiss`` binary; evaluate() returns 0
    when kiss is missing. Self-test only needs a successful exit from the
    ``kiss check`` gate line so pytest / hidden-test logic can be verified.
    """
    if _kiss_available():
        return
    bin_dir = td / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    stub = bin_dir / "kiss"
    stub.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    stub.chmod(0o755)
    os.environ["PATH"] = f"{bin_dir}{os.pathsep}{os.environ.get('PATH', '')}"


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        _with_stub_kiss_if_needed(td_path)
        fail_ws = td_path / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0, "starter must fail"

        pass_ws = td_path / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    parser.add_argument("--workspace", type=Path, default=None)
    parser.add_argument("--reward-out", type=Path, default=None)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    workspace = args.workspace or default_workspace()
    reward_out = args.reward_out or default_reward_out()
    reward = evaluate(workspace)
    write_reward(reward_out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
