#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-31. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-31"

GOLD = {
    "california_overseas_2020": 38534,
    "wyoming_overseas_2020": 868,
    "california_house_seats_2020": 52,
    "answer": 2288,
}


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


def evaluate(workspace: Path) -> int:
    path = workspace / "answer.json"
    if not path.is_file():
        return 0
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return 0
    if not isinstance(data, dict):
        return 0
    for key, expected in GOLD.items():
        value = data.get(key)
        if not isinstance(value, int) or isinstance(value, bool):
            return 0
        if value != expected:
            return 0
    # Metamorphic check: answer must equal floor(A/B)*C from the same payload.
    a = data["california_overseas_2020"]
    b = data["wyoming_overseas_2020"]
    c = data["california_house_seats_2020"]
    if b == 0 or data["answer"] != (a // b) * c:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    (workspace / "answer.json").write_text(json.dumps(GOLD, indent=2) + "\n", encoding="utf-8")


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
        # Reject internally inconsistent payloads even if fields look plausible.
        bad = dict(GOLD)
        bad["answer"] = GOLD["answer"] + 1
        (pass_ws / "answer.json").write_text(json.dumps(bad) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0
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
