#!/usr/bin/env python3
"""Run DeepSWE Harbor verifier (and optionally malvin agent) on Modal.

No local Docker required for grading: builds a Modal Image from the task
Harbor Dockerfile (or pulls the registry image) and execs ``deepswe_run.py``
once inside a sandbox (agent + grade in one command when not ``--grade-only``).

Agent sandboxes restrict outbound network to Cursor API endpoints. Grade-only
sandboxes block all network access. malvin and kiss are built from local source
trees (``MALVIN_REPO`` / ``KISS_REPO``), not crates.io.

Prerequisites: Modal CLI authenticated; Cursor API key in ``CURSOR_AGENT_API_KEY``,
``CURSOR_API_KEY``, or ``AGENT_API_KEY``; malvin repo at parent of ``ops/``; kiss at
``../kiss`` or ``KISS_REPO``; DeepSWE task at ``../deep-swe/tasks/...``.

Artifacts land under ``~/.malvin/deepswe-results/<task_id>/modal_<timestamp>/``
(``metadata.json``, ``reward.txt``). Workspace: ``.../workspace``.

Examples::

    # Gate A — verifier and Modal image sanity (expect reward=1):
    modal run ops/deepswe_modal.py --grade-only --apply-solution \\
        --task ../deep-swe/tasks/bandit-interprocedural-taint-checks

    # Gate B — normative live eval (default tenacious ``malvin code`` + Harbor grade):
    modal run ops/deepswe_modal.py \\
        --task ../deep-swe/tasks/bandit-interprocedural-taint-checks \\
        --command code

Local unit tests (no Modal credentials)::

    python ops/deepswe_modal.py --self-test
"""

from __future__ import annotations

import io
import json
import os
import socket
import sys
import tarfile
import tempfile
import threading
from datetime import datetime, timezone
from pathlib import Path
from types import SimpleNamespace
from typing import Any, TextIO
from unittest.mock import MagicMock, patch

import click
from click.testing import CliRunner
import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from deepswe_run import (
    DEFAULT_CHECKS_CODE,
    DEFAULT_CHECKS_DO,
    apply_patch,
    default_deepswe_results_dir,
    materialize_workspace,
    parse_task_dir,
    reset_workspace,
    timestamp_dir,
    write_plan_and_checks,
)

DEEPSWE_RUN_REMOTE = "/opt/malvin/ops/deepswe_run.py"
TASK_REMOTE = "/task"

APP_NAME = "deepswe-modal"
LOGS_REMOTE = "/logs"
APP_REMOTE = "/app"
TESTS_REMOTE = "/tests"
MALVIN_TOOLCHAIN_REMOTE = "/opt/toolchain/malvin"
KISS_TOOLCHAIN_REMOTE = "/opt/toolchain/kiss"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)

CURSOR_API_HOSTS = (
    "api2.cursor.sh",
    "api2geo.cursor.sh",
    "api2direct.cursor.sh",
    "api.cursor.com",
    "api3.cursor.sh",
    "repo42.cursor.sh",
)

# Modal Sandbox.create defaults are 0.125 CPU and 128 MiB — too small for malvin +
# cursor-agent. Match malvin's default mem_limit_gb (default_repo/config.toml).
AGENT_SANDBOX_CPU = 2.0
AGENT_SANDBOX_MEMORY_MIB = 4096
GRADE_SANDBOX_CPU = 1.0
GRADE_SANDBOX_MEMORY_MIB = 2048

app = modal.App(APP_NAME)


def sandbox_app() -> modal.App:
    """Return an initialized Modal app for sandbox creation."""
    if app.app_id is not None:
        return app
    return modal.App.lookup(APP_NAME, create_if_missing=True)


def malvin_repo_root() -> Path:
    """Return the malvin repository root (parent of ``ops/``)."""
    return Path(__file__).resolve().parent.parent


def kiss_repo_root() -> Path:
    """Return the kiss-ai source tree (``KISS_REPO`` or sibling ``kiss`` repo)."""
    override = os.environ.get("KISS_REPO")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "kiss"


def validate_toolchain_repos() -> tuple[Path, Path]:
    """Ensure local malvin and kiss trees exist before building the agent image."""
    malvin_repo = malvin_repo_root()
    kiss_repo = kiss_repo_root()
    if not (malvin_repo / "Cargo.toml").is_file():
        raise click.ClickException(f"malvin repo not found: {malvin_repo}")
    if not (kiss_repo / "Cargo.toml").is_file():
        raise click.ClickException(
            f"kiss repo not found: {kiss_repo} (set KISS_REPO to override)"
        )
    return malvin_repo, kiss_repo


def malvin_upload_ignore() -> list[str]:
    """Exclude heavy or ephemeral paths when uploading malvin source to Modal."""
    return [
        "target/",
        "experiments/",
        ".cargo/",
        ".malvin/",
        ".git",
        ".kissignore",
        "__pycache__/",
        "results/",
        "reports/",
    ]


def kiss_upload_ignore() -> list[str]:
    """Exclude build artifacts when uploading kiss source to Modal."""
    return ["target/", ".git", "__pycache__/"]


def resolve_cursor_api_cidrs_from_hosts(hosts: tuple[str, ...] = CURSOR_API_HOSTS) -> list[str]:
    """Resolve hostnames to CIDR strings via the local resolver (host DNS snapshot)."""
    cidrs: set[str] = set()
    for host in hosts:
        try:
            infos = socket.getaddrinfo(host, 443, type=socket.SOCK_STREAM)
        except socket.gaierror:
            continue
        for info in infos:
            ip = info[4][0]
            if ":" in ip:
                cidrs.add(f"{ip}/128")
            else:
                cidrs.add(f"{ip}/32")
    if not cidrs:
        raise click.ClickException(
            "Could not resolve Cursor API hosts for sandbox network allowlist"
        )
    return sorted(cidrs)


def resolve_cursor_api_cidrs() -> list[str]:
    """Resolve Cursor API hostnames from the host-side resolver."""
    return resolve_cursor_api_cidrs_from_hosts()


MODAL_EGRESS_DNS_SCRIPT = """
import json
import socket

HOSTS = {hosts!r}


def dns_ipv4(host: str) -> set[str]:
    out: set[str] = set()
    try:
        for info in socket.getaddrinfo(host, 443, type=socket.SOCK_STREAM):
            ip = info[4][0]
            if ":" not in ip:
                out.add(ip)
    except socket.gaierror:
        pass
    return out


cidrs: set[str] = set()
for host in HOSTS:
    for ip in dns_ipv4(host):
        cidrs.add(f"{{ip}}/32")
if not cidrs:
    raise SystemExit("Could not resolve Cursor API hostnames inside Modal sandbox")
print(json.dumps(sorted(cidrs)))
""".format(
    hosts=list(CURSOR_API_HOSTS),
)


RESOLVE_CURSOR_CIDRS_IN_SANDBOX_SCRIPT = """
import json
import socket
import ssl

HOSTS = {hosts!r}
CONNECTS_PER_HOST = 30
CONNECT_TIMEOUT_SEC = 8


def dns_ipv4(host: str) -> set[str]:
    out: set[str] = set()
    try:
        for info in socket.getaddrinfo(host, 443, type=socket.SOCK_STREAM):
            ip = info[4][0]
            if ":" not in ip:
                out.add(ip)
    except socket.gaierror:
        pass
    return out


def observe_peer_ips(host: str) -> set[str]:
    seen: set[str] = set()
    ctx = ssl.create_default_context()
    for _ in range(CONNECTS_PER_HOST):
        try:
            with socket.create_connection((host, 443), CONNECT_TIMEOUT_SEC) as sock:
                seen.add(sock.getpeername()[0])
                with ctx.wrap_socket(sock, server_hostname=host):
                    pass
        except OSError:
            continue
    return seen


cidrs: set[str] = set()
for host in HOSTS:
    for ip in dns_ipv4(host) | observe_peer_ips(host):
        if ":" not in ip:
            cidrs.add(f"{{ip}}/32")
if not cidrs:
    raise SystemExit(
        "Could not observe Cursor API peer IPs inside Modal egress sandbox"
    )
print(json.dumps(sorted(cidrs)))
""".format(
    hosts=list(CURSOR_API_HOSTS),
)


def cidr_probe_image() -> modal.Image:
    """Minimal image for Modal egress DNS probes (same sandbox network as agent runs)."""
    return modal.Image.debian_slim(python_version="3.12")


def _run_modal_cidr_probe_script(
    script: str,
    *,
    cidr_allowlist: list[str] | None = None,
    timeout: int = 300,
    error_label: str,
) -> list[str]:
    """Exec a probe script in a Modal sandbox and parse a JSON CIDR list from stdout."""
    probe_image = cidr_probe_image()
    sandbox: modal.Sandbox | None = None
    create_kwargs: dict[str, Any] = {
        "app": sandbox_app(),
        "image": probe_image,
        "timeout": timeout,
    }
    if cidr_allowlist is not None:
        create_kwargs["cidr_allowlist"] = modal_cidr_allowlist(cidr_allowlist)
    try:
        sandbox = modal.Sandbox.create(**create_kwargs)
        proc = sandbox.exec(
            "python3",
            "-c",
            script,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            text=True,
        )
        stdout = proc.stdout.read()
        stderr = proc.stderr.read()
        proc.wait()
        if proc.returncode != 0:
            detail = (stderr or stdout or "").strip()
            raise click.ClickException(
                error_label + (f": {detail}" if detail else "")
            )
        cidrs = json.loads(stdout.strip())
        if not isinstance(cidrs, list) or not all(isinstance(c, str) for c in cidrs):
            raise click.ClickException(f"{error_label}: invalid CIDR payload")
        if not cidrs:
            raise click.ClickException(f"{error_label}: empty CIDR list")
        return sorted(cidrs)
    finally:
        if sandbox is not None:
            sandbox.terminate()


def resolve_cursor_api_cidrs_in_modal_sandbox(
    image: modal.Image | None = None,
    *,
    timeout: int = 300,
) -> list[str]:
    """Resolve Cursor API hostnames from inside an open-egress Modal sandbox."""
    _ = image
    return _run_modal_cidr_probe_script(
        RESOLVE_CURSOR_CIDRS_IN_SANDBOX_SCRIPT,
        timeout=timeout,
        error_label="Modal egress open-network probe failed",
    )


def resolve_cursor_api_cidrs_under_allowlist(
    seed_cidrs: list[str],
    *,
    timeout: int = 300,
) -> list[str]:
    """Resolve Cursor API DNS from inside a Modal sandbox with a seeded CIDR allowlist."""
    return _run_modal_cidr_probe_script(
        MODAL_EGRESS_DNS_SCRIPT,
        cidr_allowlist=seed_cidrs,
        timeout=timeout,
        error_label="Modal egress allowlist DNS probe failed",
    )


def union_ipv4_cidrs(*cidr_lists: list[str]) -> list[str]:
    """Return sorted union of IPv4 /32 (or wider) CIDR strings."""
    return modal_cidr_allowlist(sorted({cidr for group in cidr_lists for cidr in group}))


def resolve_agent_sandbox_cidrs(
    image: modal.Image | None = None,
    *,
    timeout: int = 300,
    fixpoint_rounds: int = 3,
) -> list[str]:
    """Build agent allowlist: host DNS ∪ open Modal probe ∪ allowlist DNS fixpoint."""
    _ = image  # egress probes use cidr_probe_image(); agent image is irrelevant.
    host_cidrs = modal_cidr_allowlist(resolve_cursor_api_cidrs())
    open_modal_cidrs = modal_cidr_allowlist(
        resolve_cursor_api_cidrs_in_modal_sandbox(timeout=timeout)
    )
    cidrs = union_ipv4_cidrs(host_cidrs, open_modal_cidrs)
    fixpoint_added = 0
    for _ in range(fixpoint_rounds):
        before = len(cidrs)
        allowlist_dns = modal_cidr_allowlist(
            resolve_cursor_api_cidrs_under_allowlist(cidrs, timeout=timeout)
        )
        cidrs = union_ipv4_cidrs(cidrs, allowlist_dns)
        fixpoint_added += len(cidrs) - before
        if len(cidrs) == before:
            break
    click.echo(
        f"Cursor API allowlist: {len(cidrs)} IPv4 CIDRs "
        f"(host={len(host_cidrs)}, open_modal={len(open_modal_cidrs)}, "
        f"allowlist_dns_fixpoint=+{fixpoint_added})"
    )
    return cidrs


def agent_sandbox_network_kwargs(
    image: modal.Image | None = None,
    *,
    timeout: int = 300,
) -> dict[str, Any]:
    """Return Modal kwargs for agent sandboxes with Modal-aware Cursor CIDR allowlist."""
    return sandbox_network_kwargs(
        cursor_api_only=True,
        block_all=False,
        cidr_allowlist=resolve_agent_sandbox_cidrs(image, timeout=timeout),
    )


def modal_cidr_allowlist(cidrs: list[str]) -> list[str]:
    """Return IPv4-only CIDRs for Modal sandbox egress allowlists (no IPv6 support)."""
    v4 = [cidr for cidr in cidrs if ":" not in cidr.split("/", 1)[0]]
    if not v4:
        raise click.ClickException(
            "No IPv4 Cursor API CIDRs available for Modal sandbox allowlist"
        )
    return sorted(v4)


def sandbox_network_kwargs(
    *,
    cursor_api_only: bool,
    block_all: bool,
    cidr_allowlist: list[str] | None = None,
) -> dict[str, Any]:
    """Return Modal ``Sandbox.create`` kwargs for the requested network posture."""
    if block_all:
        return {"block_network": True}
    if cursor_api_only:
        cidrs = cidr_allowlist if cidr_allowlist is not None else resolve_cursor_api_cidrs()
        return {"cidr_allowlist": modal_cidr_allowlist(cidrs)}
    return {}


def agent_sandbox_resource_kwargs() -> dict[str, Any]:
    """Return Modal ``Sandbox.create`` cpu/memory for agent workloads."""
    return {"cpu": AGENT_SANDBOX_CPU, "memory": AGENT_SANDBOX_MEMORY_MIB}


def grade_sandbox_resource_kwargs() -> dict[str, Any]:
    """Return Modal ``Sandbox.create`` cpu/memory for Harbor grading."""
    return {"cpu": GRADE_SANDBOX_CPU, "memory": GRADE_SANDBOX_MEMORY_MIB}


def relay_stream(reader: Any, sink: TextIO) -> None:
    for chunk in reader:
        sink.write(chunk)
        sink.flush()


def stream_process_output(proc: Any, out: TextIO, err: TextIO) -> None:
    threads = [
        threading.Thread(target=relay_stream, args=(proc.stdout, out), daemon=True),
        threading.Thread(target=relay_stream, args=(proc.stderr, err), daemon=True),
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()


def harbor_image(spec: Any, *, dockerfile: Path) -> modal.Image:
    """Modal image with Harbor task dependencies; workspace/tests mounted at runtime."""
    if dockerfile.is_file():
        click.echo(f"Building Modal image from {dockerfile} (may take several minutes)...")
        return modal.Image.from_dockerfile(str(dockerfile))
    click.echo(f"Pulling Harbor image {spec.docker_image}...")
    return modal.Image.from_registry(spec.docker_image)


def mount_task_tree(image: modal.Image, workspace: Path, tests_dir: Path) -> modal.Image:
    return (
        image.add_local_dir(str(workspace.resolve()), remote_path=APP_REMOTE)
        .add_local_dir(str(tests_dir.resolve()), remote_path=TESTS_REMOTE)
    )


def mount_eval_context(
    image: modal.Image,
    *,
    task_dir: Path,
    workspace: Path,
    tests_dir: Path,
    deepswe_run_py: Path,
) -> modal.Image:
    """Layer workspace, tests, task metadata, and ``deepswe_run.py`` for one remote exec."""
    prepared = image.run_commands(
        "python3 -m pip install --break-system-packages click"
    )
    return (
        prepared.add_local_dir(str(workspace.resolve()), remote_path=APP_REMOTE)
        .add_local_dir(str(tests_dir.resolve()), remote_path=TESTS_REMOTE)
        .add_local_dir(str(task_dir.resolve()), remote_path=TASK_REMOTE)
        .add_local_file(str(deepswe_run_py.resolve()), remote_path=DEEPSWE_RUN_REMOTE)
    )


def mount_local_toolchain(
    image: modal.Image,
    *,
    malvin_repo: Path,
    kiss_repo: Path,
) -> modal.Image:
    """Layer local malvin/kiss source and build binaries inside the Modal image."""
    return (
        image.add_local_dir(
            str(malvin_repo.resolve()),
            remote_path=MALVIN_TOOLCHAIN_REMOTE,
            ignore=malvin_upload_ignore(),
            copy=True,
        )
        .add_local_dir(
            str(kiss_repo.resolve()),
            remote_path=KISS_TOOLCHAIN_REMOTE,
            ignore=kiss_upload_ignore(),
            copy=True,
        )
        .run_commands(
            f"bash -lc 'cargo install --path {KISS_TOOLCHAIN_REMOTE} --locked'",
            f"bash -lc 'RUSTC_WRAPPER= cargo install --path {MALVIN_TOOLCHAIN_REMOTE} --locked'",
            "curl -fsSL https://cursor.com/install | bash",
            "/root/.local/bin/agent --version || true",
        )
        .env({"PATH": TOOLCHAIN_PATH})
    )


def read_remote_file(sandbox: modal.Sandbox, path: str) -> str | None:
    try:
        with sandbox.open(path, "r") as handle:
            return handle.read()
    except OSError:
        return None


def read_remote_bytes(sandbox: modal.Sandbox, path: str) -> bytes | None:
    try:
        with sandbox.open(path, "rb") as handle:
            return handle.read()
    except OSError:
        return None


def harvest_sandbox_logs(sandbox: modal.Sandbox, out_dir: Path) -> dict[str, Any]:
    """Archive remote Harbor and malvin logs into ``out_dir/sandbox_logs/``."""
    prep = sandbox.exec(
        "bash",
        "-lc",
        f"mkdir -p {LOGS_REMOTE}/malvin && "
        f"(cp -a /root/.malvin/logs/. {LOGS_REMOTE}/malvin/ 2>/dev/null || true) && "
        f"tar -czf /tmp/deepswe_harvest.tgz -C / logs",
        workdir=APP_REMOTE,
    )
    prep.wait()
    blob = read_remote_bytes(sandbox, "/tmp/deepswe_harvest.tgz")
    if not blob:
        return {"harvested": False}
    logs_dir = out_dir / "sandbox_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as archive:
        archive.extractall(logs_dir)
    return {"harvested": True, "sandbox_logs_dir": str(logs_dir.resolve())}


def parse_deepswe_run_result(
    sandbox: modal.Sandbox,
    *,
    run_logs_remote: str,
    grade_only: bool,
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    """Read ``metadata.json`` written by ``deepswe_run.py --runtime in-sandbox``."""
    metadata_text = read_remote_file(sandbox, f"{run_logs_remote}/metadata.json")
    if metadata_text:
        metadata = json.loads(metadata_text)
        agent_result = metadata.get("agent")
        grade_result = metadata.get("grade") or {}
        if grade_only:
            agent_result = None
        return agent_result, grade_result

    reward_text = read_remote_file(sandbox, f"{LOGS_REMOTE}/verifier/reward.txt")
    reward: int | None = None
    if reward_text is not None:
        stripped = reward_text.strip()
        if stripped in {"0", "1"}:
            reward = int(stripped)
    model_patch = read_remote_file(sandbox, f"{LOGS_REMOTE}/artifacts/model.patch")
    grade_result = {
        "pass": reward == 1,
        "reward": reward,
        "model_patch_chars": len(model_patch) if model_patch else 0,
    }
    return None, grade_result


def run_deepswe_run_in_sandbox(
    image: modal.Image,
    *,
    command: str,
    malvin_argv: list[str],
    grade_only: bool,
    skip_grade: bool = False,
    cursor_secrets: list[modal.Secret],
    artifacts_dir: Path | None = None,
    timeout: int = 7200,
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    """Exec ``deepswe_run.py`` once in a Modal sandbox (agent + grade in one command)."""
    sandbox: modal.Sandbox | None = None
    run_logs_remote = f"{LOGS_REMOTE}/run"
    try:
        network = (
            sandbox_network_kwargs(cursor_api_only=False, block_all=True)
            if grade_only
            else agent_sandbox_network_kwargs(image)
        )
        resources = (
            grade_sandbox_resource_kwargs()
            if grade_only
            else agent_sandbox_resource_kwargs()
        )
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=image,
            workdir=APP_REMOTE,
            secrets=cursor_secrets if not grade_only else [],
            timeout=timeout,
            **resources,
            **network,
        )
        argv = [
            "python3",
            DEEPSWE_RUN_REMOTE,
            "run",
            "--task",
            TASK_REMOTE,
            "--workspace",
            APP_REMOTE,
            "--runtime",
            "in-sandbox",
            "--skip-materialize",
            "--results-dir",
            run_logs_remote,
        ]
        if grade_only:
            argv.append("--grade-only")
        else:
            if skip_grade:
                argv.append("--skip-grade")
            argv.extend(["--command", command, *malvin_argv])
        proc = sandbox.exec(
            *argv,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        click.echo("Running deepswe_run.py on Modal (single exec)...")
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
        agent_result, grade_result = parse_deepswe_run_result(
            sandbox,
            run_logs_remote=run_logs_remote,
            grade_only=grade_only,
        )
        if agent_result is None and not grade_only:
            agent_result = {"exit_code": int(proc.returncode or 0)}
        elif agent_result is not None and "exit_code" not in agent_result:
            agent_result["exit_code"] = int(proc.returncode or 0)
        if artifacts_dir is not None:
            artifacts_dir.mkdir(parents=True, exist_ok=True)
            model_patch = read_remote_file(sandbox, f"{LOGS_REMOTE}/artifacts/model.patch")
            if model_patch:
                (artifacts_dir / "model.patch").write_text(model_patch, encoding="utf-8")
            metadata_text = read_remote_file(sandbox, f"{run_logs_remote}/metadata.json")
            if metadata_text:
                (artifacts_dir / "metadata.json").write_text(metadata_text, encoding="utf-8")
            grade_result["harvest"] = harvest_sandbox_logs(sandbox, artifacts_dir)
        return agent_result, grade_result
    finally:
        if sandbox is not None:
            sandbox.terminate()


def harbor_agent_image(
    spec: Any,
    workspace: Path,
    tests_dir: Path,
    *,
    dockerfile: Path,
    malvin_repo: Path,
    kiss_repo: Path,
    deepswe_run_py: Path,
) -> modal.Image:
    """Harbor task image plus locally built malvin/kiss, cursor-agent, and deepswe_run."""
    base = harbor_image(spec, dockerfile=dockerfile)
    augmented = base.run_commands(
        "apt-get update -qq && apt-get install -y -qq curl build-essential pkg-config libssl-dev python3-pip",
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    )
    augmented = mount_local_toolchain(
        augmented,
        malvin_repo=malvin_repo,
        kiss_repo=kiss_repo,
    )
    return mount_eval_context(
        augmented,
        task_dir=spec.task_dir,
        workspace=workspace,
        tests_dir=tests_dir,
        deepswe_run_py=deepswe_run_py,
    )


def cursor_secrets() -> list[modal.Secret]:
    keys = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"]
    present = [k for k in keys if os.environ.get(k)]
    if not present:
        return []
    return [modal.Secret.from_local_environ(present)]


def write_metadata(out_dir: Path, payload: dict[str, Any]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "metadata.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def run_modal_eval(
    *,
    task_dir: Path,
    workspace: Path | None = None,
    results_dir: Path | None = None,
    malvin_command: str = "code",
    checks_override: str | None = None,
    grade_only: bool = False,
    skip_grade: bool = False,
    apply_solution: bool = False,
    reset_flag: bool = False,
    malvin_args: tuple[str, ...] = (),
    dry_run: bool = False,
) -> None:
    """Run agent + Harbor grade on Modal (library entry for ``deepswe_run solve``)."""
    spec = parse_task_dir(task_dir)
    results_root = results_dir or default_deepswe_results_dir()
    workspace = workspace or (results_root / spec.task_id / "workspace")
    run_root = results_root / spec.task_id / f"modal_{timestamp_dir()}"

    click.echo(f"Task: {spec.task_id}")
    click.echo("Runtime: modal")
    click.echo(f"Workspace: {workspace.resolve()}")
    click.echo(f"Artifacts: {run_root.resolve()}")

    if dry_run:
        click.echo("Dry run: would materialize workspace and exec deepswe_run on Modal")
        if grade_only:
            click.echo("Dry run: grade-only (block_network sandbox)")
        elif skip_grade:
            click.echo("Dry run: agent phase only (--skip-grade)")
        return

    materialize_workspace(spec, workspace, dry_run=False)
    if reset_flag or apply_solution:
        reset_workspace(spec, workspace, dry_run=False)
    if apply_solution:
        if spec.solution_patch is None:
            raise click.ClickException(f"No solution at {spec.task_dir / 'solution'}")
        apply_patch(workspace, spec.solution_patch, dry_run=False)

    agent_result: dict[str, Any] | None = None
    grade_result: dict[str, Any]
    deepswe_run_py = Path(__file__).resolve().parent / "deepswe_run.py"
    checks = checks_override
    if checks is None:
        checks = DEFAULT_CHECKS_CODE if malvin_command == "code" else DEFAULT_CHECKS_DO
    if not grade_only:
        malvin_repo, kiss_repo = validate_toolchain_repos()
        click.echo(f"malvin source: {malvin_repo.resolve()}")
        click.echo(f"kiss source: {kiss_repo.resolve()}")
        write_plan_and_checks(
            spec,
            workspace,
            command=malvin_command,
            checks_override=checks,
            dry_run=False,
        )
        combined = harbor_agent_image(
            spec,
            workspace,
            spec.tests_dir,
            dockerfile=spec.dockerfile,
            malvin_repo=malvin_repo,
            kiss_repo=kiss_repo,
            deepswe_run_py=deepswe_run_py,
        )
        agent_result, grade_result = run_deepswe_run_in_sandbox(
            combined,
            command=malvin_command,
            malvin_argv=list(malvin_args),
            grade_only=False,
            skip_grade=skip_grade,
            cursor_secrets=cursor_secrets(),
            artifacts_dir=run_root,
        )
    else:
        grade_img = mount_eval_context(
            harbor_image(spec, dockerfile=spec.dockerfile),
            task_dir=spec.task_dir,
            workspace=workspace,
            tests_dir=spec.tests_dir,
            deepswe_run_py=deepswe_run_py,
        )
        agent_result, grade_result = run_deepswe_run_in_sandbox(
            grade_img,
            command=malvin_command,
            malvin_argv=list(malvin_args),
            grade_only=True,
            skip_grade=False,
            cursor_secrets=[],
            artifacts_dir=run_root,
        )

    metadata = {
        "task_id": spec.task_id,
        "runtime": "modal",
        "workspace": str(workspace.resolve()),
        "malvin_command": malvin_command if not grade_only else None,
        "malvin_args": list(malvin_args),
        "agent": agent_result,
        "grade": grade_result,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
    }
    write_metadata(run_root, metadata)
    if grade_result.get("reward") is not None:
        (run_root / "reward.txt").write_text(f"{grade_result['reward']}\n", encoding="utf-8")

    click.echo("\n=== Modal evaluation ===")
    click.echo(f"reward: {grade_result.get('reward')}")
    click.echo(f"pass: {grade_result.get('pass')}")
    if agent_result:
        click.echo(f"malvin exit: {agent_result.get('exit_code')}")
    click.echo(f"artifacts: {run_root.resolve()}")

    if grade_result.get("pass") is False:
        raise SystemExit(1)
    if agent_result and agent_result.get("exit_code") not in (0, None):
        raise SystemExit(agent_result["exit_code"])


@click.command(
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.option(
    "--self-test",
    is_flag=True,
    help="Run local unit tests without Modal credentials.",
)
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--workspace",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--results-dir",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    show_default="~/.malvin/deepswe-results",
)
@click.option(
    "--command",
    "malvin_command",
    type=click.Choice(["code", "do"]),
    default="code",
    show_default=True,
)
@click.option(
    "--grade-only",
    is_flag=True,
    help="Skip agent; grade current workspace on Modal.",
)
@click.option(
    "--apply-solution",
    is_flag=True,
    help="Apply reference solution.patch before agent or grade.",
)
@click.option(
    "--reset",
    "reset_flag",
    is_flag=True,
    help="Hard reset workspace to base_commit before run.",
)
@click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def main(
    ctx: click.Context,
    self_test: bool,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    grade_only: bool,
    apply_solution: bool,
    reset_flag: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """DeepSWE evaluation on Modal (agent optional, Harbor grade in sandbox)."""
    if self_test:
        run_unit_tests()
        raise SystemExit(0)
    if task_dir is None:
        raise click.ClickException("--task is required unless --self-test is set")
    extra = tuple(ctx.args)
    if extra:
        malvin_args = malvin_args + extra
    run_modal_eval(
        task_dir=task_dir,
        workspace=workspace,
        results_dir=results_dir,
        malvin_command=malvin_command,
        grade_only=grade_only,
        apply_solution=apply_solution,
        reset_flag=reset_flag,
        malvin_args=malvin_args,
    )


@app.local_entrypoint()
def entrypoint(*arglist: str) -> None:
    """``modal run ops/deepswe_modal.py -- [OPTIONS] [-- MALVIN_ARGS]``."""
    main.main(args=list(arglist), prog_name="modal run ops/deepswe_modal.py", standalone_mode=True)


def _test_repo_roots() -> None:
    malvin_repo = malvin_repo_root()
    assert malvin_repo.name == "malvin"
    assert (malvin_repo / "ops" / "deepswe_modal.py").is_file()
    kiss_repo = kiss_repo_root()
    assert kiss_repo.name == "kiss"


def _test_default_deepswe_results_dir() -> None:
    results = default_deepswe_results_dir()
    malvin_repo = malvin_repo_root()
    assert results.is_absolute()
    assert malvin_repo not in results.parents
    assert results.name == "deepswe-results"


def _test_cursor_cidrs() -> None:
    cidrs = resolve_cursor_api_cidrs()
    assert cidrs
    for cidr in cidrs:
        assert "/" in cidr


def _test_modal_cidr_allowlist_ipv4_only() -> None:
    assert modal_cidr_allowlist(
        ["1.2.3.4/32", "2001:db8::1/128", "5.6.7.8/32"]
    ) == ["1.2.3.4/32", "5.6.7.8/32"]
    try:
        modal_cidr_allowlist(["2001:db8::1/128"])
    except click.ClickException as exc:
        assert "No IPv4" in str(exc)
    else:
        raise AssertionError("expected ClickException for IPv6-only list")


def _test_network_kwargs() -> None:
    blocked = sandbox_network_kwargs(cursor_api_only=False, block_all=True)
    assert blocked == {"block_network": True}
    with patch(f"{__name__}.resolve_cursor_api_cidrs", return_value=["1.2.3.4/32"]):
        allowed = sandbox_network_kwargs(cursor_api_only=True, block_all=False)
    assert allowed == {"cidr_allowlist": ["1.2.3.4/32"]}
    assert sandbox_network_kwargs(
        cursor_api_only=True,
        block_all=False,
        cidr_allowlist=["9.9.9.9/32"],
    ) == {"cidr_allowlist": ["9.9.9.9/32"]}
    assert sandbox_network_kwargs(cursor_api_only=False, block_all=False) == {}


def _test_sandbox_resource_kwargs() -> None:
    agent = agent_sandbox_resource_kwargs()
    assert agent == {"cpu": 2.0, "memory": 4096}
    grade = grade_sandbox_resource_kwargs()
    assert grade == {"cpu": 1.0, "memory": 2048}


def _test_agent_sandbox_cidrs_union() -> None:
    with patch(f"{__name__}.resolve_cursor_api_cidrs", return_value=["1.1.1.1/32"]):
        with patch(
            f"{__name__}.resolve_cursor_api_cidrs_in_modal_sandbox",
            return_value=["2.2.2.2/32", "3.3.3.3/32", "2001:db8::1/128"],
        ):
            with patch(
                f"{__name__}.resolve_cursor_api_cidrs_under_allowlist",
                side_effect=[
                    ["3.3.3.3/32", "4.4.4.4/32"],
                    ["4.4.4.4/32", "5.5.5.5/32"],
                    ["5.5.5.5/32"],
                ],
            ):
                cidrs = resolve_agent_sandbox_cidrs(fixpoint_rounds=3)
    assert cidrs == [
        "1.1.1.1/32",
        "2.2.2.2/32",
        "3.3.3.3/32",
        "4.4.4.4/32",
        "5.5.5.5/32",
    ]


def _test_agent_sandbox_network_kwargs() -> None:
    image = MagicMock()
    with patch(
        f"{__name__}.resolve_agent_sandbox_cidrs",
        return_value=["10.0.0.1/32"],
    ) as mock_resolve:
        kwargs = agent_sandbox_network_kwargs(image)
    mock_resolve.assert_called_once_with(image, timeout=300)
    assert kwargs == {"cidr_allowlist": ["10.0.0.1/32"]}


def _test_stream_helpers() -> None:
    sink = io.StringIO()
    relay_stream(iter(["alpha", "beta"]), sink)
    assert sink.getvalue() == "alphabeta"
    out = io.StringIO()
    err = io.StringIO()
    proc = MagicMock(stdout=iter(["out"]), stderr=iter(["err"]))
    stream_process_output(proc, out, err)
    assert out.getvalue() == "out"
    assert err.getvalue() == "err"


def _test_cursor_secrets() -> None:
    keys = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"]
    saved = {key: os.environ.pop(key, None) for key in keys}
    try:
        assert cursor_secrets() == []
        os.environ["CURSOR_API_KEY"] = "test-key"
        assert len(cursor_secrets()) == 1
    finally:
        for key, value in saved.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def _test_grade_in_sandbox_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    metadata = json.dumps({"grade": {"reward": 1, "pass": True}, "agent": None})
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = metadata
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        _agent, grade_result = run_deepswe_run_in_sandbox(
            image,
            command="code",
            malvin_argv=[],
            grade_only=True,
            cursor_secrets=[],
        )
    mock_create.assert_called_once()
    assert mock_create.call_args.kwargs["block_network"] is True
    assert mock_create.call_args.kwargs["cpu"] == GRADE_SANDBOX_CPU
    assert mock_create.call_args.kwargs["memory"] == GRADE_SANDBOX_MEMORY_MIB
    assert fake_sandbox.exec.call_count == 1
    exec_argv = fake_sandbox.exec.call_args.args
    assert exec_argv[0] == "python3"
    assert exec_argv[1] == DEEPSWE_RUN_REMOTE
    assert exec_argv[2] == "run"
    assert "--grade-only" in exec_argv
    assert grade_result["reward"] == 1
    fake_sandbox.terminate.assert_called_once()


def _test_agent_sandbox_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    metadata = json.dumps(
        {"agent": {"exit_code": 0}, "grade": {"reward": 0, "pass": False}}
    )
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = metadata
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        with patch(
            f"{__name__}.resolve_agent_sandbox_cidrs",
            return_value=["9.9.9.9/32"],
        ):
            agent_result, grade_result = run_deepswe_run_in_sandbox(
                image,
                command="code",
                malvin_argv=[],
                grade_only=False,
                cursor_secrets=[],
            )
    assert mock_create.call_args.kwargs["cidr_allowlist"] == ["9.9.9.9/32"]
    assert mock_create.call_args.kwargs["cpu"] == AGENT_SANDBOX_CPU
    assert mock_create.call_args.kwargs["memory"] == AGENT_SANDBOX_MEMORY_MIB
    assert "block_network" not in mock_create.call_args.kwargs
    assert fake_sandbox.exec.call_count == 1
    exec_argv = fake_sandbox.exec.call_args.args
    assert exec_argv[0] == "python3"
    assert "--runtime" in exec_argv
    assert "in-sandbox" in exec_argv
    assert agent_result["exit_code"] == 0
    assert grade_result["reward"] == 0
    fake_sandbox.terminate.assert_called_once()


def _test_mount_eval_context_recipe() -> None:
    malvin_repo = malvin_repo_root()
    recorder = _RecordingImage()
    deepswe_run_py = Path(__file__).resolve().parent / "deepswe_run.py"
    mount_eval_context(
        recorder,
        task_dir=malvin_repo,
        workspace=malvin_repo,
        tests_dir=malvin_repo / "tests",
        deepswe_run_py=deepswe_run_py,
    )
    uploads = [call for call in recorder.calls if call[0] == "add_local_dir"]
    assert len(uploads) == 3
    file_uploads = [call for call in recorder.calls if call[0] == "add_local_file"]
    assert len(file_uploads) == 1
    assert file_uploads[0][2]["remote_path"] == DEEPSWE_RUN_REMOTE
    pip_cmds = [call for call in recorder.calls if call[0] == "run_commands"]
    assert pip_cmds
    assert "click" in pip_cmds[0][1][0]


def _test_validate_toolchain_repos() -> None:
    validate_toolchain_repos()
    with tempfile.TemporaryDirectory() as tmp:
        missing = Path(tmp) / "empty"
        missing.mkdir()
        with patch.dict(os.environ, {"KISS_REPO": str(missing)}):
            try:
                validate_toolchain_repos()
            except click.ClickException as exc:
                assert "kiss repo not found" in str(exc)
            else:
                raise AssertionError("expected ClickException for missing kiss repo")


def _test_read_remote_file() -> None:
    sandbox = MagicMock()
    sandbox.open.return_value.__enter__.return_value.read.return_value = "payload"
    assert read_remote_file(sandbox, "/tmp/x") == "payload"
    sandbox.open.side_effect = OSError("missing")
    assert read_remote_file(sandbox, "/tmp/missing") is None


def _test_write_metadata() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        out_dir = Path(tmp) / "run"
        payload = {"task_id": "demo", "reward": 1}
        write_metadata(out_dir, payload)
        loaded = json.loads((out_dir / "metadata.json").read_text(encoding="utf-8"))
        assert loaded == payload


def _test_resolve_cursor_api_cidrs_mocked() -> None:
    def fake_getaddrinfo(host: str, port: int, *args: Any, **kwargs: Any) -> list[tuple]:
        if host == CURSOR_API_HOSTS[0]:
            return [(None, None, None, None, ("1.2.3.4", port))]
        if host == CURSOR_API_HOSTS[1]:
            return [
                (None, None, None, None, ("1.2.3.4", port)),
                (None, None, None, None, ("2001:db8::1", port)),
            ]
        raise socket.gaierror("no such host")

    with patch(f"{__name__}.socket.getaddrinfo", side_effect=fake_getaddrinfo):
        cidrs = resolve_cursor_api_cidrs()
    assert cidrs == ["1.2.3.4/32", "2001:db8::1/128"]

    def all_fail(*args: Any, **kwargs: Any) -> list[tuple]:
        raise socket.gaierror("offline")

    with patch(f"{__name__}.socket.getaddrinfo", side_effect=all_fail):
        try:
            resolve_cursor_api_cidrs()
        except click.ClickException as exc:
            assert "Could not resolve Cursor API hosts" in str(exc)
        else:
            raise AssertionError("expected ClickException when DNS fails")


def _test_upload_ignore_patterns() -> None:
    malvin_ignores = malvin_upload_ignore()
    kiss_ignores = kiss_upload_ignore()
    assert "target/" in malvin_ignores
    assert "target/" in kiss_ignores
    assert "reports/" in malvin_ignores
    assert ".malvin/" in malvin_ignores
    assert "Cargo.toml" not in malvin_ignores
    assert "src/" not in malvin_ignores


class _RecordingImage:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[Any, ...], dict[str, Any]]] = []

    def add_local_dir(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("add_local_dir", args, kwargs))
        return self

    def add_local_file(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("add_local_file", args, kwargs))
        return self

    def run_commands(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("run_commands", args, kwargs))
        return self

    def env(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("env", args, kwargs))
        return self


def _test_mount_local_toolchain_recipe() -> None:
    malvin_repo, kiss_repo = validate_toolchain_repos()
    recorder = _RecordingImage()
    mount_local_toolchain(recorder, malvin_repo=malvin_repo, kiss_repo=kiss_repo)
    uploads = [call for call in recorder.calls if call[0] == "add_local_dir"]
    assert len(uploads) == 2
    assert uploads[0][2]["remote_path"] == MALVIN_TOOLCHAIN_REMOTE
    assert uploads[0][2]["ignore"] == malvin_upload_ignore()
    assert uploads[1][2]["remote_path"] == KISS_TOOLCHAIN_REMOTE
    commands = next(call[1] for call in recorder.calls if call[0] == "run_commands")
    assert len(commands) == 4
    assert KISS_TOOLCHAIN_REMOTE in commands[0]
    assert MALVIN_TOOLCHAIN_REMOTE in commands[1]
    assert "cursor.com/install" in commands[2]
    env_calls = [call for call in recorder.calls if call[0] == "env"]
    assert env_calls[-1][1][0] == {"PATH": TOOLCHAIN_PATH}


def _test_docstring_normative_command() -> None:
    doc = __doc__ or ""
    assert "--command code" in doc
    assert "--background" not in doc
    assert "--max-loops 1" not in doc
    assert "Gate A" in doc
    assert "Gate B" in doc


def _test_sandbox_app() -> None:
    lookup_app = SimpleNamespace(app_id="lookup-id")
    module_app = SimpleNamespace(app_id="module-id")
    with patch(f"{__name__}.app", SimpleNamespace(app_id=None)):
        with patch.object(modal.App, "lookup", return_value=lookup_app) as mock_lookup:
            assert sandbox_app() is lookup_app
        mock_lookup.assert_called_once_with(APP_NAME, create_if_missing=True)
    with patch(f"{__name__}.app", module_app):
        assert sandbox_app() is module_app


def _test_self_test_flag() -> None:
    runner = CliRunner()
    with patch(f"{__name__}.run_unit_tests") as mock_tests:
        result = runner.invoke(main, ["--self-test"])
    assert result.exit_code == 0, result.output
    mock_tests.assert_called_once()


def run_unit_tests() -> None:
    """Local tests for deepswe_modal helpers (no Modal network)."""
    _test_repo_roots()
    _test_default_deepswe_results_dir()
    _test_cursor_cidrs()
    _test_modal_cidr_allowlist_ipv4_only()
    _test_network_kwargs()
    _test_sandbox_resource_kwargs()
    _test_agent_sandbox_cidrs_union()
    _test_agent_sandbox_network_kwargs()
    _test_stream_helpers()
    _test_cursor_secrets()
    _test_validate_toolchain_repos()
    _test_read_remote_file()
    _test_write_metadata()
    _test_resolve_cursor_api_cidrs_mocked()
    _test_upload_ignore_patterns()
    _test_mount_local_toolchain_recipe()
    _test_docstring_normative_command()
    _test_grade_in_sandbox_network()
    _test_agent_sandbox_network()
    _test_mount_eval_context_recipe()
    _test_sandbox_app()
    _test_self_test_flag()


if __name__ == "__main__":
    main()
