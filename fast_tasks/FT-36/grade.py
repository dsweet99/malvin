#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-36. No malvin/repo imports."""
from __future__ import annotations

import argparse
import csv
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-36"
GOLD_CAUSE = "Z"
GOLD_EFFECT = 2.0
EFFECT_TOL = 0.15
OBS_GAP_TOL = 0.05


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


def obs_x_gap(workspace: Path) -> float:
    path = workspace / "data" / "observational.csv"
    ys0: list[float] = []
    ys1: list[float] = []
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            y = float(row["Y"])
            if int(float(row["X"])) == 0:
                ys0.append(y)
            else:
                ys1.append(y)
    return sum(ys1) / len(ys1) - sum(ys0) / len(ys0)


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
    if set(data.keys()) != {"cause", "effect", "obs_x_gap"}:
        return 0
    if data.get("cause") != GOLD_CAUSE:
        return 0
    try:
        effect = float(data["effect"])
        gap = float(data["obs_x_gap"])
    except (TypeError, ValueError):
        return 0
    if isinstance(data["effect"], bool) or isinstance(data["obs_x_gap"], bool):
        return 0
    if abs(effect - GOLD_EFFECT) > EFFECT_TOL:
        return 0
    if abs(gap - obs_x_gap(workspace)) > OBS_GAP_TOL:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    payload = {
        "cause": GOLD_CAUSE,
        "effect": GOLD_EFFECT,
        "obs_x_gap": obs_x_gap(workspace),
    }
    (workspace / "answer.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0, "starter must fail"

        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"

        gap = obs_x_gap(pass_ws)
        decoy = {"cause": "X", "effect": gap, "obs_x_gap": gap}
        (pass_ws / "answer.json").write_text(json.dumps(decoy) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "observational X decoy must fail"

        decoy2 = {"cause": "W", "effect": GOLD_EFFECT, "obs_x_gap": gap}
        (pass_ws / "answer.json").write_text(json.dumps(decoy2) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "W decoy must fail"

        decoy3 = {"cause": "Z", "effect": gap, "obs_x_gap": gap}
        (pass_ws / "answer.json").write_text(json.dumps(decoy3) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "wrong effect must fail"

        decoy4 = {"cause": "Z", "effect": GOLD_EFFECT, "obs_x_gap": 0.0}
        (pass_ws / "answer.json").write_text(json.dumps(decoy4) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "wrong obs gap must fail"

        extra = {"cause": "Z", "effect": GOLD_EFFECT, "obs_x_gap": gap, "note": "x"}
        (pass_ws / "answer.json").write_text(json.dumps(extra) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "extra keys must fail"

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
