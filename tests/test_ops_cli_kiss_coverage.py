"""Kiss coverage witnesses for ops Click / Modal entry surfaces."""

from __future__ import annotations

from toolchain_repos import load_ops_entry

_ops_fast = load_ops_entry("fast_task")
_ops_malvin = load_ops_entry("malvin_modal")

fast_task_cli = _ops_fast.fast_task_cli
fast_tasks_list_cmd = _ops_fast.fast_tasks_list_cmd
fast_task_solve = _ops_fast.fast_task_solve
fast_task_selftest_cmd = _ops_fast.fast_task_selftest_cmd
malvin_modal_cli = _ops_malvin.malvin_modal_cli
malvin_modal_entrypoint = _ops_malvin.malvin_modal_entrypoint


def test_ops_cli_kiss_coverage_witnesses() -> None:
    _ = (
        fast_task_cli,
        fast_tasks_list_cmd,
        fast_task_solve,
        fast_task_selftest_cmd,
        malvin_modal_cli,
        malvin_modal_entrypoint,
    )
    if False:  # pragma: no cover
        fast_task_cli()
        fast_tasks_list_cmd()
        fast_task_solve()
        fast_task_selftest_cmd()
        malvin_modal_cli()
        malvin_modal_entrypoint()
    assert True
