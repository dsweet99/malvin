"""Packaging contract for sibling fast_tasks/ eval tree (not kiss-covered; see .kissignore)."""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FAST = ROOT / "fast_tasks"
TASK_IDS = [
    "FT-01",
    "FT-03",
    "FT-13",
    "FT-12",
    "FT-05",
    "FT-09",
    "FT-17",
    "FT-08",
    "FT-20",
    "FT-15",
    "FT-24",
    "FT-25",
    "FT-26",
    "FT-27",
    "FT-28",
    "FT-29",
    "FT-30",
    "FT-31",
    "FT-32",
    "FT-33",
    "FT-34",
    "FT-35",
]


def test_fast_tasks_inventory() -> None:
    assert FAST.is_dir()
    assert (FAST / "README.md").is_file()
    assert (FAST / "run_selftests.py").is_file()
    found = {p.name for p in FAST.iterdir() if p.is_dir() and p.name.startswith("FT-")}
    assert found == set(TASK_IDS)
    for tid in TASK_IDS:
        assert (FAST / tid / "workspace" / "plan.md").is_file()
        assert (FAST / tid / "grade.py").is_file()


def test_fast_tasks_grader_selftests() -> None:
    proc = subprocess.run(
        [sys.executable, str(FAST / "run_selftests.py")],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stdout + "\n" + proc.stderr
    assert "ALL fast_tasks self-tests OK" in proc.stdout
