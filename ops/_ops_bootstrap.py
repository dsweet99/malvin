#!/usr/bin/env python3
"""Bootstrap ``src/python`` onto ``sys.path`` for thin ``ops/`` entry scripts.

Python puts the script directory (``ops/``) at ``sys.path[0]`` for
``python ops/foo.py``. That would make ``import foo`` load the shim itself.
We strip ``ops/`` from ``sys.path`` and insert ``src/python`` first so flat
library imports resolve to the implementation modules.

When Modal (or ``importlib``) loads ``ops/foo.py`` under the basename
``foo``, a normal ``from foo import …`` would circular-import the partial
shim. ``load_library`` clears that partial and loads ``src/python/foo.py``.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType

_OPS_DIR = Path(__file__).resolve().parent
_SRC_PYTHON = _OPS_DIR.parent / "src" / "python"


def ensure_src_python_path() -> Path:
    """Prefer ``src/python`` over ``ops/`` for flat library imports."""
    ops_resolved = _OPS_DIR.resolve()
    sys.path[:] = [p for p in sys.path if Path(p).resolve() != ops_resolved]
    root = str(_SRC_PYTHON)
    if root in sys.path:
        sys.path.remove(root)
    sys.path.insert(0, root)
    return _SRC_PYTHON


def load_library(modname: str) -> ModuleType:
    """Load ``src/python/<modname>.py``, replacing any ops shim in ``sys.modules``."""
    ensure_src_python_path()
    lib_path = _SRC_PYTHON / f"{modname}.py"
    if not lib_path.is_file():
        raise ImportError(f"library module missing: {lib_path}")

    existing = sys.modules.get(modname)
    if existing is not None:
        existing_file = getattr(existing, "__file__", None)
        if existing_file and Path(existing_file).resolve().parent == _OPS_DIR.resolve():
            del sys.modules[modname]
        elif existing_file and Path(existing_file).resolve() == lib_path.resolve():
            return existing

    # Import under the canonical flat name from src/python (now first on path).
    return importlib.import_module(modname)
