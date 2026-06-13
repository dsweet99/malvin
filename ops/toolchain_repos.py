"""Shared malvin/kiss repo root helpers for ops Modal scripts."""

from __future__ import annotations

import os
from pathlib import Path

import click


def malvin_repo_root() -> Path:
    """Return the malvin repository root (parent of ``ops/``)."""
    return Path(__file__).resolve().parent.parent


def kiss_repo_root() -> Path:
    """Return the kiss source tree (``KISS_REPO`` or sibling ``kiss`` repo)."""
    override = os.environ.get("KISS_REPO")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "kiss"


def validate_toolchain_repos() -> tuple[Path, Path]:
    """Ensure local malvin and kiss trees exist before building agent images."""
    malvin_repo = malvin_repo_root()
    kiss_repo = kiss_repo_root()
    if not (malvin_repo / "Cargo.toml").is_file():
        raise click.ClickException(f"malvin repo not found: {malvin_repo}")
    if not (kiss_repo / "Cargo.toml").is_file():
        raise click.ClickException(
            f"kiss repo not found: {kiss_repo} (set KISS_REPO to override)"
        )
    return malvin_repo, kiss_repo
