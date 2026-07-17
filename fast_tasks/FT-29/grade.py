#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-29. No malvin/repo imports."""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib
import os
import shutil
import sys
import tempfile
import unicodedata
from pathlib import Path


TASK_ID = "FT-29"
PROTECTED = {
    "plan.md": "0ea174c4a199dca527d3c84b939d41e9c5ac1554819df5f22f96de35224ba748",
    "tests/test_r2_heap.py": "c1f07bf9de3cac5efadc501f610413741fe47bc5b785dce163792ce665550e59",
    "dispatch/__init__.py": "1706c18f55f31c8edb2c031d93928b62acb4f8ab9c17e1e462fbec5c281333ce",
    "dispatch/r1_linear.py": "a1d4c90d808199f387c2c28f9c23eb1a1f4efff46dd8a2215e429d4958e6ac8f",
    "dispatch/r1_heap.py": "133be93491f4f47d8096c40b80d32037661336b906e6f5dae30c00a1c38da3b2",
    "dispatch/r2_linear.py": "f633f3ccb39dd241679c918a82ac414117c912b82726837ee7a93606c6ee8bf3",
}


def write_reward(path: Path, value: int) -> None:
    assert value in (0, 1)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{value}\n", encoding="utf-8")


def default_workspace() -> Path:
    return Path(__file__).resolve().parent / "workspace"


def default_reward_out() -> Path:
    env = os.environ.get("MALVIN_REWARD_PATH") or os.environ.get("HARBOR_REWARD_PATH")
    return Path(env) if env else Path(__file__).resolve().parent / "reward.txt"


def _protected_files_unchanged(workspace: Path) -> bool:
    for relative, expected in PROTECTED.items():
        path = workspace / relative
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            return False
    return True


def _load(workspace: Path):
    temp_dir = tempfile.TemporaryDirectory()
    root = Path(temp_dir.name)
    shutil.copytree(workspace / "dispatch", root / "dispatch")
    old_path = sys.path[:]
    try:
        sys.path.insert(0, str(root))
        for name in list(sys.modules):
            if name == "dispatch" or name.startswith("dispatch."):
                del sys.modules[name]
        module = importlib.import_module("dispatch.r2_heap")
    finally:
        sys.path[:] = old_path
        temp_dir.cleanup()
    return module


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError
    result = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not result:
        raise ValueError
    return result


def _generation(value: int) -> int:
    if type(value) is not int or not 0 <= value <= 0xFFFF:
        raise ValueError
    return value


def _priority(value: int) -> int:
    if type(value) is not int or not -(1 << 31) <= value < (1 << 31):
        raise ValueError
    return value


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


class Model:
    def __init__(self) -> None:
        self.state: dict[str, tuple[int, int, bytes | None]] = {}

    def put(self, key, generation, priority, payload) -> None:
        key = _key(key)
        generation = _generation(generation)
        priority = _priority(priority)
        if payload is not None and type(payload) is not bytes:
            raise TypeError
        old = self.state.get(key)
        if old is None or _wins(generation, old[0]):
            self.state[key] = (generation, priority, payload)

    def take(self):
        ready = [
            (key, generation, priority, payload)
            for key, (generation, priority, payload) in self.state.items()
            if payload is not None
        ]
        if not ready:
            return None
        result = min(ready, key=lambda item: (-item[2], item[0]))
        key, generation, priority, payload = result
        self.state[key] = (generation, priority, None)
        return key, generation, priority, payload


def _exercise(dispatcher, operations):
    results = []
    for operation in operations:
        if operation[0] == "put":
            result = dispatcher.put(*operation[1:])
            if result is not None:
                raise AssertionError("put must return None")
        else:
            results.append(dispatcher.take())
    return results


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    if not _protected_files_unchanged(workspace):
        return 0
    try:
        source = (workspace / "dispatch" / "r2_heap.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
    except (OSError, SyntaxError):
        return 0
    imports_heapq = False
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports_heapq |= any(alias.name == "heapq" for alias in node.names)
        if isinstance(node, ast.ImportFrom):
            if node.module in {"r2_linear", "dispatch.r2_linear"}:
                return 0
            imports_heapq |= node.module == "heapq"
    if not imports_heapq:
        return 0
    try:
        module = _load(workspace)
        dispatcher_type = module.Dispatcher
    except Exception:
        return 0

    cases = [
        [
            ("put", "alpha", 1, 3, b"a"),
            ("put", "beta", 1, 9, b"b"),
            ("take",),
            ("take",),
            ("take",),
        ],
        [
            ("put", "job", 7, 2, b"old"),
            ("put", "job", 7, 20, b"new"),
            ("take",),
            ("take",),
        ],
        [
            ("put", "job", 9, 100, b"value"),
            ("put", "job", 9, -3, None),
            ("take",),
            ("put", "job", 8, 50, b"stale"),
            ("take",),
            ("put", "job", 9, 4, b"equal"),
            ("take",),
        ],
        [
            ("put", "clock", 65534, 1, b"old"),
            ("put", "clock", 1, 4, b"wrapped"),
            ("put", "clock", 65535, 99, b"late-old"),
            ("take",),
        ],
        [
            ("put", "half", 20, 5, b"keep"),
            ("put", "half", 32788, 90, b"opposite"),
            ("take",),
        ],
        [
            ("put", " Straße ", 2, 8, b"first"),
            ("put", "STRASSE", 2, 8, b"second"),
            ("put", "ＣＡＦＥ\u0301", 4, 8, b"wide"),
            ("put", "café", 5, 8, b"composed"),
            ("put", "alpha", 1, 8, b"a"),
            ("take",),
            ("take",),
            ("take",),
        ],
        [
            ("put", "used", 100, 1, b"once"),
            ("take",),
            ("put", "used", 99, 50, b"delayed"),
            ("take",),
            ("put", "used", 100, 2, b"correction"),
            ("take",),
            ("put", "used", 101, 3, None),
            ("put", "used", 100, 100, b"older"),
            ("take",),
        ],
        [
            ("put", "low", 1, -(1 << 31), b"low"),
            ("put", "high", 1, (1 << 31) - 1, b"high"),
            ("take",),
            ("take",),
        ],
    ]
    for operations in cases:
        try:
            actual = dispatcher_type()
            expected = Model()
            actual_results = _exercise(actual, operations)
            expected_results = _exercise(expected, operations)
        except Exception:
            return 0
        if actual_results != expected_results:
            return 0
        if not isinstance(getattr(actual, "_heap", None), list):
            return 0

    bad = [
        ("", 1, 2, b"x"),
        (" \t\n", 1, 2, None),
        (3, 1, 2, b"x"),
        ("x", True, 2, b"x"),
        ("x", -1, 2, b"x"),
        ("x", 65536, 2, b"x"),
        ("x", 1, True, b"x"),
        ("x", 1, -(1 << 31) - 1, b"x"),
        ("x", 1, 1 << 31, b"x"),
        ("x", 1, 2, bytearray(b"x")),
    ]
    for values in bad:
        try:
            actual = dispatcher_type()
            actual.put("safe", 1, 1, b"safe")
            actual.put(*values)
        except (TypeError, ValueError):
            if actual.take() != ("safe", 1, 1, b"safe") or actual.take() is not None:
                return 0
            continue
        except Exception:
            return 0
        return 0
    return 1


ORACLE = '''\
"""Revision 2 dispatcher using a priority heap."""
from __future__ import annotations

import heapq
import itertools
import unicodedata

from .r1_linear import _priority

Job = tuple[str, int, int, bytes]


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    result = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not result:
        raise ValueError("empty key")
    return result


def _generation(value: int) -> int:
    if type(value) is not int or not 0 <= value <= 0xFFFF:
        raise ValueError("generation out of range")
    return value


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


class Dispatcher:
    def __init__(self) -> None:
        self._heap: list[tuple[int, str, int, int, bytes]] = []
        self._state: dict[str, tuple[int, int, bytes | None, int]] = {}
        self._order = itertools.count()

    def put(
        self,
        key: str,
        generation: int,
        priority: int,
        payload: bytes | None,
    ) -> None:
        key = _key(key)
        generation = _generation(generation)
        priority = _priority(priority)
        if payload is not None and type(payload) is not bytes:
            raise TypeError("payload must be bytes or None")
        old = self._state.get(key)
        if old is None or _wins(generation, old[0]):
            token = next(self._order)
            self._state[key] = (generation, priority, payload, token)
            if payload is not None:
                heapq.heappush(
                    self._heap,
                    (-priority, key, token, generation, payload),
                )

    def take(self) -> Job | None:
        while self._heap:
            negative, key, token, generation, payload = heapq.heappop(self._heap)
            current = self._state.get(key)
            if current is None or current[3] != token or current[2] is None:
                continue
            priority = -negative
            self._state[key] = (generation, priority, None, token)
            return key, generation, priority, payload
        return None
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "dispatch" / "r2_heap.py").write_text(ORACLE, encoding="utf-8")


def self_test() -> None:
    source = default_workspace()
    with tempfile.TemporaryDirectory() as temp_dir:
        fail_workspace = Path(temp_dir) / "fail"
        shutil.copytree(source, fail_workspace)
        assert evaluate(fail_workspace) == 0
        pass_workspace = Path(temp_dir) / "pass"
        shutil.copytree(source, pass_workspace)
        _oracle_fix(pass_workspace)
        assert evaluate(pass_workspace) == 1
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    parser.add_argument("--workspace", type=Path, default=None)
    parser.add_argument("--reward-out", type=Path, default=None)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    workspace = args.workspace or default_workspace()
    reward_out = args.reward_out or default_reward_out()
    reward = evaluate(workspace)
    write_reward(reward_out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
