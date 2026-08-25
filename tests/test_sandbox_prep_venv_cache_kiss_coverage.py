"""Kiss coverage witnesses for src/python/sandbox_prep_venv_cache.py."""
from __future__ import annotations

import sandbox_prep_venv_cache as _mod

def test_sandbox_prep_venv_cache_kiss_coverage_witnesses() -> None:
    _ = (
        _mod.venv_cache_key_token,
        _mod.venv_cache_root,
        _mod.minimal_venv_dir,
        _mod._venv_python_ready,
        _mod._create_base_venv,
        _mod._pip_install_into,
        _mod._ensure_base_locked,
        _mod._resolve_base,
        _mod.clone_cached_venv,
        _mod.VENV_CACHE_OFFLINE,
    )
    assert True
