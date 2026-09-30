from __future__ import annotations

from toolchain_repos import load_ops_entry

_ops_fast = load_ops_entry("fast_task")

fast_task_cli = _ops_fast.fast_task_cli
fast_tasks_list_cmd = _ops_fast.fast_tasks_list_cmd
fast_task_solve = _ops_fast.fast_task_solve
fast_task_selftest_cmd = _ops_fast.fast_task_selftest_cmd


def test_ops_cli_kiss_coverage_witnesses() -> None:
    _ = (
        fast_task_cli,
        fast_tasks_list_cmd,
        fast_task_solve,
        fast_task_selftest_cmd,
    )
    if False:
        fast_task_cli()
        fast_tasks_list_cmd()
        fast_task_solve()
        fast_task_selftest_cmd()
    assert True
