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
    """Stub Modal app lookup for unit tests unless live Modal is requested.

    Always stub under CI / when credentials are absent. Opt into live Modal with
    ``MALVIN_LIVE_MODAL=1`` when credentials are configured.
    """
    live = os.environ.get("MALVIN_LIVE_MODAL", "") == "1"
    if live and _modal_credentials_configured():
        return
    try:
        import modal
    except ImportError:
        return

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
        "docker: requires Docker daemon (skipped when unavailable or MALVIN_SKIP_DOCKER_SELFTESTS=1)",
    )


def pytest_collection_modifyitems(config: pytest.Config, items: list[pytest.Item]) -> None:
    skip_docker = os.environ.get("MALVIN_SKIP_DOCKER_SELFTESTS", "") == "1"
    docker_marker = pytest.mark.docker
    for item in items:
        if "docker" in item.nodeid:
            item.add_marker(docker_marker)
            if skip_docker:
                item.add_marker(
                    pytest.mark.skip(reason="MALVIN_SKIP_DOCKER_SELFTESTS=1")
                )
