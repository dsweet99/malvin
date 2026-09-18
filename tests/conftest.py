from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parents[1]
_SRC_PYTHON = _REPO / "src" / "python"
if str(_SRC_PYTHON) not in sys.path:
    sys.path.insert(0, str(_SRC_PYTHON))


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
