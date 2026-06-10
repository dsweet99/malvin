#!/usr/bin/env python3
"""Diagnostic: compare DNS vs peer IPs under open vs allowlist sandboxes."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kiss_coverage_common import register_kiss_static_symbols
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import app, cidr_probe_image, stream_process_output

SCRIPT = r"""
import json, socket, ssl, urllib.request

HOST = "api2.cursor.sh"

def dns_ips():
    out = set()
    try:
        for info in socket.getaddrinfo(HOST, 443, type=socket.SOCK_STREAM):
            out.add(info[4][0])
    except socket.gaierror:
        pass
    return sorted(out)

def peer_ips(n=20):
    seen = set()
    ctx = ssl.create_default_context()
    for _ in range(n):
        try:
            with socket.create_connection((HOST, 443), 8) as sock:
                seen.add(sock.getpeername()[0])
                with ctx.wrap_socket(sock, server_hostname=HOST):
                    pass
        except OSError:
            pass
    return sorted(seen)

def https_ok():
    try:
        with urllib.request.urlopen(f"https://{HOST}/", timeout=15) as r:
            return {"ok": True, "status": r.status}
    except Exception as e:
        return {"ok": False, "error": repr(e)}

print(json.dumps({"dns_ips": dns_ips(), "peer_ips": peer_ips(), "https": https_ok()}))
"""


@app.local_entrypoint(name="diagnose_cidr_dns")
def main() -> None:
    image = cidr_probe_image()
    open_sb = modal.Sandbox.create(app=app, image=image, timeout=180)
    try:
        proc = open_sb.exec(
            "python3", "-c", SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        out = proc.stdout.read()
        proc.wait()
        data = json.loads(out.strip())
        print("=== OPEN ===")
        print(json.dumps(data, indent=2))
        seed = sorted({f"{ip}/32" for ip in data["dns_ips"] + data["peer_ips"] if ":" not in ip})
    finally:
        release_modal_sandbox(open_sb)

    print(f"=== ALLOWLIST ({len(seed)} CIDRs from open DNS+peers) ===")
    allow_sb = modal.Sandbox.create(
        app=app, image=image, timeout=180, outbound_cidr_allowlist=seed,
    )
    try:
        proc = allow_sb.exec(
            "python3", "-c", SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
    finally:
        release_modal_sandbox(allow_sb)



def test_kiss_static_coverage() -> None:
    """Register production symbols for kiss static test coverage."""
    register_kiss_static_symbols(main)

if __name__ == "__main__":
    main()
