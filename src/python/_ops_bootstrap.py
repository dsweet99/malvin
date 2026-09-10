
from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType

_SRC_PYTHON = Path(__file__).resolve().parent
_OPS_DIR = _SRC_PYTHON.parent.parent / "ops"

def ensure_src_python_path() -> Path:
    ops_resolved = _OPS_DIR.resolve()
    sys.path[:] = [p for p in sys.path if Path(p).resolve() != ops_resolved]
    root = str(_SRC_PYTHON)
    if root in sys.path:
        sys.path.remove(root)
    sys.path.insert(0, root)
    return _SRC_PYTHON

def load_library(modname: str) -> ModuleType:
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

    
    return importlib.import_module(modname)
