#!/usr/bin/env python3
"""Coarse holdout for FT-40: finiteness / mid-range sanity only (no sealed table)."""
from __future__ import annotations

import argparse
import csv
import importlib.util
import math
from pathlib import Path


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


def holdout_pass(workspace: Path) -> bool:
    """PASS if predict is finite on mid-range CSV and a few off-band probes.

    Necessary for the ticket workflow, not proof of the hidden sealed law.
    """
    predict = _load_predict(workspace)
    path = workspace / "data" / "midrange.csv"
    try:
        with path.open(encoding="utf-8") as f:
            for row in csv.DictReader(f):
                y = float(predict(float(row["x"])))
                if not math.isfinite(y):
                    return False
        for x in (0.05, 1.0, 10.0, 100.0):
            y = float(predict(x))
            if not math.isfinite(y):
                return False
            # Reject clearly explosive large-x linear blow-ups.
            if x >= 100.0 and abs(y) > 50.0:
                return False
    except Exception:
        return False
    return True


def main() -> int:
    p = argparse.ArgumentParser(description="Sealed holdout probe for FT-40")
    p.add_argument("--x", type=float, default=None, help=argparse.SUPPRESS)
    args = p.parse_args()
    root = Path(__file__).resolve().parents[1]
    if args.x is not None:
        print("holdout_status=FAIL")
        print("note=sealed point probes are not available on this ticket")
        return 0
    ok = holdout_pass(root)
    print("holdout_status=PASS" if ok else "holdout_status=FAIL")
    if ok:
        print("note=mid-range/finiteness check only; not a global-law proof")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
