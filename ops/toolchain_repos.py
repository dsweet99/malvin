"""Shared malvin repo root helpers for ops Modal scripts."""

from __future__ import annotations

import os
import shutil
from pathlib import Path

import click


def malvin_repo_root() -> Path:
    """Return the malvin repository root (parent of ``ops/``)."""
    return Path(__file__).resolve().parent.parent


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
