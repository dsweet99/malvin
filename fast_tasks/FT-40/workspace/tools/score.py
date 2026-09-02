#!/usr/bin/env python3
"""Trial-band score probe for FT-40 (interpolates data/grid.csv only)."""
from __future__ import annotations

import argparse
import csv
from pathlib import Path

X_LO = 25.0
X_HI = 65.0


def _grid_rows(root: Path) -> list[tuple[float, float]]:
    path = root / "data" / "grid.csv"
    rows: list[tuple[float, float]] = []
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            rows.append((float(row["x"]), float(row["score"])))
    rows.sort(key=lambda t: t[0])
    return rows


def score_at(root: Path, x: float) -> float:
    if not (X_LO <= x <= X_HI):
        raise SystemExit(
            f"score probe covers only the plant-trial band [{X_LO}, {X_HI}]; got x={x}"
        )
    rows = _grid_rows(root)
    if x <= rows[0][0]:
        return rows[0][1]
    if x >= rows[-1][0]:
        return rows[-1][1]
    for (x0, y0), (x1, y1) in zip(rows, rows[1:]):
        if x0 <= x <= x1:
            t = 0 if x1 == x0 else (x - x0) / (x1 - x0)
            return y0 + t * (y1 - y0)
    raise SystemExit("internal grid lookup failure")


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--x", type=float, required=True)
    args = p.parse_args()
    root = Path(__file__).resolve().parents[1]
    print(f"score={score_at(root, args.x):.10f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
