#!/usr/bin/env python3
"""Modal entry for diagnose_cidr_dns_modal — implementation in ``src/python/diagnose_cidr_dns_modal.py``."""

from __future__ import annotations

import sys
from pathlib import Path

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("diagnose_cidr_dns_modal")
app = _lib.app


@app.local_entrypoint(name="diagnose_cidr_dns")
def diagnose_cidr_dns_main() -> None:
    _lib.run_diagnose_cidr_dns()


main = diagnose_cidr_dns_main

__all__ = ["app", "main", "diagnose_cidr_dns_main"]

if __name__ == "__main__":
    main()
