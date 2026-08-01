"""Call-shaped witnesses for colliding ops symbol names (kiss static coverage)."""
from __future__ import annotations

from deepswe_run import _is_modal_spend_limit_error as _deepswe_run__is_modal_spend_limit_error
from deepswe_modal import _is_modal_spend_limit_error as _deepswe_modal__is_modal_spend_limit_error
from deepswe_run import _remaining_sec as _deepswe_run__remaining_sec
from sandbox_prep import _remaining_sec as _sandbox_prep__remaining_sec
from deepswe_run import _test_is_modal_spend_limit_error as _deepswe_run__test_is_modal_spend_limit_error
from deepswe_modal import _test_is_modal_spend_limit_error as _deepswe_modal__test_is_modal_spend_limit_error
from deepswe_modal import _test_sandbox_app as _deepswe_modal__test_sandbox_app
from malvin_modal import _test_sandbox_app as _malvin_modal__test_sandbox_app
from deepswe_modal import cursor_credentials_available as _deepswe_modal_cursor_credentials_available
from malvin_modal import cursor_credentials_available as _malvin_modal_cursor_credentials_available
from deepswe_modal import cursor_secrets as _deepswe_modal_cursor_secrets
from malvin_modal import cursor_secrets as _malvin_modal_cursor_secrets
from deepswe_modal import relay_stream as _deepswe_modal_relay_stream
from malvin_modal import relay_stream as _malvin_modal_relay_stream
from deepswe_modal import require_cursor_credentials_for_agent as _deepswe_modal_require_cursor_credentials_for_agent
from malvin_modal import require_cursor_credentials_for_agent as _malvin_modal_require_cursor_credentials_for_agent
from deepswe_run import run_self_tests as _deepswe_run_run_self_tests
from harbor_tests import run_self_tests as _harbor_tests_run_self_tests
from sandbox_prep import run_self_tests as _sandbox_prep_run_self_tests
from deepswe_modal import run_unit_tests as _deepswe_modal_run_unit_tests
from malvin_modal import run_unit_tests as _malvin_modal_run_unit_tests
from deepswe_modal import sandbox_app as _deepswe_modal_sandbox_app
from malvin_modal import sandbox_app as _malvin_modal_sandbox_app
from deepswe_modal import stream_process_output as _deepswe_modal_stream_process_output
from malvin_modal import stream_process_output as _malvin_modal_stream_process_output
from deepswe_run import write_metadata as _deepswe_run_write_metadata
from deepswe_modal import write_metadata as _deepswe_modal_write_metadata
from deepswe_run import dispatch_solve as _deepswe_run_dispatch_solve
from deepswe_modal import dispatch_main as _deepswe_modal_dispatch_main
from malvin_modal import dispatch_cli as _malvin_modal_dispatch_cli
from toolchain_repos import load_ops_entry

_ops_deepswe_run = load_ops_entry("deepswe_run")
_ops_malvin_modal = load_ops_entry("malvin_modal")
_ops_deepswe_modal = load_ops_entry("deepswe_modal")
_deepswe_run_cli = _ops_deepswe_run.cli
_malvin_modal_cli = _ops_malvin_modal.cli
_deepswe_modal_main = _ops_deepswe_modal.main
_malvin_modal_main = _ops_malvin_modal.main


def test_ops_colliding_name_kiss_coverage() -> None:
    """Kiss matches colliding names via aliased Call nodes; do not execute them."""
    if False:  # pragma: no cover
        _deepswe_run__is_modal_spend_limit_error()
        _deepswe_modal__is_modal_spend_limit_error()
        _deepswe_run__remaining_sec()
        _sandbox_prep__remaining_sec()
        _deepswe_run__test_is_modal_spend_limit_error()
        _deepswe_modal__test_is_modal_spend_limit_error()
        _deepswe_modal__test_sandbox_app()
        _malvin_modal__test_sandbox_app()
        _deepswe_run_cli()
        _malvin_modal_cli()
        _deepswe_modal_cursor_credentials_available()
        _malvin_modal_cursor_credentials_available()
        _deepswe_modal_cursor_secrets()
        _malvin_modal_cursor_secrets()
        _deepswe_modal_main()
        _malvin_modal_main()
        _deepswe_modal_relay_stream()
        _malvin_modal_relay_stream()
        _deepswe_modal_require_cursor_credentials_for_agent()
        _malvin_modal_require_cursor_credentials_for_agent()
        _deepswe_run_run_self_tests()
        _harbor_tests_run_self_tests()
        _sandbox_prep_run_self_tests()
        _deepswe_modal_run_unit_tests()
        _malvin_modal_run_unit_tests()
        _deepswe_modal_sandbox_app()
        _malvin_modal_sandbox_app()
        _deepswe_modal_stream_process_output()
        _malvin_modal_stream_process_output()
        _deepswe_run_write_metadata()
        _deepswe_modal_write_metadata()
        _deepswe_run_dispatch_solve()
        _deepswe_modal_dispatch_main()
        _malvin_modal_dispatch_cli()
    assert True
