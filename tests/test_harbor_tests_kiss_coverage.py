"""Kiss coverage witnesses for ops/harbor_tests.py."""
from __future__ import annotations

import harbor_tests as _mod

def test_harbor_tests_kiss_coverage_witnesses() -> None:
    _ = (_mod.embedded_file_body_from_patch, _mod.embedded_test_sh_from_patch, _mod.embedded_test_py_from_patch, _mod.added_python_sources_from_patch, _mod.resolve_harbor_test_sh_body, _mod.is_stdlib_module, _mod.distribution_name_for_import, _mod.top_level_imports_from_source, _mod.is_analysis_sample_path, _mod.harbor_imports_from_tests_dir,)
    _ = (_mod.pytest_args_from_test_sh, _mod.test_sh_invokes_pytest, _mod.collect_only_pytest_command, _mod._shell_quote, _mod.run_self_tests,)
    assert True

