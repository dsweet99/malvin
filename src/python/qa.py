"""Cursor SDK shutdown QA scenarios (library; Click lives in ``ops/qa.py``).

Regression check for the stdin-hold + SIGKILL abandonment path found
2026-08-07. Exit 0 means FIXED, exit 1 means STILL_BROKEN (or setup failed).
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[2]


def log(msg: str) -> None:
    print(msg, flush=True)


def repo_bridge_js() -> Path:
    return (REPO_ROOT / "cursor-sdk-bridge" / "dist" / "bridge.js").resolve()


def resolve_node_bin() -> Path:
    env = os.environ.get("MALVIN_NODE", "").strip()
    if env:
        p = Path(env)
        if p.is_file():
            return p
        raise FileNotFoundError(f"MALVIN_NODE is set but not a file: {p}")
    sticky = Path.home() / ".malvin_home" / "node_bin"
    if sticky.is_file():
        p = Path(sticky.read_text(encoding="utf-8").strip())
        if p.is_file():
            return p
    for candidate in (
        Path.home() / ".local/share/prime-agent-node/current/bin/node",
        Path("/usr/bin/node"),
    ):
        if candidate.is_file():
            return candidate
    which = subprocess.run(
        ["bash", "-lc", "command -v node"],
        check=False,
        capture_output=True,
        text=True,
    )
    if which.returncode == 0 and which.stdout.strip():
        return Path(which.stdout.strip())
    raise FileNotFoundError("Node >= 22 required for cursor-sdk-bridge")


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def ps_line(pid: int) -> str | None:
    try:
        return subprocess.check_output(
            ["ps", "-o", "pid=,ppid=,pgid=,stat=,cmd=", "-p", str(pid)],
            text=True,
        ).strip() or None
    except subprocess.CalledProcessError:
        return None


def _emit_result(name: str, fixed: bool, detail: dict[str, Any]) -> int:
    status = "FIXED" if fixed else "STILL_BROKEN"
    payload = {"scenario": name, "status": status, **detail}
    log(f"RESULT_JSON {json.dumps(payload)}")
    log(f"RESULT {status}: {name}")
    return 0 if fixed else 1


def _ppid_of(pid: int) -> int | None:
    try:
        return int(Path(f"/proc/{pid}/stat").read_text(encoding="utf-8").split()[3])
    except (FileNotFoundError, IndexError, ValueError, PermissionError):
        return None


def _write_owner_script(path: Path) -> None:
    """Nested owner that holds bridge stdin open until SIGKILL (malvin stand-in)."""
    path.write_text(
        """\
import json
import os
import subprocess
import sys
import time
from pathlib import Path

status_path, node, bridge, cwd = sys.argv[1:5]
proc = subprocess.Popen(
    [node, "--enable-source-maps", bridge],
    cwd=cwd,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
    bufsize=1,
    start_new_session=True,
)
time.sleep(0.5)
if proc.poll() is not None:
    Path(status_path).write_text(
        json.dumps({"error": f"bridge exited early code={proc.returncode}"})
    )
    raise SystemExit(1)
assert proc.stdin is not None
Path(status_path).write_text(
    json.dumps(
        {
            "owner_pid": os.getpid(),
            "bridge_pid": proc.pid,
            "stdin_write_fd": proc.stdin.fileno(),
            "bridge_pgid": os.getpgid(proc.pid),
            "owner_pgid": os.getpgid(0),
        }
    )
)
while True:
    time.sleep(60)
""",
        encoding="utf-8",
    )


def _spawn_bridge_owner(work: Path) -> tuple[subprocess.Popen[bytes], dict[str, Any]]:
    """Start owner+bridge; return owner Popen and status dict."""
    node = resolve_node_bin()
    bridge = repo_bridge_js()
    if not bridge.is_file():
        raise FileNotFoundError(
            f"missing {bridge}; run `npm ci && npm run build` in cursor-sdk-bridge/"
        )
    status_path = work / "owner_status.json"
    if status_path.exists():
        status_path.unlink()
    owner_py = work / "bridge_owner.py"
    _write_owner_script(owner_py)
    owner = subprocess.Popen(
        [sys.executable, str(owner_py), str(status_path), str(node), str(bridge), str(work)],
    )
    deadline = time.time() + 30.0
    while time.time() < deadline:
        if status_path.exists() and status_path.stat().st_size > 20:
            break
        if owner.poll() is not None:
            raise RuntimeError(f"owner exited before status file (code={owner.returncode})")
        time.sleep(0.05)
    else:
        owner.kill()
        raise RuntimeError("timeout waiting for owner status file")
    status = json.loads(status_path.read_text(encoding="utf-8"))
    if "error" in status:
        raise RuntimeError(status["error"])
    return owner, status


def _poll_bridge(bridge_pid: int, checkpoints: list[float]) -> list[dict[str, Any]]:
    """Poll bridge liveness/PPID at absolute times from now (seconds)."""
    t0 = time.time()
    samples: list[dict[str, Any]] = []
    for wait in checkpoints:
        while time.time() - t0 < wait:
            time.sleep(0.05)
        alive = pid_alive(bridge_pid)
        samples.append(
            {
                "t": wait,
                "alive": alive,
                "ppid": _ppid_of(bridge_pid) if alive else None,
                "ps": ps_line(bridge_pid) if alive else None,
            }
        )
    return samples


def repro_sigkill_stdin_hold_abandons_bridge() -> int:
    """SIGKILL parent while stdin write-end is held must not abandon bridge.

    Regression for the OS-level abandonment found 2026-08-07: duplicating the parent's
    bridge-stdin write-end via ``/proc/<owner>/fd/<n>``, then SIGKILL of the owner,
    used to leave cursor-sdk-bridge alive under PPID=1. Plain SIGKILL (no hold) EOFs
    stdin and the bridge exits. The fix is an early parent-death watch in the bridge
    (plus ``PR_SET_PDEATHSIG`` on malvin-spawned children).

    STILL_BROKEN = abandonment reproduced (problem still present).
    FIXED = bridge does not survive the held-stdin SIGKILL path.
    """
    name = "sigkill-stdin-hold-abandons-bridge"
    work = Path(tempfile.mkdtemp(prefix="malvin_qa_s6_"))
    control_dir = work / "control"
    hold_dir = work / "hold"
    control_dir.mkdir()
    hold_dir.mkdir()
    owner: subprocess.Popen[bytes] | None = None
    keeper_fd: int | None = None
    bridge_pid: int | None = None
    try:
        # Control: no stdin hold → bridge must die quickly (EOF path).
        ctrl_owner, ctrl = _spawn_bridge_owner(control_dir)
        ctrl_bridge = int(ctrl["bridge_pid"])
        os.kill(int(ctrl["owner_pid"]), signal.SIGKILL)
        try:
            ctrl_owner.wait(timeout=3)
        except subprocess.TimeoutExpired:
            pass
        ctrl_samples = _poll_bridge(ctrl_bridge, [0.5, 1.0])
        ctrl_dead = not any(s["alive"] for s in ctrl_samples)
        log(f"control_no_hold samples={ctrl_samples} dead={ctrl_dead}")

        # Repro: hold stdin write-end, SIGKILL owner → bridge under PPID=1.
        owner, st = _spawn_bridge_owner(hold_dir)
        bridge_pid = int(st["bridge_pid"])
        owner_pid = int(st["owner_pid"])
        stdin_fd = int(st["stdin_write_fd"])
        fd_path = f"/proc/{owner_pid}/fd/{stdin_fd}"
        keeper_fd = os.open(fd_path, os.O_WRONLY)
        log(
            f"hold opened {fd_path} -> keeper_fd={keeper_fd} "
            f"bridge_pid={bridge_pid} owner_pid={owner_pid}"
        )
        os.kill(owner_pid, signal.SIGKILL)
        try:
            owner.wait(timeout=3)
        except subprocess.TimeoutExpired:
            pass
        owner = None
        samples = _poll_bridge(bridge_pid, [0.5, 1.0, 2.0, 3.0])
        abandoned = all(s["alive"] and s["ppid"] == 1 for s in samples)
        log(f"hold_sigkill samples={samples} abandoned={abandoned}")

        os.close(keeper_fd)
        keeper_fd = None
        time.sleep(0.4)
        dead_after_release = not pid_alive(bridge_pid)
        log(f"after_keeper_close dead={dead_after_release}")

        detail: dict[str, Any] = {
            "control_no_hold_bridge_dead": ctrl_dead,
            "control_samples": ctrl_samples,
            "hold_samples": samples,
            "abandoned_ppid1": abandoned,
            "dead_after_keeper_close": dead_after_release,
            "bridge_pgid": st.get("bridge_pgid"),
            "owner_pgid": st.get("owner_pgid"),
            "same_pg_as_owner": st.get("bridge_pgid") == st.get("owner_pgid"),
        }
        # FIXED only when the hold-path no longer abandons; control must still EOF-die.
        fixed = (not abandoned) and ctrl_dead
        return _emit_result(name, fixed=fixed, detail=detail)
    except Exception as exc:  # noqa: BLE001
        log(f"ERROR {exc}")
        return _emit_result(name, False, {"error": str(exc)})
    finally:
        if keeper_fd is not None:
            try:
                os.close(keeper_fd)
            except OSError:
                pass
        if bridge_pid is not None and pid_alive(bridge_pid):
            try:
                os.kill(bridge_pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        if owner is not None and owner.poll() is None:
            try:
                os.kill(owner.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                owner.wait(timeout=3)
            except subprocess.TimeoutExpired:
                pass


SCENARIOS: dict[str, Callable[[], int]] = {
    "sigkill-stdin-hold-abandons-bridge": repro_sigkill_stdin_hold_abandons_bridge,
}

LIVE_SCENARIOS: frozenset[str] = frozenset()

# Local OS repros: need node + bridge.js, but no Cursor API.
LOCAL_SCENARIOS = frozenset(
    {
        "sigkill-stdin-hold-abandons-bridge",
    }
)


def list_scenarios() -> None:
    for i, name in enumerate(SCENARIOS, start=1):
        if name in LIVE_SCENARIOS:
            kind = "live"
        elif name in LOCAL_SCENARIOS:
            kind = "local"
        else:
            kind = "code"
        log(f"{i}. {name}  ({kind})")


def run_scenario(name: str) -> int:
    fn = SCENARIOS.get(name)
    if fn is None:
        log(f"unknown scenario: {name}")
        list_scenarios()
        return 2
    log(f"=== QA regression: {name} ===")
    return fn()


def run_all(include_live: bool = True) -> int:
    codes: list[int] = []
    for name, fn in SCENARIOS.items():
        if not include_live and name in LIVE_SCENARIOS:
            log(f"SKIP live scenario: {name}")
            continue
        log(f"=== QA regression: {name} ===")
        codes.append(fn())
    # 0 only if every run reports FIXED.
    return 0 if codes and all(c == 0 for c in codes) else 1


def run_self_tests() -> None:
    """Fast, offline checks (no Cursor API; no multi-second bridge spawn)."""
    assert list(SCENARIOS) == ["sigkill-stdin-hold-abandons-bridge"]
    assert "sigkill-stdin-hold-abandons-bridge" in LOCAL_SCENARIOS
    assert not LIVE_SCENARIOS
    assert pid_alive(os.getpid())
    assert repo_bridge_js().name == "bridge.js"
    with tempfile.TemporaryDirectory(prefix="malvin_qa_self_") as tmp:
        owner_py = Path(tmp) / "bridge_owner.py"
        _write_owner_script(owner_py)
        text = owner_py.read_text(encoding="utf-8")
        assert "subprocess.Popen" in text
        assert "stdin_write_fd" in text
    log("ALL qa self-tests OK")


def qa_cli_self_test() -> None:
    run_self_tests()


__all__ = [
    "LIVE_SCENARIOS",
    "LOCAL_SCENARIOS",
    "SCENARIOS",
    "list_scenarios",
    "qa_cli_self_test",
    "run_all",
    "run_scenario",
    "run_self_tests",
]
