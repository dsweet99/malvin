"""Shared malvin repo root helpers for ops Modal scripts."""

from __future__ import annotations

import importlib.util
import os
import shutil
import sys
from pathlib import Path
from types import ModuleType

import click


def malvin_repo_root() -> Path:
    """Return the malvin repository root (directory with ``Cargo.toml`` + ``ops/``).

    Works when this module lives under ``ops/`` or ``src/python/``.
    """
    here = Path(__file__).resolve()
    for parent in (here.parent, *here.parents):
        if (
            (parent / "Cargo.toml").is_file()
            and (parent / "VISION.md").is_file()
            and (parent / "ops").is_dir()
        ):
            return parent
    raise RuntimeError(f"malvin repo root not found from {here}")


def load_ops_entry(modname: str) -> ModuleType:
    """Load ``ops/<modname>.py`` under a unique name (CLI surface for CliRunner).

    Library modules in ``src/python`` must not define Click commands; self-tests
    that exercise the CLI load the thin ops entry via this helper.
    """
    root = malvin_repo_root()
    ops_dir = root / "ops"
    boot_path = root / "src" / "python" / "_ops_bootstrap.py"
    entry_path = ops_dir / f"{modname}.py"
    if not entry_path.is_file():
        raise ImportError(f"ops entry missing: {entry_path}")

    # When ``python ops/<mod>.py`` is already running, reuse ``__main__`` so we
    # do not re-register Modal local_entrypoint handlers on the shared App.
    main_mod = sys.modules.get("__main__")
    if main_mod is not None:
        main_file = getattr(main_mod, "__file__", None)
        if main_file and Path(main_file).resolve() == entry_path.resolve():
            return main_mod

    if "_ops_bootstrap" not in sys.modules:
        boot_spec = importlib.util.spec_from_file_location("_ops_bootstrap", boot_path)
        if boot_spec is None or boot_spec.loader is None:
            raise ImportError(f"cannot load bootstrap: {boot_path}")
        boot_mod = importlib.util.module_from_spec(boot_spec)
        sys.modules["_ops_bootstrap"] = boot_mod
        boot_spec.loader.exec_module(boot_mod)

    unique = f"_malvin_ops_{modname}"
    existing = sys.modules.get(unique)
    if existing is not None:
        return existing

    spec = importlib.util.spec_from_file_location(unique, entry_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load ops entry: {entry_path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[unique] = mod
    # Keep ops/ free of regenerable bytecode; ops holds CLI scripts only.
    prev_dwb = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(mod)
    finally:
        sys.dont_write_bytecode = prev_dwb
    return mod


def resolve_malvin_cmd() -> str:
    """Return malvin executable: ``MALVIN`` env, repo target build, then PATH."""
    override = os.environ.get("MALVIN")
    if override:
        return override
    root = malvin_repo_root()
    for rel in ("target/debug/malvin", "target/release/malvin"):
        candidate = root / rel
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    on_path = shutil.which("malvin")
    return on_path if on_path else "malvin"


def validate_toolchain_repos() -> Path:
    """Ensure the local malvin tree exists before building agent images."""
    malvin_repo = malvin_repo_root()
    if not (malvin_repo / "Cargo.toml").is_file():
        raise click.ClickException(f"malvin repo not found: {malvin_repo}")
    return malvin_repo
