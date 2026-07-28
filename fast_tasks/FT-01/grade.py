#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-01. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-01"


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


def evaluate(workspace: Path) -> int:
    hidden = Path(__file__).resolve().parent / "goldens" / "test_ringbuf_hidden.py"
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        shutil.copytree(workspace / "src", td_path / "src")
        (td_path / "tests").mkdir()
        shutil.copy2(hidden, td_path / "tests" / "test_ringbuf_hidden.py")
        (td_path / "tests" / "__init__.py").write_text("", encoding="utf-8")
        (td_path / "src" / "__init__.py").write_text("", encoding="utf-8")
        proc = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                "-q",
                "tests/test_ringbuf_hidden.py",
                "-p",
                "no:cacheprovider",
            ],
            cwd=td_path,
            capture_output=True,
            text=True,
            env={**os.environ, "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1", "PYTHONPATH": str(td_path)},
        )
        return 1 if proc.returncode == 0 else 0


def _oracle_fix(workspace: Path) -> None:
    text = (workspace / "src" / "ringbuf.py").read_text(encoding="utf-8")
    fixed = text.replace(
        "self._head = self._head + 1\n        if self._head > self.capacity:  # should be >= capacity / use %\n            self._head = 0",
        "self._head = (self._head + 1) % self.capacity",
    )
    if fixed == text:
        # fallback absolute correct impl
        fixed = '''class RingBuffer:
    def __init__(self, capacity: int) -> None:
        if capacity < 1:
            raise ValueError("capacity must be >= 1")
        self.capacity = capacity
        self._buf = [None] * capacity
        self._head = 0
        self._tail = 0
        self._size = 0

    def __len__(self) -> int:
        return self._size

    def push(self, item) -> None:
        if self._size >= self.capacity:
            raise IndexError("buffer full")
        self._buf[self._tail] = item
        self._tail = (self._tail + 1) % self.capacity
        self._size += 1

    def pop(self):
        if self._size == 0:
            raise IndexError("buffer empty")
        item = self._buf[self._head]
        self._buf[self._head] = None
        self._head = (self._head + 1) % self.capacity
        self._size -= 1
        return item
'''
    (workspace / "src" / "ringbuf.py").write_text(fixed, encoding="utf-8")


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0, "starter must fail"
        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"
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
