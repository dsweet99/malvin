#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-40. No malvin/repo imports."""
from __future__ import annotations

import argparse
import csv
import importlib.util
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-40"
HOLD_MSE_MAX = 0.005
NOTEBOOK_MID_SSE = 0.000421955831
SEALED_XS = (0.05, 0.1, 0.2, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0)
ORACLE_PREDICT = '''"""Oracle response law for FT-40 (logarithmic in x)."""

import math


def predict(x: float) -> float:
    return 1.173 + 0.842 * math.log(1.0 + x)
'''


def oracle_y(x: float) -> float:
    return 1.173 + 0.842 * math.log(1.0 + x)


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


def _load_predict(workspace: Path):
    path = workspace / "predict.py"
    if not path.is_file():
        return None
    spec = importlib.util.spec_from_file_location("ft40_grade_predict", path)
    if spec is None or spec.loader is None:
        return None
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except Exception:
        return None
    fn = getattr(mod, "predict", None)
    return fn if callable(fn) else None


def mid_sse(workspace: Path, predict) -> float:
    path = workspace / "data" / "midrange.csv"
    total = 0.0
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            x = float(row["x"])
            y = float(row["y"])
            err = float(predict(x)) - y
            total += err * err
    return total


def sealed_mse(predict) -> float:
    return sum((float(predict(x)) - oracle_y(x)) ** 2 for x in SEALED_XS) / len(SEALED_XS)


def evaluate(workspace: Path) -> int:
    path = workspace / "answer.json"
    if not path.is_file():
        return 0
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return 0
    if not isinstance(data, dict) or set(data.keys()) != {"mid_sse"}:
        return 0
    try:
        claimed = float(data["mid_sse"])
    except (TypeError, ValueError):
        return 0
    if isinstance(data["mid_sse"], bool):
        return 0
    predict = _load_predict(workspace)
    if predict is None:
        return 0
    try:
        actual = mid_sse(workspace, predict)
    except Exception:
        return 0
    if abs(claimed - actual) > 1e-6:
        return 0
    try:
        mse = sealed_mse(predict)
    except Exception:
        return 0
    if mse > HOLD_MSE_MAX:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    (workspace / "predict.py").write_text(ORACLE_PREDICT, encoding="utf-8")
    predict = _load_predict(workspace)
    assert predict is not None
    (workspace / "answer.json").write_text(
        json.dumps({"mid_sse": mid_sse(workspace, predict)}, indent=2) + "\n",
        encoding="utf-8",
    )


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        (fail_ws / "answer.json").write_text(
            json.dumps({"mid_sse": NOTEBOOK_MID_SSE}) + "\n", encoding="utf-8"
        )
        assert evaluate(fail_ws) == 0, "notebook MM must fail grade"
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py")],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "holdout_status=PASS" in proc.stdout, proc.stdout
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "public_check.py")],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert proc.returncode == 0, proc.stderr

        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"

        # mid-fit log from CSV should also pass
        log_ws = Path(td) / "logfit"
        shutil.copytree(src, log_ws)
        rows = []
        with (log_ws / "data" / "midrange.csv").open(encoding="utf-8") as f:
            for row in csv.DictReader(f):
                rows.append((float(row["x"]), float(row["y"])))
        n = len(rows)
        sL = sum(math.log(1 + x) for x, _ in rows)
        sy = sum(y for _, y in rows)
        sLL = sum(math.log(1 + x) ** 2 for x, _ in rows)
        sLy = sum(y * math.log(1 + x) for x, y in rows)
        b = (n * sLy - sL * sy) / (n * sLL - sL * sL)
        a = (sy - b * sL) / n
        (log_ws / "predict.py").write_text(
            "import math\n\ndef predict(x: float) -> float:\n"
            f"    return {a!r} + {b!r} * math.log(1.0 + x)\n",
            encoding="utf-8",
        )
        pred = _load_predict(log_ws)
        (log_ws / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(log_ws, pred)}) + "\n", encoding="utf-8"
        )
        assert evaluate(log_ws) == 1, "mid-fit log must pass"

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
