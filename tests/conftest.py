"""Shared pytest configuration for malvin unit tests."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

OPS = Path(__file__).resolve().parents[1] / "ops"
if str(OPS) not in sys.path:
    sys.path.insert(0, str(OPS))


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
