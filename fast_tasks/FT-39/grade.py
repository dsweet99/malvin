#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-39. No malvin/repo imports."""
from __future__ import annotations

import argparse
import csv
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-39"
HOLD_MSE_MAX = 0.002
NOTEBOOK_MID_SSE = 0.000217
SEALED_XS = (0.2, 0.3, 0.5, 0.8, 7.0, 10.0, 14.0, 20.0)
ORACLE_PREDICT = '''"""Oracle response law for FT-39 (saturating response in x)."""


def predict(x: float) -> float:
    return 2.0 * x / (3.0 + x) + 1.0
'''


def oracle_y(x: float) -> float:
    return 2.0 * x / (3.0 + x) + 1.0


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


def mid_rows(workspace: Path) -> list[tuple[float, float]]:
    path = workspace / "data" / "midrange.csv"
    rows: list[tuple[float, float]] = []
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            rows.append((float(row["x"]), float(row["y"])))
    return rows


def load_predict(workspace: Path):
    path = workspace / "predict.py"
    if not path.is_file():
        raise FileNotFoundError("predict.py")
    spec = importlib.util.spec_from_file_location("ft39_predict_grade", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load predict.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, "predict", None)
    if not callable(fn):
        raise RuntimeError("predict.py must define callable predict(x)")
    return fn


def mid_sse(predict, rows: list[tuple[float, float]]) -> float:
    return sum((y - float(predict(x))) ** 2 for x, y in rows)


def sealed_mse(predict) -> float:
    return sum((float(predict(x)) - oracle_y(x)) ** 2 for x in SEALED_XS) / len(
        SEALED_XS
    )


def _agent_holdout_pass(workspace: Path) -> bool | None:
    """Agent-visible probe is pass/fail only (no continuous MSE oracle)."""
    tool = workspace / "tools" / "holdout.py"
    if not tool.is_file():
        return None
    proc = subprocess.run(
        [sys.executable, str(tool)],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return None
    for line in proc.stdout.splitlines():
        if line.strip() == "holdout_status=PASS":
            return True
        if line.strip() == "holdout_status=FAIL":
            return False
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
    if set(data.keys()) != {"mid_sse"}:
        return 0
    try:
        mid = float(data["mid_sse"])
    except (TypeError, ValueError):
        return 0
    if isinstance(data["mid_sse"], bool):
        return 0
    try:
        predict = load_predict(workspace)
    except Exception:
        return 0
    rows = mid_rows(workspace)
    expected = mid_sse(predict, rows)
    if abs(mid - expected) > 1e-6:
        return 0
    if sealed_mse(predict) > HOLD_MSE_MAX:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    (workspace / "predict.py").write_text(ORACLE_PREDICT, encoding="utf-8")
    predict = load_predict(workspace)
    rows = mid_rows(workspace)
    payload = {"mid_sse": mid_sse(predict, rows)}
    (workspace / "answer.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0, "starter must fail"

        # Notebook answer.json alone still fails (linear predict)
        nb = {"mid_sse": NOTEBOOK_MID_SSE}
        (fail_ws / "answer.json").write_text(json.dumps(nb) + "\n", encoding="utf-8")
        assert evaluate(fail_ws) == 0, "notebook decoy must fail"
        nb_pred = load_predict(fail_ws)
        assert sealed_mse(nb_pred) > 0.1
        assert _agent_holdout_pass(fail_ws) is False

        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"
        oracle_pred = load_predict(pass_ws)
        assert abs(sealed_mse(oracle_pred)) <= 1e-9
        assert _agent_holdout_pass(pass_ws) is True

        # Local linear coefficient polish still fails sealed
        polish = Path(td) / "polish"
        shutil.copytree(src, polish)
        (polish / "predict.py").write_text(
            "def predict(x: float) -> float:\n"
            "    return 1.6 + 0.12 * x\n",
            encoding="utf-8",
        )
        pred = load_predict(polish)
        rows = mid_rows(polish)
        (polish / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(pred, rows)}) + "\n", encoding="utf-8"
        )
        assert evaluate(polish) == 0, "linear local-tweak must fail"

        # Inverse distractor must fail
        inv = Path(td) / "inv"
        shutil.copytree(src, inv)
        (inv / "predict.py").write_text(
            "def predict(x: float) -> float:\n"
            "    return 1.0 + 2.0 / x\n",
            encoding="utf-8",
        )
        pred = load_predict(inv)
        rows = mid_rows(inv)
        (inv / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(pred, rows)}) + "\n", encoding="utf-8"
        )
        assert evaluate(inv) == 0, "inverse distractor must fail"

        # Exponential distractor must fail
        exp_d = Path(td) / "exp"
        shutil.copytree(src, exp_d)
        (exp_d / "predict.py").write_text(
            "import math\n"
            "def predict(x: float) -> float:\n"
            "    return 2.5 * math.exp(-0.4 * x) + 1.2\n",
            encoding="utf-8",
        )
        pred = load_predict(exp_d)
        rows = mid_rows(exp_d)
        (exp_d / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(pred, rows)}) + "\n", encoding="utf-8"
        )
        assert evaluate(exp_d) == 0, "exponential distractor must fail"

        # Best-of-holdout log fit must fail under tight tol
        log_d = Path(td) / "log"
        shutil.copytree(src, log_d)
        (log_d / "predict.py").write_text(
            "import math\n"
            "def predict(x: float) -> float:\n"
            "    return 1.6244179116506023 + 0.3807363046352132 * math.log(x)\n",
            encoding="utf-8",
        )
        pred = load_predict(log_d)
        rows = mid_rows(log_d)
        (log_d / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(pred, rows)}) + "\n", encoding="utf-8"
        )
        assert evaluate(log_d) == 0, "log distractor must fail"

        # Near-gold saturating law should pass
        near = Path(td) / "near"
        shutil.copytree(src, near)
        (near / "predict.py").write_text(
            "def predict(x: float) -> float:\n"
            "    return 1.95 * x / (3.1 + x) + 1.02\n",
            encoding="utf-8",
        )
        pred = load_predict(near)
        rows = mid_rows(near)
        (near / "answer.json").write_text(
            json.dumps({"mid_sse": mid_sse(pred, rows)}) + "\n", encoding="utf-8"
        )
        assert evaluate(near) == 1, "near saturating fit must pass"

        # Inconsistent mid_sse
        bad = Path(td) / "bad"
        shutil.copytree(src, bad)
        _oracle_fix(bad)
        (bad / "answer.json").write_text(
            json.dumps({"mid_sse": 9.99}) + "\n", encoding="utf-8"
        )
        assert evaluate(bad) == 0, "inconsistent mid_sse must fail"

        extra = Path(td) / "extra"
        shutil.copytree(src, extra)
        _oracle_fix(extra)
        payload = json.loads((extra / "answer.json").read_text(encoding="utf-8"))
        payload["note"] = "x"
        (extra / "answer.json").write_text(json.dumps(payload) + "\n", encoding="utf-8")
        assert evaluate(extra) == 0, "extra keys must fail"

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
