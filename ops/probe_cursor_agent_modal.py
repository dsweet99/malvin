#!/usr/bin/env python3
"""Click + Modal entry for probe_cursor_agent_modal."""

from __future__ import annotations

from pathlib import Path

import click

from _ops_bootstrap import load_library

_lib = load_library("probe_cursor_agent_modal")
app = load_library("deepswe_modal").app


@click.command()
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    required=True,
)
@click.option(
    "--workspace",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--open-network",
    is_flag=True,
    help="Disable CIDR allowlist (full egress) for A/B test.",
)
@click.option(
    "--quick",
    is_flag=True,
    help="Only run cursor-agent --force -p Hello (skip trust/yolo variants).",
)
@click.option(
    "--variant",
    default="",
    show_default=True,
    help="Probe variant label (e.g. trust_auto).",
)
@click.option(
    "--malvin-probe",
    is_flag=True,
    help="Also run `malvin code plan.md` (90s cap) after cursor-agent probes.",
)
def probe_cursor_agent_main(
    task_dir: Path,
    workspace: Path | None,
    open_network: bool,
    quick: bool,
    variant: str,
    malvin_probe: bool,
) -> None:
    """Build harbor agent image and exec cursor-agent --force -p Hello in Modal."""
    _lib.run_probe_cursor_agent(
        task_dir, workspace, open_network, quick, variant, malvin_probe
    )


main = probe_cursor_agent_main


@app.local_entrypoint(name="probe_cursor_agent")
def probe_cursor_agent_entry(*arglist: str) -> None:
    main.main(
        args=list(arglist),
        prog_name="modal run ops/probe_cursor_agent_modal.py",
        standalone_mode=True,
    )


__all__ = ["app", "main", "probe_cursor_agent_main", "probe_cursor_agent_entry"]

if __name__ == "__main__":
    main()
