from __future__ import annotations

import os
import subprocess
from pathlib import Path


def _malvin_bin() -> Path:
    env = os.environ.get("MALVIN_BIN")
    if env:
        return Path(env)
    root = Path(__file__).resolve().parents[1]
    debug = root / "target" / "debug" / "malvin"
    release = root / "target" / "release" / "malvin"
    if debug.is_file():
        return debug
    if release.is_file():
        return release
    raise FileNotFoundError(
        "malvin binary not found; run `cargo build` or set MALVIN_BIN to the executable path"
    )


def test_malvin_help_exits_zero() -> None:
    p = subprocess.run(
        [_malvin_bin(), "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert p.returncode == 0
    assert "malvin" in (p.stdout or "").lower()


def test_malvin_version_flag() -> None:
    p = subprocess.run(
        [_malvin_bin(), "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert p.returncode == 0
