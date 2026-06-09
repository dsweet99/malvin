#!/usr/bin/env python3
"""Modal diagnostic: test allowlist built from observed open-egress peer IPs."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import app, cidr_probe_image, stream_process_output

PEER_SCRIPT = r"""
import json, socket, ssl

HOST = "api2.cursor.sh"
ctx = ssl.create_default_context()
seen = set()
for _ in range(15):
    try:
        with socket.create_connection((HOST, 443), 8) as sock:
            seen.add(sock.getpeername()[0])
            with ctx.wrap_socket(sock, server_hostname=HOST):
                pass
    except OSError:
        pass
print(json.dumps(sorted(seen)))
"""

HTTPS_SCRIPT = r"""
import json, urllib.request
try:
    with urllib.request.urlopen("https://api2.cursor.sh/", timeout=15) as r:
        print(json.dumps({"ok": True, "status": r.status}))
except Exception as e:
    print(json.dumps({"ok": False, "error": repr(e)}))
"""


@app.local_entrypoint(name="diagnose_cidr_observed")
def main() -> None:
    image = cidr_probe_image()
    open_sb = modal.Sandbox.create(app=app, image=image, timeout=180)
    try:
        proc = open_sb.exec(
            "python3", "-c", PEER_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        out = proc.stdout.read()
        proc.wait()
        peers = json.loads(out.strip())
        print(f"observed_peer_ips={peers}")
    finally:
        release_modal_sandbox(open_sb)

    cidrs = [f"{ip}/32" for ip in peers]
    print(f"allowlist_size={len(cidrs)}")

    allow_sb = modal.Sandbox.create(
        app=app, image=image, timeout=180, outbound_cidr_allowlist=cidrs,
    )
    try:
        proc = allow_sb.exec(
            "python3", "-c", HTTPS_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
    finally:
        release_modal_sandbox(allow_sb)


if __name__ == "__main__":
    main()
