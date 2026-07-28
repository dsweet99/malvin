"""Packaging contract for sibling fast_tasks/ eval tree (not kiss-covered; see .kissignore)."""
from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
FAST = ROOT / "fast_tasks"
_SPEC = importlib.util.spec_from_file_location(
    "fast_tasks_run_selftests", FAST / "run_selftests.py"
)
assert _SPEC is not None and _SPEC.loader is not None
_FT = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_FT)

TASK_IDS = list(_FT.TASK_IDS)


def test_fast_tasks_inventory() -> None:
    assert FAST.is_dir()
    assert (FAST / "README.md").is_file()
    assert (FAST / "run_selftests.py").is_file()
    found = {p.name for p in FAST.iterdir() if p.is_dir() and p.name.startswith("FT-")}
    assert found == set(TASK_IDS)
    for tid in TASK_IDS:
        assert (FAST / tid / "workspace" / "plan.md").is_file()
        assert (FAST / tid / "grade.py").is_file()


def test_fast_tasks_inventory_contracts() -> None:
    """DROPPED / plan length / import isolation from run_selftests.check_inventory."""
    before = _FT.check_inventory()
    assert before
    assert not (set(before) & _FT.DROPPED)


@pytest.mark.parametrize("tid", TASK_IDS)
def test_fast_tasks_starter_reward_is_zero(tid: str) -> None:
    import shutil
    import tempfile
    from pathlib import Path as P

    task = FAST / tid
    with tempfile.TemporaryDirectory() as td:
        ws_copy = P(td) / "workspace"
        shutil.copytree(task / "workspace", ws_copy)
        reward = P(td) / "reward.txt"
        proc = subprocess.run(
            [
                sys.executable,
                str(task / "grade.py"),
                "--workspace",
                str(ws_copy),
                "--reward-out",
                str(reward),
            ],
            cwd="/tmp",
            capture_output=True,
            text=True,
            env={**os.environ, "PYTHONPATH": "", "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"},
            check=False,
        )
        assert proc.returncode == 0, (tid, proc.stderr, proc.stdout)
        assert reward.read_text(encoding="utf-8").strip() == "0", tid


@pytest.mark.parametrize("tid", TASK_IDS)
def test_fast_tasks_grader_selftest(tid: str) -> None:
    ws = FAST / tid / "workspace"
    before = _FT.workspace_hash(ws)
    grade = FAST / tid / "grade.py"
    proc = subprocess.run(
        [sys.executable, str(grade), "--self-test"],
        cwd="/tmp",
        capture_output=True,
        text=True,
        env={**os.environ, "PYTHONPATH": "", "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"},
        check=False,
    )
    assert proc.returncode == 0, f"{tid} self-test failed\n{proc.stdout}\n{proc.stderr}"
    assert f"{tid} self-test OK" in proc.stdout
    assert _FT.workspace_hash(ws) == before, f"{tid} workspace mutated by self-test"
