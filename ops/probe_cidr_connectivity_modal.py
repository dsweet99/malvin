#!/usr/bin/env python3
"""Modal entry for probe_cidr_connectivity_modal — implementation in ``src/python/probe_cidr_connectivity_modal.py``."""

from __future__ import annotations

import sys
from pathlib import Path

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("probe_cidr_connectivity_modal")
app = _lib.app


@app.local_entrypoint(name="probe_cidr_connectivity")
def probe_cidr_connectivity_main() -> None:
    _lib.run_probe_cidr_connectivity()


main = probe_cidr_connectivity_main

__all__ = ["app", "main", "probe_cidr_connectivity_main"]

if __name__ == "__main__":
    main()
