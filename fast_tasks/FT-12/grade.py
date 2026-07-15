#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-12. No malvin/repo imports."""
from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


TASK_ID = "FT-12"


def write_reward(path: Path, value: int) -> None:
    assert value in (0, 1)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{value}\n", encoding="utf-8")


def default_workspace() -> Path:
    return Path(__file__).resolve().parent / "workspace"


def default_reward_out() -> Path:
    env = os.environ.get("MALVIN_REWARD_PATH") or os.environ.get("HARBOR_REWARD_PATH")
    if env:
        return Path(env)
    return Path(__file__).resolve().parent / "reward.txt"


EXPECTED_START_SHA = "1aaaaaf0e5078b0455c4c9521f180f86d7b96f50ba1931388481b681651871e0"


def _evaluate_mutable(workspace: Path) -> int:
    start = workspace / "svc" / "start.sh"
    if not start.is_file():
        return 0
    sha = hashlib.sha256(start.read_bytes()).hexdigest()
    if sha != EXPECTED_START_SHA:
        return 0
    port_file = workspace / "var" / "port"
    if port_file.exists():
        port_file.unlink()
    proc = subprocess.run(["bash", str(start)], cwd=workspace, capture_output=True, text=True)
    if proc.returncode != 0:
        return 0
    if not port_file.is_file():
        return 0
    if port_file.read_text(encoding="utf-8").strip() != "8080":
        return 0
    if port_file.exists():
        port_file.unlink()
    proc2 = subprocess.run(
        ["env", "-i", "PATH=/usr/bin:/bin", "bash", str(start)],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    if proc2.returncode != 0:
        return 0
    if port_file.read_text(encoding="utf-8").strip() != "8080":
        return 0
    return 1


def evaluate(workspace: Path) -> int:
    # Copy so start.sh side effects (var/port) never mutate the caller's tree.
    with tempfile.TemporaryDirectory() as td:
        ws = Path(td) / "ws"
        shutil.copytree(workspace, ws)
        return _evaluate_mutable(ws)


def _oracle_fix(workspace: Path) -> None:
    bad = workspace / "svc" / "env.d" / "99-bad-port.sh"
    if bad.exists():
        bad.unlink()


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0
        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    p.add_argument("--workspace", type=Path, default=None)
    p.add_argument("--reward-out", type=Path, default=None)
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    ws = args.workspace or default_workspace()
    out = args.reward_out or default_reward_out()
    reward = evaluate(ws)
    write_reward(out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
