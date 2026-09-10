
from __future__ import annotations

from types import SimpleNamespace
from typing import Any
from unittest.mock import patch

import modal

def lookup_sandbox_app(module_app: modal.App, app_name: str) -> modal.App:
    if module_app.app_id is not None:
        return module_app
    return modal.App.lookup(app_name, create_if_missing=True)

def test_sandbox_app_lookup(
    module_name: str,
    module_app: Any,
    app_name: str,
    sandbox_app: Any,
) -> None:
    lookup_app = SimpleNamespace(app_id="lookup-id")
    bound_app = SimpleNamespace(app_id="module-id")
    with patch(f"{module_name}.app", SimpleNamespace(app_id=None)):
        with patch.object(modal.App, "lookup", return_value=lookup_app) as mock_lookup:
            assert sandbox_app() is lookup_app
        mock_lookup.assert_called_once_with(app_name, create_if_missing=True)
    with patch(f"{module_name}.app", bound_app):
        assert sandbox_app() is bound_app
