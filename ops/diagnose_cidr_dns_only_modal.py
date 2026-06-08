#!/usr/bin/env python3
"""Fast diagnostic: DNS answers under allowlist vs seed set (no HTTPS)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from deepswe_modal import CURSOR_API_HOSTS, app, cidr_probe_image, stream_process_output

DNS_SCRIPT = """
import json, socket
HOSTS = {hosts!r}
out = {{}}
for host in HOSTS:
    ips = set()
    try:
        for info in socket.getaddrinfo(host, 443, type=socket.SOCK_STREAM):
            ip = info[4][0]
            if ":" not in ip:
                ips.add(ip)
    except socket.gaierror:
        pass
    out[host] = sorted(ips)
print(json.dumps(out))
""".format(hosts=list(CURSOR_API_HOSTS))


@app.local_entrypoint(name="diagnose_cidr_dns_only")
def main() -> None:
    image = cidr_probe_image()
    open_sb = modal.Sandbox.create(app=app, image=image, timeout=120)
    try:
        proc = open_sb.exec(
            "python3", "-c", DNS_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        open_out = proc.stdout.read()
        proc.wait()
        open_dns = json.loads(open_out.strip())
        seed = sorted(
            {f"{ip}/32" for ips in open_dns.values() for ip in ips}
        )
        print(f"open_dns_hosts={len(open_dns)} seed_cidrs={len(seed)}")
        print(json.dumps(open_dns, indent=2))
    finally:
        open_sb.terminate()

    print(f"=== ALLOWLIST dns-only ({len(seed)} CIDRs) ===")
    allow_sb = modal.Sandbox.create(
        app=app, image=image, timeout=120, cidr_allowlist=seed,
    )
    try:
        proc = allow_sb.exec(
            "python3", "-c", DNS_SCRIPT,
            stdout=StreamType.PIPE, stderr=StreamType.PIPE, text=True,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
        allow_out = proc.stdout.read() if hasattr(proc.stdout, "read") else ""
        if allow_out.strip():
            allow_dns = json.loads(allow_out.strip())
            allow_ips = {ip for ips in allow_dns.values() for ip in ips}
            seed_ips = {c.split("/")[0] for c in seed}
            extra = sorted(allow_ips - seed_ips)
            missing = sorted(seed_ips - allow_ips)
            print(f"allow_extra_ips={extra}")
            print(f"seed_not_in_allow_dns={missing}")
    finally:
        allow_sb.terminate()


if __name__ == "__main__":
    main()
