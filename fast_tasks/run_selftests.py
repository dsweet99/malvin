#!/usr/bin/env python3
"""Inventory + import-isolation + per-task grader self-tests for fast_tasks/."""
from __future__ import annotations

import ast
import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
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
    "FT-23",
]
DROPPED = {
    "FT-02",
    "FT-04",
    "FT-06",
    "FT-07",
    "FT-10",
    "FT-11",
    "FT-14",
    "FT-16",
    "FT-18",
    "FT-19",
    "FT-21",
    "FT-22",
}
FORBIDDEN_IMPORT_ROOTS = {"malvin", "ops", "src"}


def workspace_hash(ws: Path) -> str:
    h = hashlib.sha256()
    files = []
    for f in ws.rglob("*"):
        if not f.is_file():
            continue
        if "__pycache__" in f.parts or f.suffix == ".pyc":
            continue
        if ".pytest_cache" in f.parts:
            continue
        if f.name == "reward.txt":
            continue
        if f.name == "port" and "var" in f.parts:
            # leftover FT-12 side effect if any older grades ran in-place
            continue
        files.append(f)
    for f in sorted(files):
        h.update(f.relative_to(ws).as_posix().encode())
        h.update(b"\0")
        h.update(f.read_bytes())
    return h.hexdigest()


def check_imports(grade_py: Path) -> None:
    tree = ast.parse(grade_py.read_text(encoding="utf-8"), filename=str(grade_py))
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                root = alias.name.split(".")[0]
                if root in FORBIDDEN_IMPORT_ROOTS:
                    raise AssertionError(f"{grade_py}: forbidden import {alias.name}")
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                root = node.module.split(".")[0]
                if root in FORBIDDEN_IMPORT_ROOTS:
                    raise AssertionError(f"{grade_py}: forbidden import from {node.module}")


def check_inventory() -> dict[str, str]:
    assert ROOT.is_dir(), "fast_tasks/ missing"
    readme = ROOT / "README.md"
    assert readme.is_file() and readme.stat().st_size > 0
    text = readme.read_text(encoding="utf-8")
    for needle in ("plan.md", "grade.py", "reward", "run_selftests.py", "malvin"):
        assert needle in text, f"README missing section cue: {needle}"

    found = {p.name for p in ROOT.iterdir() if p.is_dir() and p.name.startswith("FT-")}
    assert found == set(TASK_IDS), f"task dirs {found} != {set(TASK_IDS)}"
    assert not (found & DROPPED), f"dropped ids present: {found & DROPPED}"

    hashes: dict[str, str] = {}
    for tid in TASK_IDS:
        task = ROOT / tid
        plan = task / "workspace" / "plan.md"
        grade = task / "grade.py"
        ws = task / "workspace"
        assert plan.is_file() and plan.stat().st_size > 0, tid
        assert grade.is_file() and grade.stat().st_size > 0, tid
        assert any(ws.rglob("*")), tid
        words = len(plan.read_text(encoding="utf-8").split())
        assert words >= 80, f"{tid} plan too short ({words} words)"
        check_imports(grade)
        hashes[tid] = workspace_hash(ws)
    return hashes


def run_selftests() -> None:
    for tid in TASK_IDS:
        grade = ROOT / tid / "grade.py"
        proc = subprocess.run(
            [sys.executable, str(grade), "--self-test"],
            cwd="/tmp",
            capture_output=True,
            text=True,
            env={**dict(**{k: v for k, v in __import__("os").environ.items()}), "PYTHONPATH": ""},
        )
        if proc.returncode != 0:
            raise AssertionError(
                f"{tid} self-test failed\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
            )
        print(proc.stdout.strip())


def grade_starters_are_zero() -> None:
    import tempfile
    from pathlib import Path as P

    for tid in TASK_IDS:
        task = ROOT / tid
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
                env={**dict(**{k: v for k, v in __import__("os").environ.items()}), "PYTHONPATH": ""},
            )
            assert proc.returncode == 0, (tid, proc.stderr)
            val = reward.read_text(encoding="utf-8").strip()
            assert val == "0", f"{tid} starter reward={val!r} (expected 0)"


def main() -> int:
    before = check_inventory()
    grade_starters_are_zero()
    run_selftests()
    after = {tid: workspace_hash(ROOT / tid / "workspace") for tid in TASK_IDS}
    assert before == after, "workspace mutated by self-tests"
    # reward alphabet already enforced by write_reward asserts + starter==0
    print("ALL fast_tasks self-tests OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
