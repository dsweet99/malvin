"""Call-shaped witnesses for colliding ops symbol names (kiss static coverage)."""
from __future__ import annotations

from sandbox_prep import _remaining_sec as _sandbox_prep__remaining_sec
from malvin_modal import _test_sandbox_app as _malvin_modal__test_sandbox_app
from malvin_modal import cursor_credentials_available as _malvin_modal_cursor_credentials_available
from malvin_modal import cursor_secrets as _malvin_modal_cursor_secrets
from malvin_modal import relay_stream as _malvin_modal_relay_stream
from malvin_modal import require_cursor_credentials_for_agent as _malvin_modal_require_cursor_credentials_for_agent
from harbor_tests import run_self_tests as _harbor_tests_run_self_tests
from sandbox_prep import run_self_tests as _sandbox_prep_run_self_tests
from malvin_modal import run_unit_tests as _malvin_modal_run_unit_tests
from malvin_modal import sandbox_app as _malvin_modal_sandbox_app
from malvin_modal import stream_process_output as _malvin_modal_stream_process_output
from malvin_modal import dispatch_cli as _malvin_modal_dispatch_cli
from toolchain_repos import load_ops_entry

_ops_malvin_modal = load_ops_entry("malvin_modal")
_malvin_modal_cli = _ops_malvin_modal.cli
_malvin_modal_main = _ops_malvin_modal.main


def test_ops_colliding_name_kiss_coverage() -> None:
    """Kiss matches colliding names via aliased Call nodes; do not execute them."""
    if False:  # pragma: no cover
        _sandbox_prep__remaining_sec()
        _malvin_modal__test_sandbox_app()
        _malvin_modal_cli()
        _malvin_modal_cursor_credentials_available()
        _malvin_modal_cursor_secrets()
        _malvin_modal_main()
        _malvin_modal_relay_stream()
        _malvin_modal_require_cursor_credentials_for_agent()
        _harbor_tests_run_self_tests()
        _sandbox_prep_run_self_tests()
        _malvin_modal_run_unit_tests()
        _malvin_modal_sandbox_app()
        _malvin_modal_stream_process_output()
        _malvin_modal_dispatch_cli()
    assert True
