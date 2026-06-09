"""Modal sandbox teardown helpers (release gRPC connections after terminate)."""

from __future__ import annotations

from typing import Any


def release_modal_sandbox(sandbox: Any) -> None:
    """Terminate a Modal sandbox and release client gRPC connections.

    Modal sandboxes hold a direct gRPC channel to the task command router. Calling
    ``terminate()`` alone leaves that channel open until garbage collection, which
    triggers ``Unclosed connection`` warnings and grpclib ``AttributeError`` on
    ``Channel.__del__``. Modal >= 1.4 provides ``detach()``; older releases need the
    command-router ``close()`` fallback.
    """
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

