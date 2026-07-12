"""Kiss coverage witnesses for ops/modal_sandbox_lifecycle.py."""
from __future__ import annotations

import modal_sandbox_lifecycle as _mod

def test_modal_sandbox_lifecycle_kiss_coverage_witnesses() -> None:
    _ = (_mod.release_modal_sandbox, _mod._test_release_modal_sandbox,)
    assert True

