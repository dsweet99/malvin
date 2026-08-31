#!/usr/bin/env python3
"""Public consistency check for FT-40 (necessary, not sufficient)."""
from __future__ import annotations

import csv
import json
import math
import sys
from pathlib import Path

NOTEBOOK_X = 45.0
NOTEBOOK_SCORE = 1.0


def grid_best_x(root: Path) -> float:
    path = root / "data" / "grid.csv"
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


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    ans_path = root / "answer.json"
    if not ans_path.is_file():
        print("FAIL: missing answer.json", file=sys.stderr)
        return 1
    try:
        data = json.loads(ans_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"FAIL: bad JSON: {exc}", file=sys.stderr)
        return 1
    if set(data.keys()) != {"x_star", "score", "grid_best_x"}:
        print("FAIL: answer.json keys must be exactly x_star, score, grid_best_x", file=sys.stderr)
        return 1
    try:
        x_star = float(data["x_star"])
        score = float(data["score"])
        grid_best = float(data["grid_best_x"])
    except (TypeError, ValueError):
        print("FAIL: fields must be numeric", file=sys.stderr)
        return 1
    for key in ("x_star", "score", "grid_best_x"):
        if isinstance(data[key], bool):
            print(f"FAIL: {key} must be numeric, not bool", file=sys.stderr)
            return 1
    expected = grid_best_x(root)
    if not math.isclose(grid_best, expected, abs_tol=1e-9):
        print(f"FAIL: grid_best_x={grid_best} != {expected}", file=sys.stderr)
        return 1
    notebook_ok = (
        math.isclose(x_star, NOTEBOOK_X, abs_tol=1e-9)
        and math.isclose(score, NOTEBOOK_SCORE, abs_tol=1e-6)
        and math.isclose(grid_best, NOTEBOOK_X, abs_tol=1e-9)
    )
    print("PASS (notebook-consistent)" if notebook_ok else "PASS (schema + grid_best_x)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
