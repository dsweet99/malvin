"""Host-stable cached venvs for sandbox_prep unit tests."""

from __future__ import annotations

import fcntl
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

_VENV_CACHE_ROOT: Path | None = None
_VENV_CACHE: dict[tuple[str, ...], Path] = {}
VENV_CACHE_OFFLINE = False

def venv_cache_key_token(packages: tuple[str, ...]) -> str:
    """Stable id for cache dirs (``hash()`` is process-salted)."""
    return hashlib.sha256("\0".join(packages).encode()).hexdigest()[:16]

def venv_cache_root() -> Path:
    """Host-stable cache so kiss per-test workers reuse warm venvs."""
    global _VENV_CACHE_ROOT
    if _VENV_CACHE_ROOT is None:
        root = Path(tempfile.gettempdir()) / f"malvin-venv-cache-{os.getuid()}"
        root.mkdir(mode=0o700, parents=True, exist_ok=True)
        _VENV_CACHE_ROOT = root
    return _VENV_CACHE_ROOT

def minimal_venv_dir(dest: Path) -> Path:
    """Create a venv-shaped directory with ``bin/python`` → sys.executable."""
    dest.mkdir(parents=True, exist_ok=True)
    bin_dir = dest / "bin"
    bin_dir.mkdir(exist_ok=True)
    python = bin_dir / "python"
    if not python.exists():
        python.symlink_to(sys.executable)
    return dest

def _venv_python_ready(base: Path) -> bool:
    return (base / "bin" / "python").is_file()

def _create_base_venv(base: Path) -> bool:
    """Create ``base`` via ``python -m venv``. Return True on success."""
    created = subprocess.run(
        [sys.executable, "-m", "venv", "--system-site-packages", str(base)],
        check=False,
        capture_output=True,
        text=True,
    )
    return created.returncode == 0

def _pip_install_into(base: Path, packages: tuple[str, ...]) -> bool:
    pip = str(base / "bin" / "pip")
    install = subprocess.run(
        [pip, "install", "--no-cache-dir", *packages],
        capture_output=True,
        text=True,
        check=False,
    )
    return install.returncode == 0

def _ensure_base_locked(base: Path, packages: tuple[str, ...]) -> None:
    global VENV_CACHE_OFFLINE
    if _venv_python_ready(base):
        return
    if not _create_base_venv(base):
        VENV_CACHE_OFFLINE = True
        if base.exists():
            shutil.rmtree(base)
        minimal_venv_dir(base)
        return
    if packages and not _pip_install_into(base, packages):
        VENV_CACHE_OFFLINE = True

def _resolve_base(packages: tuple[str, ...]) -> Path:
    global VENV_CACHE_OFFLINE
    token = venv_cache_key_token(packages)
    base = venv_cache_root() / f"base-{token}"
    if packages in _VENV_CACHE:
        return _VENV_CACHE[packages]
    if VENV_CACHE_OFFLINE and not _venv_python_ready(base):
        minimal_venv_dir(base)
        _VENV_CACHE[packages] = base
        return base
    lock_path = venv_cache_root() / f".lock-{token}"
    with open(lock_path, "a", encoding="utf-8") as lock_f:
        fcntl.flock(lock_f.fileno(), fcntl.LOCK_EX)
        _ensure_base_locked(base, packages)
    _VENV_CACHE[packages] = base
    return base

def clone_cached_venv(dest: Path, packages: tuple[str, ...] = ()) -> Path:
    """Copy a host-cached venv (optionally with pip packages) into ``dest``.

    Creating a venv + pip install is multi-second; copytree of a warm cache is
    ~0.2s and keeps unit tests under the 1.5s budget.
    """
    base = _resolve_base(packages)
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(base, dest, symlinks=True)
    return dest
