#!/usr/bin/env python3
"""Public consistency check for FT-40 (necessary, not sufficient)."""
from __future__ import annotations

import csv
import importlib.util
import json
import math
import sys
from pathlib import Path

NOTEBOOK_MID_SSE = 0.000421955831


def _load_predict(workspace: Path):
    path = workspace / "predict.py"
    if not path.is_file():
        raise SystemExit("missing predict.py")
    spec = importlib.util.spec_from_file_location("ft40_predict", path)
    if spec is None or spec.loader is None:
        raise SystemExit("cannot load predict.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, "predict", None)
    if not callable(fn):
        raise SystemExit("predict.py must define callable predict(x)")
    return fn


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
        claimed = float(data["mid_sse"])
    except (TypeError, ValueError):
        print("FAIL: mid_sse must be numeric", file=sys.stderr)
        return 1
    if isinstance(data["mid_sse"], bool):
        print("FAIL: mid_sse must be numeric, not bool", file=sys.stderr)
        return 1

    predict = _load_predict(root)
    actual = mid_sse(root, predict)
    if abs(claimed - actual) > 1e-9:
        print(
            f"FAIL: mid_sse={claimed} != recomputed {actual}",
            file=sys.stderr,
        )
        return 1

    if math.isclose(claimed, NOTEBOOK_MID_SSE, abs_tol=5e-6):
        print("PASS (notebook-consistent)")
        return 0
    print("PASS (schema + mid_sse match)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
