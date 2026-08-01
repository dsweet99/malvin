#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-08. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import sys
import tempfile
import importlib.util
from pathlib import Path


TASK_ID = "FT-08"


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
    path = workspace / "solve_system.py"
    name = "solve_system"
    # Execute from a temp copy outside workspace so __pycache__ cannot pollute the starter tree.
    td = tempfile.mkdtemp()
    try:
        dest = Path(td) / "solve_system.py"
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
        solve = mod.solve
    except Exception:
        return 0
    # Near-singular but with known solution (1, -1):
    # [1+1e-10, 1] [x] = [1e-10]
    # [1,       1] [y]   [-0]
    # Wait craft: A @ [1,-1] = b
    eps = 1e-12
    a11, a12, a21, a22 = 1.0 + eps, 1.0, 1.0, 1.0
    x_true, y_true = 1.0, -1.0
    b1 = a11 * x_true + a12 * y_true
    b2 = a21 * x_true + a22 * y_true
    try:
        x, y = solve(a11, a12, a21, a22, b1, b2)
    except Exception:
        return 0
    if max(abs(x - 1.0), abs(y + 1.0)) > 1e-9:
        return 0
    # Well-conditioned
    a11, a12, a21, a22 = 2.0, 1.0, 1.0, 2.0
    b1, b2 = 1.0, -1.0
    # solution: solve 2x+y=1; x+2y=-1 => x=1, y=-1
    try:
        x, y = solve(a11, a12, a21, a22, b1, b2)
    except Exception:
        return 0
    if max(abs(x - 1.0), abs(y + 1.0)) > 1e-12:
        return 0
    return 1


ORACLE = '''
def solve(a11, a12, a21, a22, b1, b2):
    det = a11 * a22 - a12 * a21
    if det == 0:
        raise ZeroDivisionError("singular")
    x = (b1 * a22 - a12 * b2) / det
    y = (a11 * b2 - b1 * a21) / det
    return (x, y)
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "solve_system.py").write_text(ORACLE, encoding="utf-8")


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
