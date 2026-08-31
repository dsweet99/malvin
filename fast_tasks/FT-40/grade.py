#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-40. No malvin/repo imports."""
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


TASK_ID = "FT-40"
GOLD_XS = (100.0,)
X_TOL_ABS = 1.5
SCORE_TOL = 0.12
GRID_BEST = 45.0
NOTEBOOK_SCORE = 1.0
TRIAL_LO = 25.0
TRIAL_HI = 65.0
CONFIRM_X = 45.0
PROBE_SCORE_TOL = 1e-3
HOLDOUT_SCORE_FLOOR = 1.0
POLISH_HOLE_LO = 40.0
POLISH_HOLE_HI = 55.0


def oracle_score(x: float) -> float:
    """Sealed law: mid polish decoy (>1) + global peak at endpoint x=100."""
    return (
        0.35 * math.exp(-(((x - 45.0) / 7.0) ** 2))
        + 1.25 * math.exp(-(((x - 52.0) / 4.0) ** 2))
        + 2.20 * math.exp(-(((x - 100.0) / 2.5) ** 2))
    )


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


def grid_best_x(workspace: Path) -> float:
    path = workspace / "data" / "grid.csv"
    best_x = None
    best_score = float("-inf")
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            x = float(row["x"])
            score = float(row["score"])
            if score > best_score or (score == best_score and (best_x is None or x < best_x)):
                best_score = score
                best_x = x
    assert best_x is not None
    return best_x


def _load_holdout_probes(workspace: Path) -> list[dict]:
    path = workspace / ".holdout_probes.jsonl"
    if not path.is_file():
        return []
    out: list[dict] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return out


def _holdout_protocol_ok(workspace: Path, x_star: float, score: float) -> bool:
    if not (workspace / ".holdout_confirm_done").is_file():
        return False
    count_path = workspace / ".holdout_query_count"
    if not count_path.is_file():
        return False
    try:
        if int(count_path.read_text(encoding="utf-8").strip()) != 1:
            return False
    except ValueError:
        return False
    probes = _load_holdout_probes(workspace)
    if len(probes) != 2:
        return False
    try:
        if abs(float(probes[0]["x"]) - CONFIRM_X) > 1e-9:
            return False
        disc_x = float(probes[1]["x"])
    except (KeyError, TypeError, ValueError, IndexError):
        return False
    if POLISH_HOLE_LO <= disc_x <= POLISH_HOLE_HI:
        return False
    if score <= HOLDOUT_SCORE_FLOOR + 1e-9:
        return False
    matched = False
    for probe in probes:
        try:
            px = float(probe["x"])
            py = float(probe["sealed_score"])
        except (KeyError, TypeError, ValueError):
            return False
        if abs(py - oracle_score(px)) > PROBE_SCORE_TOL:
            return False
        if abs(px - x_star) <= 1e-9 and abs(py - score) <= PROBE_SCORE_TOL:
            matched = True
    return matched


def _probe_trial_score(workspace: Path, x: float) -> float | None:
    tool = workspace / "tools" / "score.py"
    if not tool.is_file():
        return None
    proc = subprocess.run(
        [sys.executable, str(tool), "--x", str(x)],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return None
    for line in proc.stdout.splitlines():
        if line.startswith("score="):
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
    if not isinstance(data, dict) or set(data.keys()) != {"x_star", "score", "grid_best_x"}:
        return 0
    try:
        x_star = float(data["x_star"])
        score = float(data["score"])
        grid_best = float(data["grid_best_x"])
    except (TypeError, ValueError):
        return 0
    for key in ("x_star", "score", "grid_best_x"):
        if isinstance(data[key], bool):
            return 0
    if not math.isclose(grid_best, grid_best_x(workspace), abs_tol=1e-9):
        return 0
    if TRIAL_LO <= x_star <= TRIAL_HI:
        probed = _probe_trial_score(workspace, x_star)
        if probed is None or abs(score - probed) > 1e-6:
            return 0
    true = oracle_score(x_star)
    if min(abs(x_star - g) for g in GOLD_XS) > X_TOL_ABS:
        return 0
    if score < 1.5:
        return 0
    if abs(score - true) > SCORE_TOL:
        return 0
    if not _holdout_protocol_ok(workspace, x_star, score):
        return 0
    return 1


def _oracle_answer(workspace: Path) -> None:
    x_star = GOLD_XS[0]
    payload = {
        "x_star": x_star,
        "score": oracle_score(x_star),
        "grid_best_x": grid_best_x(workspace),
    }
    (workspace / "answer.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )


def self_test() -> None:
    src = default_workspace()
    assert oracle_score(100.0) > oracle_score(52.0)
    assert oracle_score(50.0) > HOLDOUT_SCORE_FLOOR
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        for stray in list(fail_ws.rglob(".holdout_query_count")) + list(
            fail_ws.rglob(".holdout_probes.jsonl")
        ) + list(fail_ws.rglob(".holdout_confirm_done")):
            stray.unlink()
        assert evaluate(fail_ws) == 0
        nb = {"x_star": GRID_BEST, "score": NOTEBOOK_SCORE, "grid_best_x": GRID_BEST}
        (fail_ws / "answer.json").write_text(json.dumps(nb) + "\n", encoding="utf-8")
        assert evaluate(fail_ws) == 0
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py")],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "holdout_status=FAIL" in proc.stdout

        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py"), "--x", "100"],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "confirm grid winner" in proc.stdout
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py"), "--x", "45"],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "confirmatory probe" in proc.stdout
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py"), "--x", "50"],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "sealed_score=" in proc.stdout
        mid_score = float(proc.stdout.split("sealed_score=")[1].split()[0])
        assert mid_score > HOLDOUT_SCORE_FLOOR
        mid = {"x_star": 50.0, "score": mid_score, "grid_best_x": GRID_BEST}
        (fail_ws / "answer.json").write_text(json.dumps(mid) + "\n", encoding="utf-8")
        proc = subprocess.run(
            [sys.executable, str(fail_ws / "tools" / "holdout.py")],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert "holdout_status=PASS" in proc.stdout
        assert evaluate(fail_ws) == 0

        # Free sealed oracle must be disabled
        proc = subprocess.run(
            [
                sys.executable,
                "-c",
                "import tools.holdout as h; h.sealed_at(100)",
            ],
            cwd=fail_ws,
            capture_output=True,
            text=True,
        )
        assert proc.returncode != 0 or "disabled" in (proc.stderr + proc.stdout)

        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        for stray in list(pass_ws.rglob(".holdout_query_count")) + list(
            pass_ws.rglob(".holdout_probes.jsonl")
        ) + list(pass_ws.rglob(".holdout_confirm_done")):
            stray.unlink()
        proc = subprocess.run(
            [sys.executable, str(pass_ws / "tools" / "holdout.py"), "--x", "45"],
            cwd=pass_ws,
            capture_output=True,
            text=True,
        )
        assert "confirmatory probe" in proc.stdout
        proc = subprocess.run(
            [sys.executable, str(pass_ws / "tools" / "holdout.py"), "--x", "100"],
            cwd=pass_ws,
            capture_output=True,
            text=True,
        )
        assert "sealed_score=" in proc.stdout
        assert float(proc.stdout.split("sealed_score=")[1].split()[0]) > 2.0
        _oracle_answer(pass_ws)
        assert evaluate(pass_ws) == 1
        proc = subprocess.run(
            [sys.executable, str(pass_ws / "tools" / "holdout.py")],
            cwd=pass_ws,
            capture_output=True,
            text=True,
        )
        assert "holdout_status=PASS" in proc.stdout

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
