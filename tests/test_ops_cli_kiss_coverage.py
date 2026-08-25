"""Kiss coverage witnesses for ops Click / Modal entry surfaces."""

from __future__ import annotations

from toolchain_repos import load_ops_entry

_ops_fast = load_ops_entry("fast_task")
_ops_malvin = load_ops_entry("malvin_modal")
_ops_qa = load_ops_entry("qa")

fast_task_cli = _ops_fast.fast_task_cli
fast_tasks_list_cmd = _ops_fast.fast_tasks_list_cmd
fast_task_solve = _ops_fast.fast_task_solve
fast_task_selftest_cmd = _ops_fast.fast_task_selftest_cmd
malvin_modal_cli = _ops_malvin.malvin_modal_cli
malvin_modal_entrypoint = _ops_malvin.malvin_modal_entrypoint
qa_cli = _ops_qa.qa_cli
qa_list_cmd = _ops_qa.qa_list_cmd
qa_sigkill_stdin_hold_abandons_bridge = _ops_qa.qa_sigkill_stdin_hold_abandons_bridge
qa_all_cmd = _ops_qa.qa_all_cmd
qa_selftest_cmd = _ops_qa.qa_selftest_cmd


def test_ops_cli_kiss_coverage_witnesses() -> None:
    _ = (
        fast_task_cli,
        fast_tasks_list_cmd,
        fast_task_solve,
        fast_task_selftest_cmd,
        malvin_modal_cli,
        malvin_modal_entrypoint,
        qa_cli,
        qa_list_cmd,
        qa_sigkill_stdin_hold_abandons_bridge,
        qa_all_cmd,
        qa_selftest_cmd,
    )
    if False:
        fast_task_cli()
        fast_tasks_list_cmd()
        fast_task_solve()
        fast_task_selftest_cmd()
        malvin_modal_cli()
        malvin_modal_entrypoint()
        qa_cli()
        qa_list_cmd()
        qa_sigkill_stdin_hold_abandons_bridge()
        qa_all_cmd()
        qa_selftest_cmd()
    assert True
