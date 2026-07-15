"""Shared pytest configuration for malvin unit tests."""

from __future__ import annotations

import os
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

_REPO = Path(__file__).resolve().parents[1]
_SRC_PYTHON = _REPO / "src" / "python"
if str(_SRC_PYTHON) not in sys.path:
    sys.path.insert(0, str(_SRC_PYTHON))


def _modal_credentials_configured() -> bool:
    if os.environ.get("MODAL_TOKEN_ID") and os.environ.get("MODAL_TOKEN_SECRET"):
        return True
    try:
        import modal.config as modal_config

        cfg = modal_config.config
        return bool(cfg.get("token_id") and cfg.get("token_secret"))
    except Exception:
        return False


def pytest_sessionstart(session: pytest.Session) -> None:
    """Stub Modal app lookup when host credentials are absent."""
    if _modal_credentials_configured():
        return
    import modal

    session._modal_lookup_patcher = patch.object(  # type: ignore[attr-defined]
        modal.App, "lookup", return_value=MagicMock(name="modal_app")
    )
    session._modal_lookup_patcher.start()


def pytest_sessionfinish(session: pytest.Session, exitstatus: int) -> None:
    patcher = getattr(session, "_modal_lookup_patcher", None)
    if patcher is not None:
        patcher.stop()


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "docker: requires Docker daemon (skipped when unavailable or DEEPSWE_SKIP_DOCKER_SELFTESTS=1)",
    )


def pytest_collection_modifyitems(config: pytest.Config, items: list[pytest.Item]) -> None:
    skip_docker = os.environ.get("DEEPSWE_SKIP_DOCKER_SELFTESTS", "") == "1"
    docker_marker = pytest.mark.docker
    for item in items:
        if "docker" in item.nodeid or item.name.startswith("test_deepswe_docker_"):
            item.add_marker(docker_marker)
            if skip_docker:
                item.add_marker(
                    pytest.mark.skip(reason="DEEPSWE_SKIP_DOCKER_SELFTESTS=1")
                )
