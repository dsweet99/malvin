#!/usr/bin/env python3
"""Sealed score probe for FT-40. Confirm + 1 discretionary probe."""
from __future__ import annotations

import argparse
import json
import math
import struct
from pathlib import Path

X_LO = 0.0
X_HI = 100.0
CONFIRM_X = 45.0
QUERY_BUDGET = 1
_COUNT_NAME = ".holdout_query_count"
_CONFIRM_NAME = ".holdout_confirm_done"
_PROBE_LOG = ".holdout_probes.jsonl"
_KEY = b"ft40-seal-v44"
_BLOB = bytes.fromhex("001252564b15b35e6c2d763434e63274302d7365616c3136343466743430d94c65616c2d76347e267434302d7365712cb7efadadffed35702d7365616c2d2f7434667434302d7725")
_ALLOW_RAW = False


def _count_path(root: Path) -> Path:
    return root / _COUNT_NAME


def _confirm_path(root: Path) -> Path:
    return root / _CONFIRM_NAME


def _probe_log_path(root: Path) -> Path:
    return root / _PROBE_LOG


def _queries_used(root: Path) -> int:
    path = _count_path(root)
    if not path.is_file():
        return 0
    try:
        return int(path.read_text(encoding="utf-8").strip() or "0")
    except ValueError:
        return 0


def _bump_query(root: Path) -> int:
    used = _queries_used(root) + 1
    _count_path(root).write_text(f"{used}\n", encoding="utf-8")
    return used


def _confirm_done(root: Path) -> bool:
    return _confirm_path(root).is_file()


def _mark_confirm(root: Path) -> None:
    _confirm_path(root).write_text("1\n", encoding="utf-8")


def _params() -> tuple[float, ...]:
    raw = bytes(b ^ _KEY[i % len(_KEY)] for i, b in enumerate(_BLOB))
    return struct.unpack("9d", raw)


def _raw_sealed(x: float) -> float:
    if not _ALLOW_RAW:
        raise SystemExit(
            "direct sealed evaluation is disabled; use: python tools/holdout.py --x <float>"
        )
    if not (X_LO <= x <= X_HI):
        raise SystemExit(f"x out of range [{X_LO}, {X_HI}]; got {x}")
    a1, c1, w1, a2, c2, w2, a3, c3, w3 = _params()
    return (
        a1 * math.exp(-(((x - c1) / w1) ** 2))
        + a2 * math.exp(-(((x - c2) / w2) ** 2))
        + a3 * math.exp(-(((x - c3) / w3) ** 2))
    )


def _log_probe(root: Path, x: float, y: float) -> None:
    with _probe_log_path(root).open("a", encoding="utf-8") as f:
        f.write(json.dumps({"x": x, "sealed_score": y}) + "\n")


def _load_probes(root: Path) -> list[dict]:
    path = _probe_log_path(root)
    if not path.is_file():
        return []
    out = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return out


def measure(root: Path, x: float) -> float:
    """Only legal sealed measurement. Enforces confirm + discretionary budget."""
    global _ALLOW_RAW
    x = float(x)
    if not _confirm_done(root):
        if abs(x - CONFIRM_X) > 1e-9:
            raise SystemExit(
                f"confirm grid winner at x={CONFIRM_X:g} first "
                "(free; does not consume discretionary budget)"
            )
        _ALLOW_RAW = True
        try:
            y = _raw_sealed(x)
        finally:
            _ALLOW_RAW = False
        _log_probe(root, x, y)
        _mark_confirm(root)
        return y
    if _queries_used(root) >= QUERY_BUDGET:
        raise SystemExit(f"discretionary query budget exhausted ({QUERY_BUDGET})")
    _ALLOW_RAW = True
    try:
        y = _raw_sealed(x)
    finally:
        _ALLOW_RAW = False
    _bump_query(root)
    _log_probe(root, x, y)
    return y


def sealed_at(*_a, **_k):
    raise SystemExit(
        "direct sealed evaluation is disabled; use: python tools/holdout.py --x <float>"
    )


_sealed_at = sealed_at


def check_answer(root: Path) -> bool:
    path = root / "answer.json"
    if not path.is_file():
        return False
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    if not isinstance(data, dict) or set(data.keys()) != {"x_star", "score", "grid_best_x"}:
        return False
    try:
        x_star = float(data["x_star"])
        score = float(data["score"])
        float(data["grid_best_x"])
    except (TypeError, ValueError, KeyError):
        return False
    if score <= 1.0 + 1e-9:
        return False
    for p in _load_probes(root):
        try:
            px = float(p["x"])
            py = float(p["sealed_score"])
        except (KeyError, TypeError, ValueError):
            continue
        if abs(px - x_star) <= 1e-9 and abs(py - score) <= 1e-3:
            return True
    return False


def main() -> int:
    p = argparse.ArgumentParser(description="Sealed holdout probe for FT-40")
    p.add_argument("--x", type=float, default=None)
    args = p.parse_args()
    root = Path(__file__).resolve().parents[1]
    if args.x is not None:
        try:
            y = measure(root, float(args.x))
        except SystemExit as exc:
            print("holdout_status=FAIL")
            print(f"note={exc}")
            return 0
        used = _queries_used(root)
        if used == 0:
            print(f"sealed_score={y:.10f}")
            print("queries_used=0")
            print(f"queries_remaining={QUERY_BUDGET}")
            print("note=confirmatory probe (free)")
        else:
            print(f"sealed_score={y:.10f}")
            print(f"queries_used={used}")
            print(f"queries_remaining={QUERY_BUDGET - used}")
        return 0
    ok = check_answer(root)
    print("holdout_status=PASS" if ok else "holdout_status=FAIL")
    if not ok:
        print(
            "note=answer must match a prior --x probe at x_star with "
            "score > 1.0 (confirm grid winner first, then one discretionary probe)"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
