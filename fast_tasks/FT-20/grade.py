#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-20. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import statistics
import sys
import tempfile
import time
import importlib.util
from pathlib import Path


TASK_ID = "FT-20"


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


def _load(workspace: Path):
    path = workspace / "uniq.py"
    name = "uniq"
    # Execute from a temp copy outside workspace so __pycache__ cannot pollute the starter tree.
    td = tempfile.mkdtemp()
    try:
        dest = Path(td) / "uniq.py"
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
        fn = mod.unique_sorted
    except Exception:
        return 0
    cases = [
        ([], []),
        ([3, 1, 2, 1], [1, 2, 3]),
        ([-2, -2, 0, 5], [-2, 0, 5]),
        ([1], [1]),
    ]
    for inp, exp in cases:
        try:
            out = fn(list(inp))
        except Exception:
            return 0
        if out != exp:
            return 0
    data = list(range(5000)) + list(range(5000))
    times = []
    for _ in range(3):
        t0 = time.perf_counter()
        out = fn(list(data))
        t1 = time.perf_counter()
        times.append(t1 - t0)
        if out != list(range(5000)):
            return 0
    if statistics.median(times) > 0.020:
        return 0
    return 1


ORACLE = '''
def unique_sorted(xs: list[int]) -> list[int]:
    return sorted(set(xs))
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "uniq.py").write_text(ORACLE, encoding="utf-8")


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
        # O(n^2) should fail timing
        slow = Path(td) / "slow"
        shutil.copytree(src, slow)
        (slow / "uniq.py").write_text(
            "def unique_sorted(xs):\n"
            "    out = []\n"
            "    for x in xs:\n"
            "        if x not in out:\n"
            "            out.append(x)\n"
            "    out.sort()\n"
            "    return out\n",
            encoding="utf-8",
        )
        assert evaluate(slow) == 0
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
