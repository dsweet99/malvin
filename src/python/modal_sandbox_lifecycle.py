
from __future__ import annotations

from typing import Any

def release_modal_sandbox(sandbox: Any) -> None:
    if sandbox is None:
        return
    sandbox.terminate()
    detach = getattr(sandbox, "detach", None)
    if detach is not None:
        detach()
        return
    router = getattr(sandbox, "_command_router_client", None)
    if router is not None:
        close = getattr(router, "close", None)
        if close is not None:
            close()

def _test_release_modal_sandbox() -> None:
    from unittest.mock import MagicMock

    sandbox = MagicMock()
    release_modal_sandbox(sandbox)
    sandbox.terminate.assert_called_once()
    sandbox.detach.assert_called_once()

    class _LegacySandbox:
        def terminate(self) -> None:
            self.terminated = True

    legacy = _LegacySandbox()
    router = MagicMock()
    legacy._command_router_client = router
    release_modal_sandbox(legacy)
    assert legacy.terminated
    router.close.assert_called_once()

    release_modal_sandbox(None)

