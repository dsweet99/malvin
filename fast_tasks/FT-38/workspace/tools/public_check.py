#!/usr/bin/env python3
"""Public consistency check for FT-38 (necessary, not sufficient)."""
from __future__ import annotations

import csv
import json
import sys
from pathlib import Path


def grid_best(path: Path) -> tuple[float, float]:
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
    return best_lr, best_loss


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
    if set(data.keys()) != {"lr_star", "holdout_loss", "grid_best_lr"}:
        print("FAIL: answer.json keys must be exactly lr_star, holdout_loss, grid_best_lr", file=sys.stderr)
        return 1
    try:
        lr_star = float(data["lr_star"])
        holdout = float(data["holdout_loss"])
        grid_lr = float(data["grid_best_lr"])
    except (TypeError, ValueError):
        print("FAIL: fields must be numeric", file=sys.stderr)
        return 1
    gb_lr, gb_loss = grid_best(root / "data" / "grid.csv")
    if abs(grid_lr - gb_lr) > 1e-9:
        print(f"FAIL: grid_best_lr={grid_lr} != csv argmin {gb_lr}", file=sys.stderr)
        return 1
    if not (0.001 <= lr_star <= 1.0):
        print("FAIL: lr_star out of range", file=sys.stderr)
        return 1
    # Notebook triple is accepted: shipping the grid winner with train loss as holdout.
    notebook_ok = (
        abs(lr_star - gb_lr) <= 1e-12
        and abs(holdout - gb_loss) <= 1e-6
    )
    # Also accept any in-range lr_star if holdout looks like a real float (schema-level).
    # Public check is deliberately weak on optimality — hidden grading is stricter.
    if notebook_ok:
        print("PASS (notebook-consistent)")
        return 0
    print("PASS (schema + grid_best)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
