#!/usr/bin/env python3
"""Modal entry for diagnose_cidr_observed_modal — implementation in ``src/python/diagnose_cidr_observed_modal.py``."""

from __future__ import annotations

from _ops_bootstrap import load_library

_lib = load_library("diagnose_cidr_observed_modal")
app = _lib.app


@app.local_entrypoint(name="diagnose_cidr_observed")
def diagnose_cidr_observed_main() -> None:
    _lib.run_diagnose_cidr_observed()


main = diagnose_cidr_observed_main

__all__ = ["app", "main", "diagnose_cidr_observed_main"]

if __name__ == "__main__":
    main()
