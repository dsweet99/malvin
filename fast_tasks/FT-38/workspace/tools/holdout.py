#!/usr/bin/env python3
"""Sealed holdout-loss probe for FT-38. Outcomes come from a packed table."""
from __future__ import annotations

import argparse
import base64
import math
import zlib

N = 501
X_LO = 1e-3
X_HI = 1.0
_PACK = (
    "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"
    "ax7VRBl9sB89oFmAdBPiWv8BxWY/mQ=="
)


def _ys() -> list[float]:
    raw = zlib.decompress(base64.b64decode("".join(_PACK))).decode()
    vals = [float(x) for x in raw.split(",") if x]
    if len(vals) != N:
        raise SystemExit(f"pack has {len(vals)} rows; need {N}")
    return vals


def _x_at(i: int) -> float:
    return 10 ** (math.log10(X_LO) + i * (math.log10(X_HI) - math.log10(X_LO)) / (N - 1))


def holdout_loss(lr: float) -> float:
    if not (X_LO <= lr <= X_HI):
        raise SystemExit(f"lr must be in [{X_LO}, {X_HI}], got {lr}")
    ys = _ys()
    t = (math.log10(lr) - math.log10(X_LO)) / (math.log10(X_HI) - math.log10(X_LO))
    pos = t * (N - 1)
    i0 = int(math.floor(pos))
    i1 = min(i0 + 1, N - 1)
    if i0 == i1:
        return ys[i0]
    x0, x1 = _x_at(i0), _x_at(i1)
    w = (math.log10(lr) - math.log10(x0)) / (math.log10(x1) - math.log10(x0))
    return ys[i0] + w * (ys[i1] - ys[i0])


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="FT-38 sealed holdout probe")
    p.add_argument("--lr", type=float, required=True, help="learning rate in [0.001, 1]")
    args = p.parse_args(argv)
    loss = holdout_loss(args.lr)
    print(f"lr={args.lr:.8g}")
    print(f"holdout_loss={loss:.8f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
