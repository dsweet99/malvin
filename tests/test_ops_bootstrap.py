"""Coverage for ``src/python/_ops_bootstrap.py`` path + library loading."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType

import pytest

_ROOT = Path(__file__).resolve().parents[1]
_OPS = _ROOT / "ops"
_BOOTSTRAP_PATH = _ROOT / "src" / "python" / "_ops_bootstrap.py"


def _load_bootstrap() -> ModuleType:
    spec = importlib.util.spec_from_file_location("_ops_bootstrap", _BOOTSTRAP_PATH)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_ops_bootstrap"] = mod
    spec.loader.exec_module(mod)
    return mod


def test_ensure_src_python_path_prefers_library() -> None:
    boot = _load_bootstrap()
    ops = str(_OPS)
    if ops not in sys.path:
        sys.path.insert(0, ops)
    root = boot.ensure_src_python_path()
    assert root == _ROOT / "src" / "python"
    assert str(root) == sys.path[0]
    assert ops not in sys.path or sys.path.index(str(root)) < sys.path.index(ops)


def test_load_library_returns_src_python_module() -> None:
    boot = _load_bootstrap()
    mod = boot.load_library("toolchain_repos")
    assert "src/python" in Path(mod.__file__).as_posix()
    assert (mod.malvin_repo_root() / "Cargo.toml").is_file()
    again = boot.load_library("toolchain_repos")
    assert again is mod


def test_load_library_missing_raises() -> None:
    boot = _load_bootstrap()
    with pytest.raises(ImportError, match="library module missing"):
        boot.load_library("definitely_not_a_malvin_module_zz")


def test_load_library_replaces_ops_shim_partial() -> None:
    boot = _load_bootstrap()
    fake = ModuleType("tox_gates")
    fake.__file__ = str(_OPS / "tox_gates.py")
    sys.modules["tox_gates"] = fake
    mod = boot.load_library("tox_gates")
    assert "src/python" in Path(mod.__file__).as_posix()
    assert mod is not fake
