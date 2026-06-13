"""Pytest path setup for ops integration self-tests."""

from __future__ import annotations

import sys
from pathlib import Path

OPS_DIR = Path(__file__).resolve().parent.parent / "ops"
if str(OPS_DIR) not in sys.path:
    sys.path.insert(0, str(OPS_DIR))
