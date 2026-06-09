#!/usr/bin/env python3
"""Modal diagnostic: compare open vs CIDR-allowlist egress to api2.cursor.sh."""

from __future__ import annotations

import sys
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import (
    app,
    cidr_probe_image,
    resolve_agent_sandbox_cidrs,
    stream_process_output,
)

DIAG_SCRIPT = r"""
import json, socket, ssl, urllib.request

HOST = "api2.cursor.sh"

def peer_ips(n=20):
    seen = set()
    ctx = ssl.create_default_context()
    for _ in range(n):
        try:
            with socket.create_connection((HOST, 443), 8) as sock:
                seen.add(sock.getpeername()[0])
                with ctx.wrap_socket(sock, server_hostname=HOST) as tls:
                    seen.add(tls.getpeername()[0])
        except OSError:
            pass
    return sorted(seen)

def dns_ips():
    out = set()
    try:
        for info in socket.getaddrinfo(HOST, 443, type=socket.SOCK_STREAM):
            out.add(info[4][0])
    except socket.gaierror:
        pass
    return sorted(out)

def https_ok():
    try:
        with urllib.request.urlopen(f"https://{HOST}/", timeout=15) as r:
            return {"ok": True, "status": r.status}
    except Exception as e:
        return {"ok": False, "error": repr(e)}

print(json.dumps({
    "dns_ips": dns_ips(),
    "peer_ips": peer_ips(),
    "https": https_ok(),
}))
"""


@app.local_entrypoint(name="diagnose_cidr")
def main() -> None:
    image = cidr_probe_image()
    print("=== OPEN sandbox ===")
    open_sandbox = modal.Sandbox.create(app=app, image=image, timeout=180)
    try:
        proc = open_sandbox.exec(
            "python3", "-c", DIAG_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
    finally:
        release_modal_sandbox(open_sandbox)

    cidrs = resolve_agent_sandbox_cidrs(image)
    print(f"=== ALLOWLIST sandbox ({len(cidrs)} IPv4 CIDRs) ===")
    allow_sandbox = modal.Sandbox.create(
        app=app, image=image, timeout=180,
        **{"outbound_cidr_allowlist": cidrs},
    )
    try:
        proc = allow_sandbox.exec(
            "python3", "-c", DIAG_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
    finally:
        release_modal_sandbox(allow_sandbox)


if __name__ == "__main__":
    main()
