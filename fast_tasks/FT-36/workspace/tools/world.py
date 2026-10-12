#!/usr/bin/env python3
"""Intervention lab for FT-36. Outcomes are served from a sealed pack."""
from __future__ import annotations

import argparse
import base64
import csv
import sys
import zlib

# Sealed sample pack (zlib+base64). Not an analytics export.
_PACK = {
    'do_W_0': (
        "eNrNmV0KwzAMgy80xoTuf7fBoI+DJvkkd0+h7fLj2LIs6/35/V66Bv4/0MrAN57svVra"
        "2N7mqQnvGNNbH2vFLN5aS7Qn+OzIzR1SplPMW/ae+CzefRaDhnCjsKgga5gGNDyom8jv"
        "/C37DLopb8kdmbKYaD/EA4R6peK9i87mOVLhGCJR8H5IBlR09cKRFftXzuCF66YIp/P5"
        "Al/0EFepaqKAoqILKMf2QyUFqhTCYwcn0qKZXq4QM52pBcF7rkjXqN4imuQ4pu0oP5hl"
        "IM15CmahyNJDCCfO+V1UPHACnMspztNvF4Vx09Wo8hN6QoUoMCLTzCHHexXzXsVKV5x+"
        "51pRLtZxubrA+R7ZrDzr0R5iTj9s3rvy3quJnFIQgnISt4vQpDwI+/G9G+V1JLyto9EG"
        "RJHlfgFlB4fj"
    ),
    'do_W_1': (
        "eNrNmV0KwzAMgy80xoTuf7fBoI+DJvkkd0+h7fLj2LIs6/35/V66Bv4/0MrAN57svVra"
        "2N7mqQnvGNNbH2vFLN5aS7Qn+OzIzR1SplPMW/ae+CzefRaDhnCjsKgga5gGNDyom8jv"
        "/C37DLopb8kdmbKYaD/EA4R6peK9i87mOVLhGCJR8H5IBlR09cKRFftXzuCF66YIp/P5"
        "Al/0EFepaqKAoqILKMf2QyUFqhTCYwcn0qKZXq4QM52pBcF7rkjXqN4imuQ4pu0oP5hl"
        "IM15CmahyNJDCCfO+V1UPHACnMspztNvF4Vx09Wo8hN6QoUoMCLTzCHHexXzXsVKV5x+"
        "51pRLtZxubrA+R7ZrDzr0R5iTj9s3rvy3quJnFIQgnISt4vQpDwI+/G9G+V1JLyto9EG"
        "RJHlfgFlB4fj"
    ),
    'do_X_0': (
        "eNrNmVEKwzAMQy80xsS7/90Gg/2OrHly2q+SpE0b27Is5/n6XA++N1kYsRavTP1YHGnN"
        "ysjKVP556tqm/PMZ2IbbvEGycmwLnj0E3W3Y+y/dKNam9KHp2gsjQdM1JNF3xwZG9nBs"
        "c69enFIbSQ0lrBfqPrYZcfSzcI8DWOHQS6PYUYkNuey5lk4YkGhYL/1ZnmnxTAaJa2+x"
        "9dTmQWHHBTYP7xEhTjh/aoCWWgl81n90Go/NurFtgUSJdZ1kgO72rJMar6PPcq06JSeO"
        "l1p+7wUjtWJtkgNM2iv3UI2oCQu6RpSavtEr/8+WZpbWPZCgM+gt9LF3kmzrATJJrQfk"
        "WfqIPZAmerDDYFqnprwx2Cfirk1SbK2JwYbs2UZYatXNEXktfQxPv06h3w7WiTR9oaPX"
        "ZxxUM96xjIgR"
    ),
    'do_X_1': (
        "eNrNmVEKwzAMQy80xsS7/90Gg/2OrHly2q+SpE0b27Is5/n6XA++N1kYsRavTP1YHGnN"
        "ysjKVP556tqm/PMZ2IbbvEGycmwLnj0E3W3Y+y/dKNam9KHp2gsjQdM1JNF3xwZG9nBs"
        "c69enFIbSQ0lrBfqPrYZcfSzcI8DWOHQS6PYUYkNuey5lk4YkGhYL/1ZnmnxTAaJa2+x"
        "9dTmQWHHBTYP7xEhTjh/aoCWWgl81n90Go/NurFtgUSJdZ1kgO72rJMar6PPcq06JSeO"
        "l1p+7wUjtWJtkgNM2iv3UI2oCQu6RpSavtEr/8+WZpbWPZCgM+gt9LF3kmzrATJJrQfk"
        "WfqIPZAmerDDYFqnprwx2Cfirk1SbK2JwYbs2UZYatXNEXktfQxPv06h3w7WiTR9oaPX"
        "ZxxUM96xjIgR"
    ),
    'do_Z_0': (
        "eNrtxkEJAAAIBLBCIl7/coIxZHstPaciIiIiIiIiIiIiIiIiIiIiIvI2C5FghCc="
    ),
    'do_Z_1': (
        "eNrtxkEJAAAIBLBCIsL17yYYQ7bX0nMqIiIiIiIiIiIiIiIiIiIiIiJvswkOi/c="
    ),
    'observe': (
        "eNrNmUsKwzAQQy9USh+6/90Khay6si1pnFVIHMefkUYj6/35XS89N/w90Uqbvc/Z+pyz"
        "Dpf60covll6RH9heP0vjWepwbzG1tWKchbGrjWt9XECzz1QmTihs996N8tuNO54PAwAT"
        "h2MK9cPYUJEYcaMbN1XaZ+FKf3KzlitFFtDEBE4xba7cYMRN3QUI2/MpRfmkGGDtkYmp"
        "mnBlYdx0oZjCJ59TZAJ+Djs53Yt7zLlykpgELSwCd4gl5TVkzm/RrRNsWhb2umCE8HO8"
        "kat8cXtNudzdrPFz2UHFmrEQ/IrFhr3yzdnXirVhonBWTOAxWqQXPFhGnfac42p3m1UU"
        "ycS2KWd0EBOKOUObCZVyf4qkeFJTsHAL6Wa22iJmYshNjIpBOJfNKZ7dXHLsRdHmpXiK"
        "V/TivuNXh9s="
    ),
}


def _ys(key: str, n: int) -> list[float]:
    raw = zlib.decompress(base64.b64decode("".join(_PACK[key])))
    vals = [float(x) for x in raw.decode().split(",") if x]
    if len(vals) < n:
        raise SystemExit(f"pack {key} has only {len(vals)} rows; need {n}")
    return vals[:n]


def _print_y(ys: list[float]) -> None:
    writer = csv.DictWriter(sys.stdout, fieldnames=["Y"])
    writer.writeheader()
    for y in ys:
        writer.writerow({"Y": f"{y:.6f}"})
    print(f"# n={len(ys)} mean_Y={sum(ys)/len(ys):.6f}", file=sys.stderr)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="FT-36 world simulator")
    sub = p.add_subparsers(dest="cmd", required=True)
    po = sub.add_parser("observe", help="observational Y sample (no knobs revealed)")
    po.add_argument("--n", type=int, default=200)
    pd = sub.add_parser("do", help="force one knob, then sample Y")
    pd.add_argument("--var", required=True, choices=["X", "Z", "W", "x", "z", "w"])
    pd.add_argument("--value", type=int, required=True, choices=[0, 1])
    pd.add_argument("--n", type=int, default=200)
    args = p.parse_args(argv)
    if args.cmd == "observe":
        _print_y(_ys("observe", args.n))
        return 0
    var = args.var.upper()
    key = f"do_{var}_{args.value}"
    if key not in _PACK:
        print(f"missing pack {key}", file=sys.stderr)
        return 2
    _print_y(_ys(key, args.n))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
