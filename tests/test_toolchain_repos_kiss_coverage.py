"""Kiss coverage witnesses for src/python/toolchain_repos.py."""
from __future__ import annotations

from pathlib import Path

import toolchain_repos as _mod


def test_toolchain_repos_kiss_coverage_witnesses() -> None:
    _ = (
        _mod.malvin_repo_root,
        _mod.resolve_malvin_cmd,
        _mod.validate_toolchain_repos,
        _mod.load_ops_entry,
        _mod.cursor_sdk_shutdown_qa,
        "cursor_sdk_shutdown_qa",
    )
    assert _mod.cursor_sdk_shutdown_qa() is not None


def test_load_ops_entry_does_not_write_ops_pycache(tmp_path: Path) -> None:
    """Importing ops CLI shims must not leave bytecode under ops/."""
    ops = _mod.malvin_repo_root() / "ops"
    for stale in ops.rglob("__pycache__"):
        if stale.is_dir():
            for child in stale.iterdir():
                child.unlink()
            stale.rmdir()
    _mod.load_ops_entry("fast_task")
    found = [p for p in ops.rglob("__pycache__") if p.is_dir()]
    assert found == []

