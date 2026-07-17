#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-28. No malvin/repo imports."""
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
import zlib
from pathlib import Path


TASK_ID = "FT-28"
PROTECTED = {
    "plan.md": "c36f403f6f1c4d2aa396b0c98f367efe0776cd58cc8d0e6b8608e1fbe3614ed0",
    "tests/test_r2_binary.py": "06561dd5a4871c632366e99a2279844bf47e80ba1e0b781cec9e16923560ec1d",
    "timeline/__init__.py": "1938a17e177002100a3118a1acb3f7d20a4fb9b8cdce7370d8d93f977d61a58c",
    "timeline/r1_binary.py": "3406cdd0c2c3cccc5839bf5d35033ed4be999b4a6c6f7b5c64c7982fb9a784c5",
    "timeline/r1_text.py": "d08a86c49feff045bf0893e5c4f36868e3cb051c3b4acaaa2165aa9e311cabd9",
    "timeline/r2_text.py": "0b9dca6133739a3b5c6c03c76c683a63b8f260d850984205b0a3e9505d2c76da",
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
    shutil.copytree(workspace / "timeline", root / "timeline")
    old_path = sys.path[:]
    try:
        sys.path.insert(0, str(root))
        for name in list(sys.modules):
            if name == "timeline" or name.startswith("timeline."):
                del sys.modules[name]
        module = importlib.import_module("timeline.r2_binary")
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


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


def _varint(value: int) -> bytes:
    result = bytearray()
    while value >= 0x80:
        result.append((value & 0x7F) | 0x80)
        value >>= 7
    result.append(value)
    return bytes(result)


def _canonical(entries):
    latest = {}
    for raw_key, version, value in entries:
        key = _key(raw_key)
        if type(version) is not int or not 0 <= version <= 0xFFFF:
            raise ValueError
        if value is not None and type(value) is not bytes:
            raise TypeError
        old = latest.get(key)
        if old is None or _wins(version, old[0]):
            latest[key] = (version, value)
    return [(key, *latest[key]) for key in sorted(latest)]


def _frame(records, magic=b"B2", count=None, checksum=True) -> bytes:
    chunks = [magic, _varint(len(records) if count is None else count)]
    for key, version, value in records:
        encoded = key.encode("utf-8")
        chunks.extend((_varint(len(encoded)), encoded, version.to_bytes(2, "little")))
        if value is None:
            chunks.append(b"\x00")
        else:
            compressed = zlib.compress(value, level=9)
            chunks.extend((b"\x01", _varint(len(compressed)), compressed))
    body = b"".join(chunks)
    trailer = zlib.adler32(body).to_bytes(4, "big") if checksum else b"\0\0\0\0"
    return body + trailer


def _reference(entries):
    records = _canonical(entries)
    return _frame(records), records


def _with_check(body: bytes) -> bytes:
    return body + zlib.adler32(body).to_bytes(4, "big")


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    if not _protected_files_unchanged(workspace):
        return 0
    try:
        source = (workspace / "timeline" / "r2_binary.py").read_text()
        tree = ast.parse(source)
    except (OSError, SyntaxError):
        return 0
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module in {
            "r1_text",
            "r2_text",
            "timeline.r1_text",
            "timeline.r2_text",
        }:
            return 0
    try:
        module = _load(workspace)
        pack = module.pack
        unpack = module.unpack
    except Exception:
        return 0

    cases = [
        [],
        [("alpha", 1, b"a"), ("beta", 2, None), ("gamma", 3, b"")],
        [(" z ", 8, b"old"), ("A", 4, b"a"), ("Z", 9, b"new")],
        [("Straße", 10, b"old"), (" STRASSE ", 10, None)],
        [("ＣＡＦＥ\u0301", 65535, b"wide"), ("café", 0, b"wrapped")],
        [("clock", 1, b"new"), ("clock", 65534, b"stale")],
        [("half", 20, b"keep"), ("half", 32788, b"lose")],
        [("x", 5, None), ("x", 5, b""), ("x", 5, b"last")],
        [("large", 128, bytes(range(256)) * 2)],
    ]
    for entries in cases:
        expected_frame, expected_records = _reference(entries)
        try:
            actual_frame = pack(list(entries))
            decoded_expected = unpack(expected_frame)
            decoded_actual = unpack(actual_frame)
        except Exception:
            return 0
        if type(actual_frame) is not bytes or actual_frame != expected_frame:
            return 0
        if decoded_expected != expected_records or decoded_actual != expected_records:
            return 0

    bad_entries = [
        [("", 1, b"x")],
        [(3, 1, b"x")],
        [("x", True, b"x")],
        [("x", -1, b"x")],
        [("x", 65536, b"x")],
        [("x", 1, bytearray(b"x"))],
    ]
    for entries in bad_entries:
        try:
            pack(entries)
        except (TypeError, ValueError):
            continue
        except Exception:
            return 0
        return 0

    valid = _reference([("alpha", 1, b"x")])[0]
    duplicate = _frame([("a", 1, b"x"), ("a", 2, b"y")])
    descending = _frame([("b", 1, b"x"), ("a", 2, b"y")])
    noncanonical = _frame([(" A ", 1, b"x")])
    bad_utf8_body = b"B2\x01\x01\xff\x00\x01\x00"
    overlong_count_body = b"B2\x81\x00" + valid[3:-4]
    malformed = [
        b"",
        b"B1" + valid[2:],
        valid[:-1],
        valid[:-4] + b"\0\0\0\0",
        valid + b"x",
        duplicate,
        descending,
        noncanonical,
        _with_check(bad_utf8_body),
        _with_check(overlong_count_body),
        _with_check(b"B2\x01"),
        _with_check(b"B2\x01\x01a\x01\x00\x02"),
        _with_check(b"B2\x01\x01a\x01\x00\x01\x80"),
        _with_check(b"B2\x01\x01a\x01\x00\x02"),
        _with_check(b"B2\x00x"),
    ]
    for frame in malformed:
        try:
            unpack(frame)
        except (TypeError, ValueError, UnicodeDecodeError):
            continue
        except Exception:
            return 0
        return 0
    try:
        unpack(bytearray(valid))
    except TypeError:
        pass
    except Exception:
        return 0
    else:
        return 0
    return 1


ORACLE = '''\
"""Revision 2 timeline codec over the binary transport."""
from __future__ import annotations

import zlib
import unicodedata

from .r1_binary import Entry, _take_varint, _varint, _version


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    result = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not result:
        raise ValueError("empty key")
    return result


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


def _canonical(entries: list[Entry]) -> list[Entry]:
    latest = {}
    for raw_key, raw_version, value in entries:
        key = _key(raw_key)
        version = _version(raw_version)
        if value is not None and type(value) is not bytes:
            raise TypeError("value must be bytes or None")
        old = latest.get(key)
        if old is None or _wins(version, old[0]):
            latest[key] = (version, value)
    return [(key, *latest[key]) for key in sorted(latest)]


def pack(entries: list[Entry]) -> bytes:
    records = _canonical(entries)
    chunks = [b"B2", _varint(len(records))]
    for key, version, value in records:
        encoded = key.encode("utf-8")
        chunks.extend((_varint(len(encoded)), encoded, version.to_bytes(2, "little")))
        if value is None:
            chunks.append(b"\\x00")
        else:
            compressed = zlib.compress(value, level=9)
            chunks.extend((b"\\x01", _varint(len(compressed)), compressed))
    body = b"".join(chunks)
    return body + zlib.adler32(body).to_bytes(4, "big")


def unpack(frame: bytes) -> list[Entry]:
    if type(frame) is not bytes:
        raise TypeError("frame must be bytes")
    if len(frame) < 7 or frame[:2] != b"B2":
        raise ValueError("wrong revision")
    body, checksum = frame[:-4], frame[-4:]
    if zlib.adler32(body).to_bytes(4, "big") != checksum:
        raise ValueError("bad checksum")
    count, offset = _take_varint(body, 2)
    result = []
    previous = None
    for _ in range(count):
        size, offset = _take_varint(body, offset)
        end = offset + size
        if end + 3 > len(body):
            raise ValueError("truncated record")
        try:
            key = body[offset:end].decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError("bad key encoding") from error
        if key != _key(key) or previous is not None and key <= previous:
            raise ValueError("non-canonical key order")
        version = int.from_bytes(body[end:end + 2], "little")
        marker = body[end + 2]
        offset = end + 3
        if marker == 0:
            value = None
        elif marker == 1:
            size, offset = _take_varint(body, offset)
            end = offset + size
            if end > len(body):
                raise ValueError("truncated payload")
            inflater = zlib.decompressobj()
            try:
                value = inflater.decompress(body[offset:end])
                value += inflater.flush()
            except zlib.error as error:
                raise ValueError("bad payload") from error
            if not inflater.eof or inflater.unused_data or inflater.unconsumed_tail:
                raise ValueError("bad payload")
            offset = end
        else:
            raise ValueError("bad payload marker")
        result.append((key, version, value))
        previous = key
    if offset != len(body):
        raise ValueError("trailing data")
    return result
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "timeline" / "r2_binary.py").write_text(ORACLE, encoding="utf-8")


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
