#!/usr/bin/env python3
"""Minimal probe: run cursor-agent in a Modal agent sandbox (same image/network as Gate B)."""

from __future__ import annotations

import sys
from pathlib import Path

import click
import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kiss_coverage_common import register_kiss_static_symbols
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import (
    APP_REMOTE,
    agent_sandbox_network_kwargs,
    app,
    cursor_secrets,
    harbor_agent_image,
    sandbox_app,
    stream_process_output,
    validate_toolchain_repos,
)
from deepswe_run import materialize_workspace, parse_task_dir

PROBE_SCRIPT = r"""set -uo pipefail
run_probe() {
  local label="$1"
  shift
  echo "=== PROBE: $label ==="
  START=$(date +%s)
  set +e
  "$@" 2>&1
  RC=$?
  END=$(date +%s)
  echo "=== result label=$label exit_code=$RC elapsed_sec=$((END-START)) ==="
  echo
}

echo '=== which ==='
which agent cursor-agent 2>/dev/null || true
echo '=== versions ==='
agent --version 2>&1 || true
echo '=== auth status ==='
agent auth status 2>&1 || true

run_probe 'cursor-agent --force --trust -p Hello' cursor-agent --force --trust -p Hello
if [ "${QUICK:-0}" != 1 ]; then
  run_probe 'cursor-agent --force --trust -p Hello' cursor-agent --force --trust -p Hello
  run_probe 'cursor-agent --force --yolo -p Hello' cursor-agent --force --yolo -p Hello
  run_probe 'cursor-agent --force --trust --model auto -p Hello' cursor-agent --force --trust --model auto -p Hello
fi
if [ "${VARIANT:-}" = trust_auto ]; then
  run_probe 'cursor-agent --force --trust --model auto -p Hello' cursor-agent --force --trust --model auto -p Hello
fi
run_probe 'HTTPS api2.cursor.sh' curl -sS --max-time 15 -o /dev/null -w 'http_code=%{http_code} time=%{time_total}s\n' https://api2.cursor.sh/ || echo curl_failed
if [ "${MALVIN_PROBE:-0}" = 1 ]; then
  echo 'Reply with exactly: ok' > plan.md
  run_probe 'malvin code plan.md (90s cap)' timeout 90 malvin code plan.md
fi
exit 0
"""


def probe_in_sandbox(
    image: modal.Image,
    *,
    secrets: list[modal.Secret],
    timeout: int = 900,
    open_network: bool = False,
    quick: bool = False,
    variant: str = "",
    malvin_probe: bool = False,
) -> None:
    sandbox: modal.Sandbox | None = None
    try:
        if open_network:
            net_kwargs: dict = {}
        else:
            net_kwargs = agent_sandbox_network_kwargs(image)
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=image,
            workdir=APP_REMOTE,
            secrets=secrets,
            timeout=timeout,
            **net_kwargs,
        )
        quick_prefix = (
            f"QUICK={1 if quick else 0}; VARIANT={variant!r}; "
            f"MALVIN_PROBE={1 if malvin_probe else 0}; "
        )
        proc = sandbox.exec(
            "bash",
            "-lc",
            quick_prefix + PROBE_SCRIPT,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
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
def main(
    task_dir: Path,
    workspace: Path | None,
    open_network: bool,
    quick: bool,
    variant: str,
    malvin_probe: bool,
) -> None:
    """Build harbor agent image and exec cursor-agent --force -p Hello in Modal."""
    spec = parse_task_dir(task_dir)
    workspace = workspace or (task_dir.parent.parent / "evaluations" / "deepswe" / spec.task_id / "workspace")
    if not workspace.is_dir():
        from deepswe_run import default_deepswe_results_dir

        workspace = default_deepswe_results_dir() / spec.task_id / "workspace"
    materialize_workspace(spec, workspace, dry_run=False)
    malvin_repo, kiss_repo = validate_toolchain_repos()
    click.echo(f"Task: {spec.task_id}")
    click.echo(f"Workspace: {workspace.resolve()}")
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
    click.echo("Running cursor-agent probe in Modal sandbox...")
    if open_network:
        click.echo("Network: OPEN (no CIDR allowlist)")
    else:
        click.echo("Network: Cursor API CIDR allowlist (Gate B default)")
    probe_in_sandbox(
        image,
        secrets=cursor_secrets(),
        open_network=open_network,
        quick=quick,
        variant=variant,
        malvin_probe=malvin_probe,
    )


@app.local_entrypoint(name="probe_cursor_agent")
def probe_cursor_agent_entry(*arglist: str) -> None:
    main.main(args=list(arglist), prog_name="modal run ops/probe_cursor_agent_modal.py", standalone_mode=True)



def test_kiss_static_coverage() -> None:
    """Register production symbols for kiss static test coverage."""
    register_kiss_static_symbols(probe_in_sandbox, main, probe_cursor_agent_entry)

if __name__ == "__main__":
    main()
