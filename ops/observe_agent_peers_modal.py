#!/usr/bin/env python3
"""Observe TCP peers while cursor-agent runs under open Modal egress."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import click
import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import (
    APP_REMOTE,
    OBSERVE_AGENT_PEERS_SCRIPT,
    app,
    cursor_secrets,
    harbor_agent_image,
    resolve_agent_sandbox_cidrs,
    sandbox_app,
    validate_toolchain_repos,
)
from deepswe_run import materialize_workspace, parse_task_dir


def observe_in_sandbox(
    image: modal.Image,
    *,
    secrets: list[modal.Secret],
    timeout: int = 900,
) -> dict:
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=image,
            workdir=APP_REMOTE,
            secrets=secrets,
            timeout=timeout,
        )
        proc = sandbox.exec(
            "python3",
            "-c",
            OBSERVE_AGENT_PEERS_SCRIPT,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
        )
        stdout = proc.stdout.read()
        stderr = proc.stderr.read()
        proc.wait()
        if stderr.strip():
            click.echo(stderr, err=True)
        return json.loads(stdout.strip() or "{}")
    finally:
        if sandbox is not None:
            release_modal_sandbox(sandbox)


@click.command()
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    required=True,
)
def main(task_dir: Path) -> None:
    """Run cursor-agent under open egress and print observed TCP peer IPs."""
    spec = parse_task_dir(task_dir)
    workspace = (
        task_dir.parent.parent
        / "evaluations"
        / "deepswe"
        / spec.task_id
        / "workspace"
    )
    materialize_workspace(spec, workspace, dry_run=False)
    malvin_repo, kiss_repo = validate_toolchain_repos()
    deepswe_run_py = Path(__file__).resolve().parent / "deepswe_run.py"
    image = harbor_agent_image(
        spec,
        workspace,
        spec.tests_dir,
        dockerfile=spec.dockerfile,
        malvin_repo=malvin_repo,
        kiss_repo=kiss_repo,
        deepswe_run_py=deepswe_run_py,
    )
    click.echo("Observing cursor-agent TCP peers (open egress)...")
    observed = observe_in_sandbox(image, secrets=cursor_secrets())
    peer_ips = observed.get("peer_ips", [])
    click.echo(f"observed_peer_ips={peer_ips}")
    click.echo(f"observed_ports={observed.get('peer_ports', [])}")
    click.echo(f"agent_exit={observed.get('exit_code')}")

    allowlist = resolve_agent_sandbox_cidrs(image)
    allow_ips = {cidr.split("/")[0] for cidr in allowlist}
    missing = sorted(set(peer_ips) - allow_ips)
    click.echo(f"allowlist_size={len(allowlist)}")
    click.echo(f"missing_from_allowlist={missing}")


@app.local_entrypoint(name="observe_agent_peers")
def observe_agent_peers_entry(*arglist: str) -> None:
    main.main(
        args=list(arglist),
        prog_name="modal run ops/observe_agent_peers_modal.py",
        standalone_mode=True,
    )


if __name__ == "__main__":
    main()
