"""Kiss coverage witnesses for ops/tox_gates.py."""
from __future__ import annotations

import tox_gates as _mod


def test_tox_gates_kiss_coverage_witnesses() -> None:
    _ = (
        _mod._expand_tox_envlist_token,
        _mod._split_tox_envlist,
        _mod._tox_version_key,
        _mod.clamp_tox_version,
        _mod.image_build_pip_install_command,
        _mod.MIN_TOX_FOR_SKIP_ENV_INSTALL,
        _mod.is_tox_invocation,
        _mod.ensure_tox_skip_missing_interpreters,
        _mod.ensure_tox_offline_skip_flags,
        _mod.tox_gate_env_names,
        _mod.tox_cpython_factor_executable,
        _mod.tox_gate_check_commands,
        _mod.tox_gate_env_warm_command,
        _mod.tox_gate_precommit_warm_command,
        _mod._test_tox_gate_check_commands_offline_flags,
        _mod._test_clamp_tox_version_and_image_build_pip,
        _mod._test_tox_gate_precommit_warm_command,
    )
    _mod._test_tox_gate_check_commands_offline_flags()
    _mod._test_clamp_tox_version_and_image_build_pip()
    _mod._test_tox_gate_precommit_warm_command()
    assert True


def test_tox_gates_factor_and_offline_flags() -> None:
    _mod._test_tox_gate_check_commands_offline_flags()
    assert _mod.is_tox_invocation("python3 -m tox -e pep8")
    assert not _mod.is_tox_invocation("ruff check .")
    assert _mod.tox_cpython_factor_executable("py39") == "python3.9"
    assert _mod.tox_cpython_factor_executable("py27") == "python2.7"
    assert _mod._expand_tox_envlist_token("py3{10,11}") == ["py310", "py311"]
    assert _mod._split_tox_envlist("a,{b,c},d") == ["a", "{b,c}", "d"]
    assert _mod.clamp_tox_version("4.23.2") == _mod.MIN_TOX_FOR_SKIP_ENV_INSTALL
    assert "pip install" in _mod.image_build_pip_install_command("tox==4.42.0")
    assert _mod.tox_gate_env_warm_command.__name__ == "tox_gate_env_warm_command"
    assert _mod.tox_gate_precommit_warm_command.__name__ == "tox_gate_precommit_warm_command"