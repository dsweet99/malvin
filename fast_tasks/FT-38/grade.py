#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-38. No malvin/repo imports."""
from __future__ import annotations

import argparse
import csv
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-38"
GOLD_LR = 0.020044720273651614
GOLD_LOSS = 0.3831572287722192
LR_TOL_RATIO = 0.25  # |log10(lr/gold)| bound via relative band around gold
LR_TOL_ABS = 0.006
LOSS_TOL = 0.05
GRID_BEST = 0.3
NOTEBOOK_TRAIN_AT_GRID = 0.657286


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


def grid_best_lr(workspace: Path) -> float:
    path = workspace / "data" / "grid.csv"
    best_lr: float | None = None
    best_loss = float("inf")
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            lr = float(row["lr"])
            loss = float(row["train_loss"])
            if loss < best_loss or (loss == best_loss and (best_lr is None or lr < best_lr)):
                best_loss = loss
                best_lr = lr
    assert best_lr is not None
    return best_lr


def _lr_close(got: float, gold: float) -> bool:
    if got <= 0:
        return False
    if abs(got - gold) <= LR_TOL_ABS:
        return True
    return abs(math.log10(got) - math.log10(gold)) <= math.log10(1.0 + LR_TOL_RATIO)


def _probe_loss(workspace: Path, lr: float) -> float | None:
    tool = workspace / "tools" / "holdout.py"
    if not tool.is_file():
        return None
    proc = subprocess.run(
        [sys.executable, str(tool), "--lr", str(lr)],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return None
    for line in proc.stdout.splitlines():
        if line.startswith("holdout_loss="):
            try:
                return float(line.split("=", 1)[1])
            except ValueError:
                return None
    return None


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
    if set(data.keys()) != {"lr_star", "holdout_loss", "grid_best_lr"}:
        return 0
    try:
        lr_star = float(data["lr_star"])
        holdout = float(data["holdout_loss"])
        grid_lr = float(data["grid_best_lr"])
    except (TypeError, ValueError):
        return 0
    for key in ("lr_star", "holdout_loss", "grid_best_lr"):
        if isinstance(data[key], bool):
            return 0
    if abs(grid_lr - grid_best_lr(workspace)) > 1e-9:
        return 0
    if not _lr_close(lr_star, GOLD_LR):
        return 0
    if abs(holdout - GOLD_LOSS) > LOSS_TOL:
        return 0
    # Reported holdout must match the sealed probe at lr_star
    probed = _probe_loss(workspace, lr_star)
    if probed is None or abs(probed - holdout) > 1e-4:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    payload = {
        "lr_star": GOLD_LR,
        "holdout_loss": GOLD_LOSS,
        "grid_best_lr": grid_best_lr(workspace),
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

        gb = grid_best_lr(pass_ws)
        assert abs(gb - GRID_BEST) < 1e-12

        # Notebook decoy (train loss as holdout)
        decoy = {
            "lr_star": gb,
            "holdout_loss": NOTEBOOK_TRAIN_AT_GRID,
            "grid_best_lr": gb,
        }
        (pass_ws / "answer.json").write_text(json.dumps(decoy) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "grid decoy must fail"

        # Correct lr but notebook holdout number
        decoy2 = {
            "lr_star": GOLD_LR,
            "holdout_loss": NOTEBOOK_TRAIN_AT_GRID,
            "grid_best_lr": gb,
        }
        (pass_ws / "answer.json").write_text(json.dumps(decoy2) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "wrong holdout must fail"

        # Wrong grid_best
        decoy3 = {
            "lr_star": GOLD_LR,
            "holdout_loss": GOLD_LOSS,
            "grid_best_lr": 0.01,
        }
        (pass_ws / "answer.json").write_text(json.dumps(decoy3) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "wrong grid_best must fail"

        # Probe at grid winner (falsifies notebook equate, but not optimal)
        decoy4 = {
            "lr_star": gb,
            "holdout_loss": 1.57632610,
            "grid_best_lr": gb,
        }
        (pass_ws / "answer.json").write_text(json.dumps(decoy4) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "grid lr with true holdout must fail"

        # Local refine around grid winner still wrong
        decoy5 = {
            "lr_star": 0.18,
            "holdout_loss": 1.23320378,
            "grid_best_lr": gb,
        }
        (pass_ws / "answer.json").write_text(json.dumps(decoy5) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "local-refine decoy must fail"

        extra = {
            "lr_star": GOLD_LR,
            "holdout_loss": GOLD_LOSS,
            "grid_best_lr": gb,
            "note": "x",
        }
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
