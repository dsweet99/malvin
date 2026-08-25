"""Click + Modal entry for malvin_modal — implementation in ``src/python/malvin_modal.py``."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
_lib = importlib.import_module("_ops_bootstrap").load_library("malvin_modal")
app = _lib.app

@click.command(
    context_settings={
        "help_option_names": ["-h", "--help"],
        "allow_extra_args": True,
        "ignore_unknown_options": True,
    },
)
@click.option(
    "--self-test",
    is_flag=True,
    help="Run local unit tests without Modal credentials.",
)
@click.pass_context
def malvin_modal_cli(ctx: click.Context, self_test: bool) -> None:
    """Run malvin on Modal, forwarding arguments to the remote process."""
    _lib.dispatch_cli(ctx, self_test)

cli = malvin_modal_cli

@app.local_entrypoint(name="main")
def malvin_modal_entrypoint(*arglist: str) -> None:
    """Modal entry: ``modal run ops/malvin_modal.py -- [MALVIN_ARGS...]``."""
    cli.main(
        args=list(arglist),
        prog_name="modal run ops/malvin_modal.py",
        standalone_mode=True,
    )

main = malvin_modal_entrypoint
run_unit_tests = _lib.run_unit_tests

__all__ = ["app", "cli", "main", "malvin_modal_cli", "malvin_modal_entrypoint", "run_unit_tests"]

if __name__ == "__main__":
    malvin_modal_cli(prog_name="python ops/malvin_modal.py")
