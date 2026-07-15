#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-05. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-05"


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


GOLD = {
    "claim": "Resting membrane potential is approximately -70 mV under baseline ACSF.",
    "source_path": "corpus/03_nguyen.md",
    "author": "Nguyen",
}


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
    claim = data.get("claim")
    if not isinstance(claim, str) or len(claim) > 120:
        return 0
    if "-70" not in claim or "mV" not in claim:
        return 0
    for k in ("claim", "source_path", "author"):
        if data.get(k) != GOLD[k]:
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
