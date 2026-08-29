#!/usr/bin/env python3
"""Public consistency check for FT-39 (necessary, not sufficient)."""
from __future__ import annotations

import csv
import importlib.util
import json
import sys
from pathlib import Path

NOTEBOOK_A = 1.553973
NOTEBOOK_B = 0.148807
NOTEBOOK_MID_SSE = 0.000217


def mid_rows(root: Path) -> list[tuple[float, float]]:
    path = root / "data" / "midrange.csv"
    rows: list[tuple[float, float]] = []
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            rows.append((float(row["x"]), float(row["y"])))
    return rows


def load_predict(root: Path):
    path = root / "predict.py"
    if not path.is_file():
        raise FileNotFoundError("predict.py")
    spec = importlib.util.spec_from_file_location("ft39_predict_pub", path)
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
    if set(data.keys()) != {"mid_sse"}:
        print("FAIL: answer.json keys must be exactly mid_sse", file=sys.stderr)
        return 1
    try:
        mid = float(data["mid_sse"])
    except (TypeError, ValueError):
        print("FAIL: mid_sse must be numeric", file=sys.stderr)
        return 1
    if isinstance(data["mid_sse"], bool):
        print("FAIL: mid_sse must be numeric, not bool", file=sys.stderr)
        return 1

    try:
        predict = load_predict(root)
    except Exception as exc:  # noqa: BLE001 — public check reports load errors
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    rows = mid_rows(root)
    expected = mid_sse(predict, rows)
    if abs(mid - expected) > 1e-6:
        print(
            f"FAIL: mid_sse={mid} != recomputed {expected} for predict.py",
            file=sys.stderr,
        )
        return 1

    # Notebook-consistent: linear starter coefficients within tight tol on band.
    notebook_preds = [NOTEBOOK_A + NOTEBOOK_B * x for x, _ in rows]
    actual_preds = [float(predict(x)) for x, _ in rows]
    notebook_ok = (
        abs(mid - NOTEBOOK_MID_SSE) <= 1e-6
        and all(abs(a - b) <= 1e-9 for a, b in zip(actual_preds, notebook_preds))
    )
    if notebook_ok:
        print("PASS (notebook-consistent)")
        return 0
    print("PASS (schema + mid_sse)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
