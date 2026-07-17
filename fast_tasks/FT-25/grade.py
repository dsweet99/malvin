#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-25. No malvin/repo imports."""
from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import os
import shutil
import sys
import tempfile
import unicodedata
from pathlib import Path


TASK_ID = "FT-25"
PROTECTED = {
    "plan.md": "2e3ca8982bc01e0612a22a546add04ceeaa40c990f0d85cc5c5f9f3f3f2ea228",
    "tests/test_r2_binary.py": "fd73717b3fb84a5b45bb8e815d0659ebb3c017da4f1a9d9c8929d576ec78390d",
    "wire/__init__.py": "5522c073f89d78188cdca513cd8b82edac945450b377288a158e65985ad1650c",
    "wire/r1_binary.py": "8db78103369c9bc1e1195a3f70bf8e1b7432f9cf7f436370dfeb72ca97611d8e",
    "wire/r1_text.py": "1d996216f733ada1555b4007736d6d83bcb2daa84272cddcf5b6a9eac586206e",
    "wire/r2_text.py": "0dba6170f1a3f4aca2c33e2e7e99c56246eed84b8306c679a8f97714515d1c73",
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


def _name(value: bytes) -> bytes:
    text = value.decode("utf-8")
    normalized = unicodedata.normalize("NFKC", " ".join(text.split()).casefold()).encode()
    if not normalized:
        raise ValueError
    return normalized


def _reference(fields: list[tuple[bytes, bytes | None]]) -> tuple[bytes, list[tuple[bytes, bytes | None]]]:
    latest: dict[bytes, bytes | None] = {}
    for name, value in fields:
        latest[_name(name)] = value
    records = sorted(latest.items())
    chunks = [b"B2", _varint(len(records))]
    for name, value in records:
        chunks.extend((_varint(len(name)), name))
        if value is None:
            chunks.append(b"\x00")
        else:
            chunks.extend((_varint(len(value) + 1), value))
    return b"".join(chunks), records


def _golden_cases() -> list[tuple[list[tuple[bytes, bytes | None]], bytes]]:
    path = Path(__file__).resolve().parent / "goldens" / "vectors.json"
    raw = json.loads(path.read_text(encoding="utf-8"))
    cases = []
    for vector in raw:
        fields = [
            (bytes.fromhex(name), None if value is None else bytes.fromhex(value))
            for name, value in vector["fields"]
        ]
        cases.append((fields, bytes.fromhex(vector["packed_hex"])))
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
        [(b" Z ", b"\x00:\xff"), (b"a", b""), (b"z", None), (b"z", b"2")],
        [(b"a" * 130, b"v" * 127)],
        [(b"\r\nMiXeD\t Name\v", b"\x00" * 300), (b"mixed name", None)],
        [(b"k", None), (b"k", b""), (b"k", None), (b"k", b"\x80")],
        [
            (" Straße ".encode(), b"first"),
            (b"STRASSE", None),
            ("ＣＡＦＥ\u0301".encode(), b"wide"),
            ("café".encode(), b"composed"),
        ],
    ]
    cases.extend((fields, _reference(fields)[0]) for fields in generated)
    for fields, expected_frame in cases:
        expected_records = _reference(fields)[1]
        try:
            actual_frame = pack(list(fields))
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
        b"B2\x00\x00",
        b"B2\x01\x00\x00",
        b"B2\x01\x01A\x00",
        b"B2\x01\x03a  \x00",
        b"B2\x01\x01\xff\x00",
        b"B2\x01\x07Stra\xc3\x9fe\x00",
        b"B2\x02\x01b\x00\x01a\x00",
        b"B2\x02\x01a\x00\x01a\x01",
        b"B2\x01\x81\x00a\x00",
        b"B2\x01\x01a\x80\x00",
        b"B2\x01\x01a\x02",
        b"B2\x00junk",
    ]
    for frame in invalid:
        try:
            unpack(frame)
        except (ValueError, TypeError):
            continue
        except Exception:
            return 0
        return 0

    bad_fields = [
        [(b"", b"x")],
        [(b" \t\r\n", None)],
        [(b"\xff", b"x")],
    ]
    for fields in bad_fields:
        try:
            pack(fields)
        except ValueError:
            continue
        except Exception:
            return 0
        return 0
    return 1


ORACLE = '''\
"""Revision 2 over the binary transport."""
from __future__ import annotations

import unicodedata

from .r1_binary import Field, _take_varint, _varint


def _name(value: bytes) -> bytes:
    text = value.decode("utf-8")
    normalized = unicodedata.normalize("NFKC", " ".join(text.split()).casefold()).encode()
    if not normalized:
        raise ValueError("empty field name")
    return normalized


def _nullable_size(value: bytes | None) -> bytes:
    """Zero is null; a present value is encoded as its byte length plus one."""
    return _varint(0 if value is None else len(value) + 1)


def pack(fields: list[Field]) -> bytes:
    latest: dict[bytes, bytes | None] = {}
    for name, value in fields:
        latest[_name(name)] = value
    records = sorted(latest.items())
    chunks = [b"B2", _varint(len(records))]
    for name, value in records:
        chunks.extend((_varint(len(name)), name, _nullable_size(value)))
        if value is not None:
            chunks.append(value)
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Field]:
    if not frame.startswith(b"B2"):
        raise ValueError("wrong revision")
    count, offset = _take_varint(frame, 2)
    result: list[Field] = []
    for _ in range(count):
        name_size, offset = _take_varint(frame, offset)
        end = offset + name_size
        if end > len(frame):
            raise ValueError("truncated name")
        name = frame[offset:end]
        offset = end
        nullable_size, offset = _take_varint(frame, offset)
        if nullable_size == 0:
            value = None
        else:
            end = offset + nullable_size - 1
            if end > len(frame):
                raise ValueError("truncated value")
            value = frame[offset:end]
            offset = end
        if _name(name) != name:
            raise ValueError("non-canonical name")
        result.append((name, value))
    if offset != len(frame):
        raise ValueError("trailing data")
    names = [name for name, _ in result]
    if names != sorted(set(names)):
        raise ValueError("non-canonical order")
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
