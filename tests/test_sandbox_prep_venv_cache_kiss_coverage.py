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


def test_minimal_venv_dir_tolerates_a_concurrent_python_link(tmp_path, monkeypatch) -> None:
    original_symlink_to = _mod.Path.symlink_to
    link_was_created = False

    def peer_creates_link(path, target, target_is_directory=False) -> None:
        nonlocal link_was_created
        if not link_was_created:
            original_symlink_to(path, target, target_is_directory=target_is_directory)
            link_was_created = True
            raise FileExistsError
        original_symlink_to(path, target, target_is_directory=target_is_directory)

    monkeypatch.setattr(_mod.Path, "symlink_to", peer_creates_link)

    venv = _mod.minimal_venv_dir(tmp_path / "venv")

    assert link_was_created
    assert (venv / "bin" / "python").is_symlink()
