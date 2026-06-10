#!/usr/bin/env python3
"""Quick Modal probe: verify Cursor API HTTPS works under the agent CIDR allowlist."""

from __future__ import annotations

import sys
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kiss_coverage_common import register_kiss_static_symbols
from modal_sandbox_lifecycle import release_modal_sandbox
from deepswe_modal import agent_sandbox_network_kwargs, app, cidr_probe_image, stream_process_output


@app.local_entrypoint(name="probe_cidr_connectivity")
def main() -> None:
    image = cidr_probe_image()
    net_kwargs = agent_sandbox_network_kwargs(image)
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=app,
            image=image,
            timeout=180,
            **net_kwargs,
        )
        proc = sandbox.exec(
            "python3",
            "-c",
            (
                "import urllib.request\n"
                "try:\n"
                "    with urllib.request.urlopen('https://api2.cursor.sh/', timeout=15) as r:\n"
                "        print(f'http_code={r.status}')\n"
                "except Exception as e:\n"
                "    print(f'error={e!r}')\n"
                "    raise SystemExit(1)\n"
            ),
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        rc = proc.wait()
        if rc != 0:
            raise SystemExit(rc)
    finally:
        if sandbox is not None:
            release_modal_sandbox(sandbox)



def test_kiss_static_coverage() -> None:
    """Register production symbols for kiss static test coverage."""
    register_kiss_static_symbols(main)

if __name__ == "__main__":
    main()
