"""Shared malvin/kiss repo root helpers for ops Modal scripts."""

from __future__ import annotations

import os
from pathlib import Path

import click

KISS_CRATE = "kiss-ai"
KISS_STABLE_VERSION = "0.4.8"


def kiss_cargo_install_command() -> str:
    """Install the pinned stable kiss release from crates.io."""
    return f"cargo install {KISS_CRATE} --version {KISS_STABLE_VERSION} --locked"


def malvin_repo_root() -> Path:
    """Return the malvin repository root (parent of ``ops/``)."""
    return Path(__file__).resolve().parent.parent


def kiss_repo_root() -> Path:
    """Return the kiss source tree (``KISS_REPO`` or sibling ``kiss`` repo)."""
    override = os.environ.get("KISS_REPO")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "kiss"


def validate_toolchain_repos() -> Path:
    """Ensure the local malvin tree exists before building agent images."""
    malvin_repo = malvin_repo_root()
    if not (malvin_repo / "Cargo.toml").is_file():
        raise click.ClickException(f"malvin repo not found: {malvin_repo}")
    return malvin_repo
