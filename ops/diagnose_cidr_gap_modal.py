#!/usr/bin/env python3
"""Time-gap diagnostic for CIDR allowlist staleness."""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import (
    RESOLVE_CURSOR_CIDRS_IN_SANDBOX_SCRIPT,
    app,
    cidr_probe_image,
    stream_process_output,
)

HTTPS = r"""
import json, urllib.request
try:
    with urllib.request.urlopen("https://api2.cursor.sh/", timeout=15) as r:
        print(json.dumps({"ok": True, "status": r.status}))
except Exception as e:
    print(json.dumps({"ok": False, "error": repr(e)}))
"""


def run_https(sandbox: modal.Sandbox) -> None:
    proc = sandbox.exec(
        "python3", "-c", HTTPS,
        stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
    )
    stream_process_output(proc, sys.stdout, sys.stderr)
    proc.wait()


@app.local_entrypoint(name="diagnose_cidr_gap")
def main() -> None:
    image = cidr_probe_image()
    t0 = time.time()
    probe = modal.Sandbox.create(app=app, image=image, timeout=600)
    try:
        proc = probe.exec(
            "python3", "-c", RESOLVE_CURSOR_CIDRS_IN_SANDBOX_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        out = proc.stdout.read()
        proc.wait()
        cidrs = json.loads(out.strip())
        print(f"probe_sec={time.time()-t0:.1f} cidrs={len(cidrs)}")
    finally:
        release_modal_sandbox(probe)

    allow = modal.Sandbox.create(
        app=app, image=image, timeout=180, outbound_cidr_allowlist=cidrs,
    )
    try:
        print(f"gap_sec={time.time()-t0:.1f}")
        run_https(allow)
    finally:
        release_modal_sandbox(allow)



if __name__ == "__main__":
    main()
