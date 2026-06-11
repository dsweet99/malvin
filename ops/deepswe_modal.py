#!/usr/bin/env python3
"""Run DeepSWE Harbor verifier (and optionally malvin agent) on Modal.

No local Docker required for grading: builds a Modal Image from the task
Harbor Dockerfile (or pulls the registry image) and execs ``deepswe_run.py``
once inside a sandbox (agent + grade in one command when not ``--grade-only``).

The default ``solve`` path runs malvin in a Modal sandbox with a Cursor API
``cidr_allowlist`` (no general internet egress), harvests the workspace, then grades
in a separate Modal sandbox with ``block_network=True``. Open egress remains
available via ``run_deepswe_run_in_sandbox(open_network=True)`` for diagnostics.
malvin and kiss are built from local source
trees (``MALVIN_REPO`` / ``KISS_REPO``) when an in-sandbox agent image is required.

Prerequisites: Modal CLI authenticated; Cursor API key in ``CURSOR_AGENT_API_KEY``,
``CURSOR_API_KEY``, or ``AGENT_API_KEY``; malvin repo at parent of ``ops/``; kiss at
``../kiss`` or ``KISS_REPO``; DeepSWE task at ``../deep-swe/tasks/...``.

For headless eval without Cursor credentials, pass ``--mini`` to malvin and set
``OPENROUTER_API_KEY`` in the sandbox environment (OpenRouter HTTP egress required).

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
import stat
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
    default_deepswe_tasks_root,
    find_latest_malvin_log,
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
    "marketplace.cursorapi.com",
)

# Agent sessions must reach these; geo/direct fallbacks are best-effort under the cap.
CRITICAL_CURSOR_API_HOSTS = ("api2.cursor.sh", "api.cursor.com")

# Modal Sandbox.create defaults are 0.125 CPU and 128 MiB — too small for malvin +
# cursor-agent. Match malvin's default mem_limit_gb (default_repo/config.toml).
AGENT_SANDBOX_CPU = 2.0
AGENT_SANDBOX_MEMORY_MIB = 4096
GRADE_SANDBOX_CPU = 1.0
GRADE_SANDBOX_MEMORY_MIB = 2048
MODAL_MAX_CIDR_ALLOWLIST = 100

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
CONNECTS_PER_HOST = 50
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

# TLS peer observation under a converged allowlist (after DNS fixpoint).
RESOLVE_CURSOR_CIDRS_UNDER_ALLOWLIST_SCRIPT = """
import json
import socket
import ssl

HOSTS = {hosts!r}
CONNECTS_PER_HOST = 15
CONNECT_TIMEOUT_SEC = 5


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
        "Could not observe Cursor API peer IPs inside allowlist sandbox"
    )
print(json.dumps(sorted(cidrs)))
""".format(
    hosts=list(CURSOR_API_HOSTS),
)

VALIDATE_CURSOR_HTTPS_UNDER_ALLOWLIST_SCRIPT = """
import json
import socket
import ssl
import urllib.request

HOSTS = {hosts!r}
CONNECTS_PER_HOST = 15
CONNECT_TIMEOUT_SEC = 8


def https_ok(host: str) -> bool:
    try:
        req = urllib.request.Request(f"https://{{host}}/", method="HEAD")
        with urllib.request.urlopen(req, timeout=CONNECT_TIMEOUT_SEC) as resp:
            return resp.status < 500
    except Exception:
        return False


def tls_peer_ips(host: str) -> set[str]:
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


failed: list[str] = []
extra_ips: set[str] = set()
for host in HOSTS:
    if not https_ok(host):
        failed.append(host)
        for ip in tls_peer_ips(host):
            if ":" not in ip:
                extra_ips.add(ip)
print(json.dumps({{"failed_hosts": failed, "extra_ips": sorted(extra_ips)}}))
""".format(
    hosts=list(CURSOR_API_HOSTS),
)

ALLOWLIST_CIDR_PROBE_TIMEOUT = 600

OBSERVE_AGENT_PEERS_SCRIPT = r"""
import json
import socket
import struct
import subprocess
import time


def hex_endpoint(raw: str) -> tuple[str, int]:
    addr_hex, port_hex = raw.split(":")
    return socket.inet_ntoa(struct.pack("<L", int(addr_hex, 16))), int(port_hex, 16)


def tcp_peers(pid: int) -> set[tuple[str, int]]:
    out: set[tuple[str, int]] = set()
    path = f"/proc/{pid}/net/tcp"
    try:
        with open(path, encoding="ascii") as handle:
            for line in handle.read().splitlines()[1:]:
                cols = line.split()
                if len(cols) < 4 or cols[3] != "01":
                    continue
                ip, port = hex_endpoint(cols[2])
                out.add((ip, port))
    except OSError:
        pass
    return out


proc = subprocess.Popen(
    ["cursor-agent", "--force", "--trust", "-p", "Hello"],
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
)
seen: set[str] = set()
for _ in range(40):
    if proc.poll() is not None:
        break
    for ip, _port in tcp_peers(proc.pid):
        seen.add(ip)
    time.sleep(0.25)
try:
    proc.communicate(timeout=30)
except subprocess.TimeoutExpired:
    proc.kill()
    proc.communicate()
print(json.dumps({"peer_ips": sorted(seen)}))
"""


def cidr_probe_image() -> modal.Image:
    """Minimal image for Modal egress DNS probes (same sandbox network as agent runs)."""
    return modal.Image.debian_slim(python_version="3.12")


def agent_peer_probe_image() -> modal.Image:
    """Minimal image with cursor-agent for live TCP peer observation."""
    return (
        modal.Image.debian_slim(python_version="3.12")
        .apt_install("curl")
        .run_commands(
            "curl -fsSL https://cursor.com/install | bash",
            "/root/.local/bin/agent --version || true",
        )
        .env({"PATH": "/root/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"})
    )


def _probe_sandbox_timeout(requested: int) -> int:
    """Sandbox lifetime must cover worst-case TLS peer probes under an allowlist."""
    return max(requested, ALLOWLIST_CIDR_PROBE_TIMEOUT)


def _run_modal_cidr_probe_script(
    script: str,
    *,
    cidr_allowlist: list[str] | None = None,
    timeout: int = ALLOWLIST_CIDR_PROBE_TIMEOUT,
    error_label: str,
    probe_image: modal.Image | None = None,
    secrets: list[modal.Secret] | None = None,
) -> list[str]:
    """Exec a probe script in a Modal sandbox and parse a JSON CIDR list from stdout."""
    image = probe_image or cidr_probe_image()
    sandbox: modal.Sandbox | None = None
    sandbox_timeout = _probe_sandbox_timeout(timeout)
    create_kwargs: dict[str, Any] = {
        "app": sandbox_app(),
        "image": image,
        "timeout": sandbox_timeout,
    }
    if cidr_allowlist is not None:
        create_kwargs["cidr_allowlist"] = modal_cidr_allowlist(cidr_allowlist)
    if secrets:
        create_kwargs["secrets"] = secrets
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
        try:
            proc.wait()
        except modal.exception.NotFoundError as exc:
            detail = (stderr or stdout or str(exc)).strip()
            raise click.ClickException(
                f"{error_label}: sandbox expired before probe finished"
                + (f": {detail}" if detail else "")
            ) from exc
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


def resolve_cursor_api_cidrs_under_allowlist_peers(
    seed_cidrs: list[str],
    *,
    timeout: int = ALLOWLIST_CIDR_PROBE_TIMEOUT,
) -> list[str]:
    """Observe TLS peer IPs reachable under a converged CIDR allowlist."""
    return _run_modal_cidr_probe_script(
        RESOLVE_CURSOR_CIDRS_UNDER_ALLOWLIST_SCRIPT,
        cidr_allowlist=seed_cidrs,
        timeout=timeout,
        error_label="Modal egress allowlist peer probe failed",
    )


def validate_cursor_https_under_allowlist(
    seed_cidrs: list[str],
    *,
    timeout: int = ALLOWLIST_CIDR_PROBE_TIMEOUT,
) -> dict[str, Any]:
    """Return failed Cursor API hosts and extra peer IPs under a CIDR allowlist."""
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=cidr_probe_image(),
            timeout=_probe_sandbox_timeout(timeout),
            cidr_allowlist=modal_cidr_allowlist(seed_cidrs),
        )
        proc = sandbox.exec(
            "python3",
            "-c",
            VALIDATE_CURSOR_HTTPS_UNDER_ALLOWLIST_SCRIPT,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            text=True,
        )
        stdout = proc.stdout.read()
        stderr = proc.stderr.read()
        try:
            proc.wait()
        except modal.exception.NotFoundError as exc:
            detail = (stderr or stdout or str(exc)).strip()
            raise click.ClickException(
                "Modal allowlist HTTPS validation: sandbox expired before probe finished"
                + (f": {detail}" if detail else "")
            ) from exc
        if proc.returncode != 0:
            detail = (stderr or stdout or "").strip()
            raise click.ClickException(
                "Modal allowlist HTTPS validation failed"
                + (f": {detail}" if detail else "")
            )
        payload = json.loads(stdout.strip() or "{}")
        if not isinstance(payload, dict):
            raise click.ClickException(
                "Modal allowlist HTTPS validation: invalid payload"
            )
        failed = payload.get("failed_hosts", [])
        extra = payload.get("extra_ips", [])
        if not isinstance(failed, list) or not isinstance(extra, list):
            raise click.ClickException(
                "Modal allowlist HTTPS validation: invalid payload fields"
            )
        return {"failed_hosts": failed, "extra_ips": extra}
    finally:
        if sandbox is not None:
            sandbox.terminate()


def resolve_cursor_api_cidrs_from_agent_peers(
    *,
    timeout: int = ALLOWLIST_CIDR_PROBE_TIMEOUT,
) -> list[str]:
    """Observe TCP peers during a live cursor-agent hello under open egress."""
    if not cursor_secrets():
        return []
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=agent_peer_probe_image(),
            secrets=cursor_secrets(),
            timeout=_probe_sandbox_timeout(timeout),
        )
        proc = sandbox.exec(
            "python3",
            "-c",
            OBSERVE_AGENT_PEERS_SCRIPT,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            text=True,
        )
        stdout = proc.stdout.read()
        stderr = proc.stderr.read()
        try:
            proc.wait()
        except modal.exception.NotFoundError as exc:
            detail = (stderr or stdout or str(exc)).strip()
            raise click.ClickException(
                "Modal agent peer probe: sandbox expired before probe finished"
                + (f": {detail}" if detail else "")
            ) from exc
        if stderr.strip():
            click.echo(stderr, err=True)
        payload = json.loads(stdout.strip() or "{}")
        peer_ips = payload.get("peer_ips", [])
        if not isinstance(peer_ips, list):
            if proc.returncode != 0:
                detail = (stderr or stdout or "").strip()
                raise click.ClickException(
                    "Modal agent peer probe failed"
                    + (f": {detail}" if detail else "")
                )
            raise click.ClickException("Modal agent peer probe: invalid payload")
        if proc.returncode != 0 and peer_ips:
            click.echo(
                f"Modal agent peer probe exit={proc.returncode} "
                f"but observed {len(peer_ips)} peer IP(s); merging into allowlist",
                err=True,
            )
        elif proc.returncode != 0:
            detail = (stderr or stdout or "").strip()
            raise click.ClickException(
                "Modal agent peer probe failed"
                + (f": {detail}" if detail else "")
            )
        return [f"{ip}/32" for ip in peer_ips if isinstance(ip, str) and ":" not in ip]
    finally:
        if sandbox is not None:
            sandbox.terminate()


def resolve_agent_session_peers_under_allowlist(
    seed_cidrs: list[str],
    *,
    timeout: int = ALLOWLIST_CIDR_PROBE_TIMEOUT,
) -> list[str]:
    """Observe TCP peers during cursor-agent hello under a converged CIDR allowlist."""
    if not cursor_secrets():
        return []
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=agent_peer_probe_image(),
            secrets=cursor_secrets(),
            timeout=_probe_sandbox_timeout(timeout),
            cidr_allowlist=modal_cidr_allowlist(seed_cidrs),
        )
        proc = sandbox.exec(
            "python3",
            "-c",
            OBSERVE_AGENT_PEERS_SCRIPT,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            text=True,
        )
        stdout = proc.stdout.read()
        stderr = proc.stderr.read()
        try:
            proc.wait()
        except modal.exception.NotFoundError as exc:
            detail = (stderr or stdout or str(exc)).strip()
            raise click.ClickException(
                "Modal agent peer probe under allowlist: sandbox expired before probe finished"
                + (f": {detail}" if detail else "")
            ) from exc
        if stderr.strip():
            click.echo(stderr, err=True)
        payload = json.loads(stdout.strip() or "{}")
        peer_ips = payload.get("peer_ips", [])
        if not isinstance(peer_ips, list):
            if proc.returncode != 0:
                detail = (stderr or stdout or "").strip()
                raise click.ClickException(
                    "Modal agent peer probe under allowlist failed"
                    + (f": {detail}" if detail else "")
                )
            raise click.ClickException(
                "Modal agent peer probe under allowlist: invalid payload"
            )
        if proc.returncode != 0 and peer_ips:
            click.echo(
                f"Allowlist agent peer probe exit={proc.returncode} "
                f"but observed {len(peer_ips)} peer IP(s); merging into allowlist",
                err=True,
            )
        elif proc.returncode != 0:
            detail = (stderr or stdout or "").strip()
            raise click.ClickException(
                "Modal agent peer probe under allowlist failed"
                + (f": {detail}" if detail else "")
            )
        return [f"{ip}/32" for ip in peer_ips if isinstance(ip, str) and ":" not in ip]
    finally:
        if sandbox is not None:
            sandbox.terminate()


def union_ipv4_cidrs(*cidr_lists: list[str]) -> list[str]:
    """Return sorted union of IPv4 /32 (or wider) CIDR strings."""
    return modal_cidr_allowlist(sorted({cidr for group in cidr_lists for cidr in group}))


def compress_ipv4_cidrs(
    cidrs: list[str],
    *,
    max_cidrs: int = MODAL_MAX_CIDR_ALLOWLIST,
) -> list[str]:
    """Merge co-located /32s into /24s so the list fits Modal's allowlist cap."""
    import ipaddress

    by_24: dict[str, list[str]] = {}
    wider: list[str] = []
    for cidr in cidrs:
        net = ipaddress.ip_network(cidr, strict=False)
        if net.prefixlen == 32:
            parent = str(ipaddress.ip_network(f"{net.network_address}/24", strict=False))
            by_24.setdefault(parent, []).append(cidr)
        else:
            wider.append(cidr)
    merged: set[str] = set(wider)
    for parent, _hosts in by_24.items():
        # Always widen /32 to /24 so DNS round-robin within the same subnet stays allowed.
        merged.add(parent)
    ordered = sorted(merged)
    if len(ordered) <= max_cidrs:
        return ordered
    # Still over cap: promote every /32 to its /24 parent.
    promoted: set[str] = set(wider)
    for parent, hosts in by_24.items():
        promoted.add(parent)
    ordered = sorted(promoted)
    if len(ordered) <= max_cidrs:
        return ordered
    # Still over cap: merge co-located /24 (and narrower) entries into /16 parents.
    by_16: dict[str, list[str]] = {}
    keep: set[str] = set()
    for cidr in ordered:
        net = ipaddress.ip_network(cidr, strict=False)
        if net.prefixlen <= 16:
            keep.add(cidr)
            continue
        parent16 = str(ipaddress.ip_network(f"{net.network_address}/16", strict=False))
        by_16.setdefault(parent16, []).append(cidr)
    merged16: set[str] = set(keep)
    for parent, members in by_16.items():
        merged16.add(parent if len(members) >= 2 else members[0])
    ordered = sorted(merged16)
    if len(ordered) <= max_cidrs:
        return ordered
    merged16 = set(keep)
    for parent in by_16:
        merged16.add(parent)
    ordered = sorted(merged16)
    if len(ordered) <= max_cidrs:
        return ordered
    # Still over cap: merge /16 and narrower into /8 parents.
    by_8: dict[str, list[str]] = {}
    keep8: set[str] = set()
    for cidr in ordered:
        net = ipaddress.ip_network(cidr, strict=False)
        if net.prefixlen <= 8:
            keep8.add(cidr)
            continue
        parent8 = str(ipaddress.ip_network(f"{net.network_address}/8", strict=False))
        by_8.setdefault(parent8, []).append(cidr)
    merged8: set[str] = set(keep8)
    for parent, members in by_8.items():
        merged8.add(parent if len(members) >= 2 else members[0])
    ordered = sorted(merged8)
    if len(ordered) <= max_cidrs:
        return ordered
    merged8 = set(keep8)
    for parent in by_8:
        merged8.add(parent)
    ordered = sorted(merged8)
    if len(ordered) > max_cidrs:
        raise click.ClickException(
            f"Cursor API CIDR allowlist has {len(ordered)} entries after /8 "
            f"compression; Modal allows at most {max_cidrs}"
        )
    return ordered


def allowlist_near_modal_cap(cidrs: list[str], *, headroom: int = 2) -> bool:
    """Return True when compressed allowlist is within ``headroom`` of Modal's cap."""
    try:
        return len(compress_ipv4_cidrs(cidrs)) >= MODAL_MAX_CIDR_ALLOWLIST - headroom
    except click.ClickException:
        return True


def _merge_allowlist_agent_peers(
    cidrs: list[str],
    *,
    timeout: int,
    max_rounds: int = 3,
    label: str = "allowlist agent peer",
) -> tuple[list[str], int]:
    """Observe cursor-agent TCP peers under the current allowlist and merge /32 CIDRs."""
    added_total = 0
    for _ in range(max_rounds):
        before = len(cidrs)
        try:
            seed = compress_ipv4_cidrs(cidrs)
            raw_peers = resolve_agent_session_peers_under_allowlist(
                seed, timeout=timeout
            )
            if not raw_peers:
                break
            cidrs = union_ipv4_cidrs(cidrs, modal_cidr_allowlist(raw_peers))
            added = len(cidrs) - before
            added_total += added
            if added == 0:
                break
            if allowlist_near_modal_cap(cidrs):
                break
        except click.ClickException as exc:
            click.echo(f"{label} probe skipped: {exc}", err=True)
            break
    return cidrs, added_total


def resolve_agent_sandbox_cidrs(
    image: modal.Image | None = None,
    *,
    timeout: int = 300,
    fixpoint_rounds: int = 8,
) -> list[str]:
    """Build agent allowlist: host DNS ∪ open Modal probe ∪ allowlist DNS fixpoint."""
    _ = image  # egress probes use cidr_probe_image(); agent image is irrelevant.
    host_cidrs = modal_cidr_allowlist(resolve_cursor_api_cidrs())
    open_modal_cidrs = modal_cidr_allowlist(
        resolve_cursor_api_cidrs_in_modal_sandbox(timeout=timeout)
    )
    agent_peer_added = 0
    agent_peer_cidrs = modal_cidr_allowlist(
        resolve_cursor_api_cidrs_from_agent_peers(timeout=timeout)
    )
    cidrs = union_ipv4_cidrs(host_cidrs, open_modal_cidrs, agent_peer_cidrs)
    agent_peer_added = len(agent_peer_cidrs)
    fixpoint_added = 0
    for _ in range(fixpoint_rounds):
        before = len(cidrs)
        allowlist_dns = modal_cidr_allowlist(
            resolve_cursor_api_cidrs_under_allowlist(
                compress_ipv4_cidrs(cidrs), timeout=timeout
            )
        )
        cidrs = union_ipv4_cidrs(cidrs, allowlist_dns)
        fixpoint_added += len(cidrs) - before
        if len(cidrs) == before:
            break
        if allowlist_near_modal_cap(cidrs):
            break
    peer_added = 0
    before_peers = len(cidrs)
    try:
        peer_cidrs = modal_cidr_allowlist(
            resolve_cursor_api_cidrs_under_allowlist_peers(
                compress_ipv4_cidrs(cidrs), timeout=timeout
            )
        )
        cidrs = union_ipv4_cidrs(cidrs, peer_cidrs)
        peer_added = len(cidrs) - before_peers
    except click.ClickException as exc:
        click.echo(f"Allowlist TLS peer probe skipped: {exc}", err=True)
    allowlist_agent_peer_added = 0
    https_validation_added = 0
    for _ in range(5):
        seed = compress_ipv4_cidrs(cidrs)
        validation = validate_cursor_https_under_allowlist(seed, timeout=timeout)
        failed_hosts = validation.get("failed_hosts", [])
        if not failed_hosts:
            break
        before = len(cidrs)
        extra_ips = validation.get("extra_ips", [])
        cidrs = union_ipv4_cidrs(cidrs, [f"{ip}/32" for ip in extra_ips if isinstance(ip, str)])
        try:
            peer_cidrs = modal_cidr_allowlist(
                resolve_cursor_api_cidrs_under_allowlist_peers(
                    compress_ipv4_cidrs(cidrs), timeout=timeout
                )
            )
            cidrs = union_ipv4_cidrs(cidrs, peer_cidrs)
        except click.ClickException as exc:
            click.echo(f"HTTPS validation peer probe skipped: {exc}", err=True)
        allowlist_dns = modal_cidr_allowlist(
            resolve_cursor_api_cidrs_under_allowlist(
                compress_ipv4_cidrs(cidrs), timeout=timeout
            )
        )
        cidrs = union_ipv4_cidrs(cidrs, allowlist_dns)
        https_validation_added += len(cidrs) - before
        click.echo(
            f"Allowlist HTTPS validation: {len(failed_hosts)} host(s) failed "
            f"({', '.join(str(h) for h in failed_hosts[:3])}"
            f"{'...' if len(failed_hosts) > 3 else ''}); "
            f"added {len(cidrs) - before} CIDR(s)",
            err=True,
        )
        if len(cidrs) == before:
            break
        if allowlist_near_modal_cap(cidrs):
            break
        post_round, added = _merge_allowlist_agent_peers(
            cidrs, timeout=timeout, max_rounds=1, label="HTTPS-round agent peer"
        )
        cidrs = post_round
        allowlist_agent_peer_added += added
    else:
        seed = compress_ipv4_cidrs(cidrs)
        validation = validate_cursor_https_under_allowlist(seed, timeout=timeout)
        failed_hosts = validation.get("failed_hosts", [])
        critical_failed = [
            host for host in failed_hosts if host in CRITICAL_CURSOR_API_HOSTS
        ]
        if critical_failed:
            raise click.ClickException(
                "Cursor API HTTPS unreachable under Modal CIDR allowlist after "
                f"validation rounds: {', '.join(str(h) for h in critical_failed)}"
            )
        if failed_hosts:
            click.echo(
                "Allowlist HTTPS validation: non-critical host(s) still unreachable "
                f"({', '.join(str(h) for h in failed_hosts[:5])}"
                f"{'...' if len(failed_hosts) > 5 else ''}); continuing with cap",
                err=True,
            )
    cidrs, post_https_agent_peers = _merge_allowlist_agent_peers(
        cidrs, timeout=timeout, max_rounds=3, label="Post-HTTPS agent peer"
    )
    allowlist_agent_peer_added += post_https_agent_peers
    seed = compress_ipv4_cidrs(cidrs)
    final_validation = validate_cursor_https_under_allowlist(seed, timeout=timeout)
    critical_failed = [
        host
        for host in final_validation.get("failed_hosts", [])
        if host in CRITICAL_CURSOR_API_HOSTS
    ]
    if critical_failed:
        raise click.ClickException(
            "Cursor API HTTPS unreachable under Modal CIDR allowlist after "
            f"post-HTTPS agent peer merge: {', '.join(str(h) for h in critical_failed)}"
        )
    compressed = compress_ipv4_cidrs(cidrs)
    click.echo(
        f"Cursor API allowlist: {len(compressed)} IPv4 CIDRs "
        f"(raw={len(cidrs)}, host={len(host_cidrs)}, open_modal={len(open_modal_cidrs)}, "
        f"agent_session_peers={agent_peer_added}, "
        f"allowlist_dns_fixpoint=+{fixpoint_added}, allowlist_peer=+{peer_added}, "
        f"allowlist_agent_peers=+{allowlist_agent_peer_added}, "
        f"https_validation=+{https_validation_added})"
    )
    return compressed


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


# Root-owned ``.git`` objects from the Modal sandbox cannot overwrite host checkout files.
HARVEST_WORKSPACE_TAR_EXCLUDES = (
    "--exclude=./.git",
    "--exclude=./.malvin/logs",
)


def _extract_tar_over_workspace(archive: tarfile.TarFile, workspace: Path) -> None:
    """Extract agent workspace tarball without root-owned permission clashes on reuse."""
    for member in archive.getmembers():
        if member.isdir():
            continue
        target = workspace / member.name
        if not target.is_file():
            continue
        if os.access(target, os.W_OK):
            continue
        try:
            target.chmod(target.stat().st_mode | stat.S_IWUSR)
        except OSError:
            target.unlink(missing_ok=True)
    archive.extractall(workspace)


def harvest_sandbox_workspace(sandbox: modal.Sandbox, workspace: Path) -> dict[str, Any]:
    """Copy remote ``/app`` from a Modal sandbox into local ``workspace``."""
    excludes = " ".join(HARVEST_WORKSPACE_TAR_EXCLUDES)
    prep = sandbox.exec(
        "bash",
        "-lc",
        f"tar -czf /tmp/deepswe_workspace.tgz -C {APP_REMOTE} {excludes} .",
        workdir=APP_REMOTE,
    )
    prep.wait()
    blob = read_remote_bytes(sandbox, "/tmp/deepswe_workspace.tgz")
    if not blob:
        return {"harvested": False}
    workspace.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as archive:
        _extract_tar_over_workspace(archive, workspace)
    return {"harvested": True, "workspace": str(workspace.resolve())}


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
    open_network: bool = False,
    cursor_secrets: list[modal.Secret],
    artifacts_dir: Path | None = None,
    harvest_workspace: Path | None = None,
    timeout: int = 7200,
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    """Exec ``deepswe_run.py`` once in a Modal sandbox (agent + grade in one command)."""
    sandbox: modal.Sandbox | None = None
    run_logs_remote = f"{LOGS_REMOTE}/run"
    try:
        if grade_only:
            network = sandbox_network_kwargs(cursor_api_only=False, block_all=True)
        elif open_network:
            network = sandbox_network_kwargs(cursor_api_only=False, block_all=False)
        else:
            network = agent_sandbox_network_kwargs(image)
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
        if harvest_workspace is not None:
            harvest_info = harvest_sandbox_workspace(sandbox, harvest_workspace)
            if agent_result is not None:
                agent_result["workspace_harvest"] = harvest_info
            elif not grade_only:
                agent_result = {"workspace_harvest": harvest_info}
        if artifacts_dir is not None:
            artifacts_dir.mkdir(parents=True, exist_ok=True)
            model_patch = read_remote_file(sandbox, f"{LOGS_REMOTE}/artifacts/model.patch")
            if model_patch:
                (artifacts_dir / "model.patch").write_text(model_patch, encoding="utf-8")
            metadata_text = read_remote_file(sandbox, f"{run_logs_remote}/metadata.json")
            if metadata_text:
                (artifacts_dir / "metadata.json").write_text(metadata_text, encoding="utf-8")
            harvest = harvest_sandbox_logs(sandbox, artifacts_dir)
            if grade_only:
                grade_result["harvest"] = harvest
            elif agent_result is not None:
                agent_result["harvest"] = harvest
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
        click.echo("Dry run: would materialize workspace")
        if grade_only:
            click.echo("Dry run: grade-only on Modal (block_network sandbox)")
        else:
            click.echo("Dry run: malvin agent in Modal sandbox (Cursor API allowlist)")
            if not skip_grade:
                click.echo("Dry run: Harbor grade in separate Modal sandbox (block_network)")
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
    if grade_only:
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
    else:
        write_plan_and_checks(
            spec,
            workspace,
            command=malvin_command,
            checks_override=checks,
            dry_run=False,
        )
        malvin_repo, kiss_repo = validate_toolchain_repos()
        agent_img = harbor_agent_image(
            spec,
            workspace,
            spec.tests_dir,
            dockerfile=spec.dockerfile,
            malvin_repo=malvin_repo,
            kiss_repo=kiss_repo,
            deepswe_run_py=deepswe_run_py,
        )
        agent_artifacts = run_root / "agent_sandbox"
        click.echo("Running malvin agent in Modal sandbox (Cursor API allowlist)...")
        agent_result, _ = run_deepswe_run_in_sandbox(
            agent_img,
            command=malvin_command,
            malvin_argv=list(malvin_args),
            grade_only=False,
            skip_grade=True,
            cursor_secrets=cursor_secrets(),
            artifacts_dir=agent_artifacts,
            harvest_workspace=workspace,
        )
        if agent_result is None:
            agent_result = {"exit_code": 1}
        agent_result["runtime"] = "modal-agent-sandbox"
        if skip_grade:
            grade_result = {"pass": None, "reward": None, "skipped": True}
        else:
            grade_img = mount_eval_context(
                harbor_image(spec, dockerfile=spec.dockerfile),
                task_dir=spec.task_dir,
                workspace=workspace,
                tests_dir=spec.tests_dir,
                deepswe_run_py=deepswe_run_py,
            )
            click.echo("Running Harbor verifier in Modal sandbox (block_network)...")
            _, grade_result = run_deepswe_run_in_sandbox(
                grade_img,
                command=malvin_command,
                malvin_argv=list(malvin_args),
                grade_only=True,
                skip_grade=False,
                cursor_secrets=[],
                artifacts_dir=run_root,
            )
            grade_result["runtime"] = "modal-grade-sandbox"

    malvin_log = find_latest_malvin_log(workspace)
    metadata = {
        "task_id": spec.task_id,
        "runtime": "modal",
        "workspace": str(workspace.resolve()),
        "malvin_command": malvin_command if not grade_only else None,
        "malvin_args": list(malvin_args),
        "agent": agent_result,
        "grade": grade_result,
        "malvin_log_dir": str(malvin_log.resolve()) if malvin_log else None,
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


def _test_compress_ipv4_cidrs() -> None:
    merged = compress_ipv4_cidrs(
        ["18.204.92.138/32", "18.204.92.140/32", "3.2.3.4/32"]
    )
    assert merged == ["18.204.92.0/24", "3.2.3.0/24"]
    assert len(compress_ipv4_cidrs([f"10.0.{i}.1/32" for i in range(50)])) == 50
    many_24 = [f"18.{i // 256}.{i % 256}.0/24" for i in range(102)]
    assert len(compress_ipv4_cidrs(many_24)) <= MODAL_MAX_CIDR_ALLOWLIST
    # /8 promotion when /16 merge still exceeds Modal's cap.
    crowded = [f"10.{i // 256}.{i % 256}.0/24" for i in range(120)]
    assert len(compress_ipv4_cidrs(crowded)) <= MODAL_MAX_CIDR_ALLOWLIST


def _test_allowlist_near_modal_cap() -> None:
    assert not allowlist_near_modal_cap(["1.1.1.1/32"])
    crowded = [f"{i}.0.0.0/16" for i in range(1, 106)]
    assert allowlist_near_modal_cap(crowded)


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
                with patch(
                    f"{__name__}.resolve_cursor_api_cidrs_under_allowlist_peers",
                    return_value=["6.6.6.6/32"],
                ):
                    with patch(
                        f"{__name__}.resolve_cursor_api_cidrs_from_agent_peers",
                        return_value=["7.7.7.7/32"],
                    ):
                        with patch(
                            f"{__name__}.resolve_agent_session_peers_under_allowlist",
                            return_value=[],
                        ):
                            with patch(
                                f"{__name__}.validate_cursor_https_under_allowlist",
                                return_value={"failed_hosts": [], "extra_ips": []},
                            ):
                                cidrs = resolve_agent_sandbox_cidrs(fixpoint_rounds=3)
    assert cidrs == [
        "1.1.1.0/24",
        "2.2.2.0/24",
        "3.3.3.0/24",
        "4.4.4.0/24",
        "5.5.5.0/24",
        "6.6.6.0/24",
        "7.7.7.0/24",
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


def _test_agent_sandbox_open_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    metadata = json.dumps({"agent": {"exit_code": 0}, "grade": {"skipped": True}})
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = metadata
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        agent_result, _grade = run_deepswe_run_in_sandbox(
            image,
            command="code",
            malvin_argv=[],
            grade_only=False,
            skip_grade=True,
            open_network=True,
            cursor_secrets=[],
        )
    assert "cidr_allowlist" not in mock_create.call_args.kwargs
    assert "block_network" not in mock_create.call_args.kwargs
    assert agent_result["exit_code"] == 0


def _test_harvest_sandbox_workspace() -> None:
    fake_proc = MagicMock()
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    payload = b"fake tarball"
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = payload
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp) / "workspace"
        with patch(f"{__name__}.read_remote_bytes", return_value=payload):
            with patch("tarfile.open") as mock_tar:
                mock_archive = MagicMock()
                mock_tar.return_value.__enter__.return_value = mock_archive
                result = harvest_sandbox_workspace(fake_sandbox, workspace)
    assert result["harvested"] is True
    mock_archive.extractall.assert_called_once_with(workspace)
    exec_cmd = fake_sandbox.exec.call_args.args[2]
    assert "--exclude=./.git" in exec_cmd


def _test_extract_tar_over_workspace_readonly_target() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        readonly = workspace / "locked.py"
        readonly.write_text("old\n", encoding="utf-8")
        readonly.chmod(stat.S_IRUSR)
        buf = io.BytesIO()
        with tarfile.open(fileobj=buf, mode="w:gz") as archive:
            data = b"new\n"
            info = tarfile.TarInfo(name="locked.py")
            info.size = len(data)
            info.mode = 0o644
            info.uid = 0
            info.gid = 0
            archive.addfile(info, io.BytesIO(data))
        buf.seek(0)
        with tarfile.open(fileobj=buf, mode="r:gz") as archive:
            _extract_tar_over_workspace(archive, workspace)
        assert readonly.read_text(encoding="utf-8") == "new\n"


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


def _test_probe_sandbox_timeout() -> None:
    assert _probe_sandbox_timeout(120) == ALLOWLIST_CIDR_PROBE_TIMEOUT
    assert _probe_sandbox_timeout(900) == 900


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


def _test_run_modal_eval_modal_agent_modal_grade() -> None:
    """solve path: malvin in Cursor-allowlist Modal sandbox, Harbor grade in block_network sandbox."""
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp) / "workspace"
        workspace.mkdir()
        (workspace / "plan.md").write_text("task\n", encoding="utf-8")
        results = Path(tmp) / "results"
        fake_agent = {"exit_code": 0, "agent_seconds": 1.0}
        fake_grade = {"pass": True, "reward": 1}
        with (
            patch(f"{__name__}.materialize_workspace"),
            patch(f"{__name__}.write_plan_and_checks"),
            patch(f"{__name__}.validate_toolchain_repos", return_value=(Path("/m"), Path("/k"))),
            patch(f"{__name__}.harbor_agent_image", return_value=MagicMock()),
            patch(f"{__name__}.harbor_image", return_value=MagicMock()),
            patch(f"{__name__}.mount_eval_context", return_value=MagicMock()),
            patch(
                f"{__name__}.run_deepswe_run_in_sandbox",
                side_effect=[(fake_agent, {"skipped": True}), (None, fake_grade)],
            ) as mock_sandbox,
            patch(f"{__name__}.cursor_secrets", return_value=[MagicMock()]),
            patch(f"{__name__}.find_latest_malvin_log", return_value=None),
        ):
            run_modal_eval(
                task_dir=task,
                workspace=workspace,
                results_dir=results,
                malvin_command="code",
                grade_only=False,
                skip_grade=False,
                dry_run=False,
            )
        assert mock_sandbox.call_count == 2
        agent_call, grade_call = mock_sandbox.call_args_list
        assert agent_call.kwargs["grade_only"] is False
        assert agent_call.kwargs["skip_grade"] is True
        assert agent_call.kwargs.get("open_network") is not True
        assert agent_call.kwargs["harvest_workspace"] == workspace
        assert grade_call.kwargs["grade_only"] is True
        assert grade_call.kwargs["cursor_secrets"] == []
        reward_files = list(results.rglob("reward.txt"))
        assert reward_files, results
        assert reward_files[0].read_text(encoding="utf-8").strip() == "1"


def run_unit_tests() -> None:
    """Local tests for deepswe_modal helpers (no Modal network)."""
    _test_repo_roots()
    _test_default_deepswe_results_dir()
    _test_cursor_cidrs()
    _test_modal_cidr_allowlist_ipv4_only()
    _test_network_kwargs()
    _test_sandbox_resource_kwargs()
    _test_compress_ipv4_cidrs()
    _test_allowlist_near_modal_cap()
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
    _test_agent_sandbox_open_network()
    _test_harvest_sandbox_workspace()
    _test_extract_tar_over_workspace_readonly_target()
    _test_mount_eval_context_recipe()
    _test_probe_sandbox_timeout()
    _test_sandbox_app()
    _test_self_test_flag()
    _test_run_modal_eval_modal_agent_modal_grade()


if __name__ == "__main__":
    main()
