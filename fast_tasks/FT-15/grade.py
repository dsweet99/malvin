#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-15. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import sys
import tempfile
import importlib.util
from pathlib import Path


TASK_ID = "FT-15"


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


CASES = [
    ([], []),
    ([[2, 2]], [[2, 2]]),
    ([[1, 3], [2, 4]], [[1, 4]]),
    ([[1, 2], [2, 3]], [[1, 3]]),  # abutting merges
    ([[5, 6], [1, 2], [2, 5]], [[1, 6]]),
    ([[1, 10], [2, 3], [4, 5]], [[1, 10]]),
    ([[-5, -1], [-1, 0], [2, 3]], [[-5, 0], [2, 3]]),
    ([[0, 0], [0, 1]], [[0, 1]]),
]


def _load(workspace: Path):
    path = workspace / "merge.py"
    name = "merge"
    # Execute from a temp copy outside workspace so __pycache__ cannot pollute the starter tree.
    td = tempfile.mkdtemp()
    try:
        dest = Path(td) / "merge.py"
        dest.write_bytes(path.read_bytes())
        spec = importlib.util.spec_from_file_location(name, dest)
        mod = importlib.util.module_from_spec(spec)
        assert spec.loader is not None
        spec.loader.exec_module(mod)
        return mod
    finally:
        shutil.rmtree(td, ignore_errors=True)


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    try:
        mod = _load(workspace)
        fn = mod.merge_intervals
    except Exception:
        return 0
    for inp, exp in CASES:
        try:
            out = fn([list(x) for x in inp])
        except Exception:
            return 0
        if out != exp:
            return 0
    return 1


ORACLE = '''
from typing import List

def merge_intervals(intervals: List[List[int]]) -> List[List[int]]:
    if not intervals:
        return []
    ints = sorted((list(x) for x in intervals), key=lambda p: (p[0], p[1]))
    out = [ints[0]]
    for a, b in ints[1:]:
        if a <= out[-1][1]:
            out[-1][1] = max(out[-1][1], b)
        else:
            out.append([a, b])
    return out
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "merge.py").write_text(ORACLE, encoding="utf-8")


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0
        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    p.add_argument("--workspace", type=Path, default=None)
    p.add_argument("--reward-out", type=Path, default=None)
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    ws = args.workspace or default_workspace()
    out = args.reward_out or default_reward_out()
    reward = evaluate(ws)
    write_reward(out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
