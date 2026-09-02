#!/usr/bin/env python3
"""Sealed holdout acceptance probe for FT-39."""
from __future__ import annotations

import importlib.util
from pathlib import Path


def _load_predict(workspace: Path):
    path = workspace / "predict.py"
    if not path.is_file():
        raise SystemExit("missing predict.py")
    spec = importlib.util.spec_from_file_location("ft39_predict", path)
    if spec is None or spec.loader is None:
        raise SystemExit("cannot load predict.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, "predict", None)
    if not callable(fn):
        raise SystemExit("predict.py must define callable predict(x)")
    return fn


def holdout_pass(workspace: Path) -> bool:
    """Asymptote gate: near-origin floor and finite large-x capacity.

    Passing is necessary for the ticket workflow, not proof of the hidden law.
    """
    predict = _load_predict(workspace)
    try:
        y0 = float(predict(1e-4))
        y_mid = float(predict(3.0))
        y_hi = float(predict(100.0))
        y_inf = float(predict(1e6))
    except Exception:
        return False

    if not (0.97 <= y0 <= 1.03):
        return False
    if not (2.90 <= y_inf <= 3.10):
        return False
    # Capacity must already be visible by x=100 (rejects late hard clips).
    if abs(y_inf - y_hi) > 0.12:
        return False
    if not (y0 < y_mid < y_inf):
        return False
    return True


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    ok = holdout_pass(root)
    print("holdout_status=PASS" if ok else "holdout_status=FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
