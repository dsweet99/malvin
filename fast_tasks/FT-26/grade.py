#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-26. No malvin/repo imports."""
from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-26"
PROTECTED = {
    "plan.md": "4a405904dfd18b0e748ebaf8bb898e89d2bfcba73c765e92d07598d60b8eaba0",
    "tests/test_r2_binary.py": "fd65437d6f0417e5c7db9b9904e542a4daa6edcf87e611b16d48519de992f783",
    "wire/__init__.py": "8b2d25ff00018823fdaabd260d702711766b366344145a7969c14e43a8780758",
    "wire/r1_binary.py": "4935e2723e92166df722f71a889c590ac63bc807ed0f4cbf705e1e551896a6e5",
    "wire/r1_text.py": "1bb58ae8943842996f94329859775c090fec06acd2b07fcd5e9c29cc0c2d15b3",
    "wire/r2_text.py": "b310860805b5a8adca8812663150242ad20ef0a6dc49e0257db28a9da1285fda",
}


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


def _protected_files_unchanged(workspace: Path) -> bool:
    for relative, expected in PROTECTED.items():
        path = workspace / relative
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            return False
    return True


def _load(workspace: Path):
    temp_dir = tempfile.TemporaryDirectory()
    root = Path(temp_dir.name)
    shutil.copytree(workspace / "wire", root / "wire")
    old_path = sys.path[:]
    try:
        sys.path.insert(0, str(root))
        for name in list(sys.modules):
            if name == "wire" or name.startswith("wire."):
                del sys.modules[name]
        module = importlib.import_module("wire.r2_binary")
    finally:
        sys.path[:] = old_path
        temp_dir.cleanup()
    return module


def _varint(value: int) -> bytes:
    result = bytearray()
    while value >= 0x80:
        result.append((value & 0x7F) | 0x80)
        value >>= 7
    result.append(value)
    return bytes(result)


def _reference(entries: list[tuple[int, bool]]) -> tuple[bytes, list[tuple[int, bool]]]:
    latest: dict[int, bool] = {}
    for identifier, live in entries:
        latest[identifier] = bool(live)
    records = sorted(latest.items())
    chunks = [b"B2", _varint(len(records))]
    previous = None
    for identifier, live in records:
        gap = identifier if previous is None else identifier - previous
        chunks.extend((_varint(gap), b"\x01" if live else b"\x00"))
        previous = identifier
    return b"".join(chunks), records


def _golden_cases() -> list[tuple[list[tuple[int, bool]], bytes]]:
    path = Path(__file__).resolve().parent / "goldens" / "vectors.json"
    raw = json.loads(path.read_text(encoding="utf-8"))
    cases = []
    for vector in raw:
        entries = [(identifier, live) for identifier, live in vector["entries"]]
        cases.append((entries, bytes.fromhex(vector["packed_hex"])))
    return cases


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    if not _protected_files_unchanged(workspace):
        return 0
    try:
        module = _load(workspace)
        pack = module.pack
        unpack = module.unpack
    except Exception:
        return 0

    cases = _golden_cases()
    generated = [
        [(0, True), (5, False), (5, True), (2, False)],
        [(10, False)],
        [(1, True), (1, False), (1, True)],
        [(9, True), (9, False)],
        [(300, True), (0, False), (128, True), (300, False)],
        [],
    ]
    cases.extend((entries, _reference(entries)[0]) for entries in generated)
    for entries, expected_frame in cases:
        expected_records = _reference(entries)[1]
        try:
            actual_frame = pack(list(entries))
            actual_records = unpack(expected_frame)
        except Exception:
            return 0
        if type(actual_frame) is not bytes or actual_frame != expected_frame:
            return 0
        if type(actual_records) is not list or actual_records != expected_records:
            return 0
        try:
            if unpack(actual_frame) != expected_records:
                return 0
        except Exception:
            return 0

    invalid = [
        b"",
        b"B1\x00",
        b"B2",
        b"B2\x80\x00",
        b"B2\x80\x80\x80\x80\x10",
        b"B2\x01",
        b"B2\x01\x00",
        b"B2\x01\x00\x02",
        b"B2\x02\x00\x01\x00\x01",
        b"B2\x01\x00\x01\xff",
        b"B2\x02\x00\x01",
        b"B2\x02\x05\x01\x00\x01",
    ]
    for frame in invalid:
        try:
            unpack(frame)
        except (ValueError, TypeError):
            continue
        except Exception:
            return 0
        return 0

    bad_entries = [
        [(-1, True)],
        [(1, True), (-3, False)],
        [("x", True)],
    ]
    for entries in bad_entries:
        try:
            pack(entries)
        except (ValueError, TypeError):
            continue
        except Exception:
            return 0
        return 0
    return 1


ORACLE = '''\
"""Revision 2 over the binary transport."""
from __future__ import annotations

from .r1_binary import Posting, _check, _take_varint, _varint

Entry = tuple[Posting, bool]

_LIVE = b"\\x01"
_TOMB = b"\\x00"


def _flag(live: bool) -> bytes:
    """Encode the liveness flag as a single trailing byte."""
    return _LIVE if live else _TOMB


def pack(entries: list[Entry]) -> bytes:
    latest: dict[Posting, bool] = {}
    for identifier, live in entries:
        latest[_check(identifier)] = bool(live)
    ordered = sorted(latest.items())
    chunks = [b"B2", _varint(len(ordered))]
    previous = None
    for identifier, live in ordered:
        gap = identifier if previous is None else identifier - previous
        chunks.extend((_varint(gap), _flag(live)))
        previous = identifier
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Entry]:
    if not frame.startswith(b"B2"):
        raise ValueError("wrong revision")
    count, offset = _take_varint(frame, 2)
    result: list[Entry] = []
    previous = None
    for _ in range(count):
        gap, offset = _take_varint(frame, offset)
        if offset >= len(frame):
            raise ValueError("truncated liveness flag")
        flag = frame[offset:offset + 1]
        offset += 1
        if flag not in (_LIVE, _TOMB):
            raise ValueError("bad liveness flag")
        if previous is None:
            value = gap
        else:
            if gap < 1:
                raise ValueError("non-increasing posting")
            value = previous + gap
        result.append((value, flag == _LIVE))
        previous = value
    if offset != len(frame):
        raise ValueError("trailing data")
    return result
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "wire" / "r2_binary.py").write_text(ORACLE, encoding="utf-8")


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
