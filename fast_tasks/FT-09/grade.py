#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-09. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-09"


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


GOLD_R = 0.9999625209322575
GOLD_MX = 9.75
GOLD_MY = 19.5


def evaluate(workspace: Path) -> int:
    path = workspace / "answer.json"
    if not path.is_file():
        return 0
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return 0
    if data.get("n") != 40:
        return 0
    try:
        r = float(data["pearson_r"])
        mx = float(data["mean_x"])
        my = float(data["mean_y"])
    except (KeyError, TypeError, ValueError):
        return 0
    if abs(r - GOLD_R) > 1e-6:
        return 0
    if abs(mx - GOLD_MX) > 1e-9 or abs(my - GOLD_MY) > 1e-9:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    payload = {"n": 40, "pearson_r": GOLD_R, "mean_x": GOLD_MX, "mean_y": GOLD_MY}
    (workspace / "answer.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


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
