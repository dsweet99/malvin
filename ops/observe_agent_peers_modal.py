#!/usr/bin/env python3
"""Click + Modal entry for observe_agent_peers_modal."""

from __future__ import annotations

import sys
from pathlib import Path

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))
from _ops_bootstrap import load_library  # noqa: E402

_lib = load_library("observe_agent_peers_modal")
app = load_library("deepswe_modal").app


@click.command()
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    required=True,
)
def observe_agent_peers_main(task_dir: Path) -> None:
    """Run cursor-agent under open egress and print observed TCP peer IPs."""
    _lib.run_observe_agent_peers(task_dir)


main = observe_agent_peers_main


@app.local_entrypoint(name="observe_agent_peers")
def observe_agent_peers_entry(*arglist: str) -> None:
    main.main(
        args=list(arglist),
        prog_name="modal run ops/observe_agent_peers_modal.py",
        standalone_mode=True,
    )


__all__ = ["app", "main", "observe_agent_peers_main", "observe_agent_peers_entry"]

if __name__ == "__main__":
    main()
