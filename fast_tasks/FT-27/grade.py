#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-27. No malvin/repo imports."""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib
import os
import shutil
import sqlite3
import statistics
import sys
import tempfile
import time
import unicodedata
from pathlib import Path


TASK_ID = "FT-27"
PROTECTED = {
    "plan.md": "608049d640b681b94d4e35cbd2f788386048e5f3bc8485af47ac4a7b62d1192f",
    "tests/test_r2_sqlite.py": "f45ebd6fc616fbe5a8007447efab365d448acaafa5553d55b35ea0bb087ade7a",
    "journal/__init__.py": "3d6767ae02447c70ffd26da1a9233b7e4a228c30fbecc63611ddf020f828de3e",
    "journal/r1_memory.py": "4317d39f53fe6f7528491599d1e6af89079596a2385288fcc08b898320c86a47",
    "journal/r1_sqlite.py": "f39a64a544ba6b922d82e008ce5742753d350bba46586358cd3672f5b9fe115a",
    "journal/r2_memory.py": "a1ee69c52e5610609404b7b29df0ad8ffb0dde9c485c1c376b14cbb74599d381",
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
    shutil.copytree(workspace / "journal", root / "journal")
    old_path = sys.path[:]
    try:
        sys.path.insert(0, str(root))
        for name in list(sys.modules):
            if name == "journal" or name.startswith("journal."):
                del sys.modules[name]
        module = importlib.import_module("journal.r2_sqlite")
    finally:
        sys.path[:] = old_path
        temp_dir.cleanup()
    return module


def _key(value: str) -> str:
    normalized = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not normalized:
        raise ValueError
    return normalized


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


def _reference(events):
    state: dict[str, tuple[int, bytes | None]] = {}
    for raw_key, sequence, value in events:
        key = _key(raw_key)
        old = state.get(key)
        if old is None or _wins(sequence, old[0]):
            state[key] = (sequence, value)
    rows = [(key, sequence, value) for key, (sequence, value) in sorted(state.items())]
    visible = [(key, value) for key, _, value in rows if value is not None]
    return rows, visible


def _rows(db: sqlite3.Connection):
    return [
        (key, sequence, None if value is None else bytes(value))
        for key, sequence, value in db.execute(
            "SELECT key, sequence, value FROM journal ORDER BY key"
        )
    ]


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    if not _protected_files_unchanged(workspace):
        return 0
    try:
        tree = ast.parse((workspace / "journal" / "r2_sqlite.py").read_text())
    except (OSError, SyntaxError):
        return 0
    for node in ast.walk(tree):
        if (
            isinstance(node, ast.ImportFrom)
            and node.module in {"r1_memory", "r2_memory"}
        ):
            return 0
    try:
        module = _load(workspace)
        open_db = module.open_db
        apply = module.apply
        snapshot = module.snapshot
    except Exception:
        return 0

    cases = [
        [("Beta", 1, b"old"), ("beta", 2, b"new"), ("alpha", 3, b"a")],
        [("clock", 65534, b"old"), ("clock", 1, b"wrapped")],
        [("clock", 1, b"new"), ("clock", 65534, b"stale")],
        [("k", 9, b"one"), ("k", 9, None), ("k", 9, b"last")],
        [("k", 20, b"keep"), ("k", 32788, b"ambiguous")],
        [("gone", 4, b"value"), ("gone", 5, None), ("gone", 3, b"stale")],
        [
            (" Straße ", 5, b"first"),
            ("STRASSE", 6, None),
            ("ＣＡＦＥ\u0301", 65535, b"wide"),
            ("café", 0, b"wrapped"),
        ],
        [],
    ]
    for events in cases:
        try:
            db = open_db()
            result = apply(db, iter(events))
            actual_rows = _rows(db)
            actual_visible = snapshot(db)
        except Exception:
            return 0
        expected_rows, expected_visible = _reference(events)
        if result is not None or actual_rows != expected_rows:
            return 0
        if type(actual_visible) is not list or actual_visible != expected_visible:
            return 0
        db.close()

    try:
        db = open_db()
        apply(db, [("base", 1, b"before")])
        before = _rows(db)

        def broken_events():
            yield ("base", 2, b"changed")
            yield ("other", 1, b"added")
            raise RuntimeError("source failed")

        try:
            apply(db, broken_events())
            return 0
        except RuntimeError:
            pass
        if _rows(db) != before:
            return 0

        transaction_observations = []

        def observing_events():
            transaction_observations.append(db.in_transaction)
            yield ("observed", 1, b"inside")

        apply(db, observing_events())
        if transaction_observations != [True]:
            return 0

        db.execute(
            "INSERT INTO journal(key, sequence, value) VALUES ('outer', 1, X'78')"
        )
        if not db.in_transaction:
            return 0
        apply(db, [("inside", 1, b"y")])
        if not db.in_transaction:
            return 0
        if snapshot(db) != [
            ("base", b"before"),
            ("inside", b"y"),
            ("observed", b"inside"),
            ("outer", b"x"),
        ]:
            return 0
        db.rollback()
        if snapshot(db) != [("base", b"before"), ("observed", b"inside")]:
            return 0

        db.execute(
            "INSERT INTO journal(key, sequence, value) VALUES ('outer', 2, X'7a')"
        )

        def invalid_inside():
            yield ("temp", 1, b"temp")
            yield ("bad", True, b"bad")

        try:
            apply(db, invalid_inside())
            return 0
        except (ValueError, TypeError):
            pass
        if not db.in_transaction or snapshot(db) != [
            ("base", b"before"),
            ("observed", b"inside"),
            ("outer", b"z"),
        ]:
            return 0
        db.rollback()
        db.close()
    except Exception:
        return 0

    bad_batches = [
        [("", 1, b"x")],
        [(" \t\r\n", 1, None)],
        [(3, 1, b"x")],
        [("x", -1, b"x")],
        [("x", 65536, b"x")],
        [("x", True, b"x")],
        [("x", 1, "not bytes")],
    ]
    for events in bad_batches:
        try:
            db = open_db()
            apply(db, events)
        except (ValueError, TypeError):
            db.close()
            continue
        except Exception:
            return 0
        return 0

    replay = [(" hot ", sequence & 0xFFFF, b"x") for sequence in range(30_000)]
    timings = []
    for _ in range(3):
        db = open_db()
        started = time.perf_counter()
        apply(db, iter(replay))
        timings.append(time.perf_counter() - started)
        if _rows(db) != [("hot", 29_999, b"x")]:
            return 0
        db.close()
    if statistics.median(timings) > 0.120:
        return 0
    return 1


ORACLE = '''\
"""Revision 2 journal backed by SQLite."""
from __future__ import annotations

import sqlite3
import unicodedata
from collections.abc import Iterable

Event = tuple[str, int, bytes | None]


def _sequence(value: int) -> int:
    if type(value) is not int or not 0 <= value <= 0xFFFF:
        raise ValueError("sequence out of range")
    return value


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    normalized = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not normalized:
        raise ValueError("empty key")
    return normalized


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


def open_db(path: str = ":memory:") -> sqlite3.Connection:
    db = sqlite3.connect(path)
    db.execute(
        "CREATE TABLE IF NOT EXISTS journal "
        "(key TEXT PRIMARY KEY, sequence INTEGER NOT NULL, value BLOB)"
    )
    return db


def apply(db: sqlite3.Connection, events: Iterable[Event]) -> None:
    db.execute("SAVEPOINT journal_apply")
    try:
        normalized = []
        keys = set()
        for raw_key, raw_sequence, value in events:
            key = _key(raw_key)
            sequence = _sequence(raw_sequence)
            if value is not None and type(value) is not bytes:
                raise TypeError("value must be bytes or None")
            normalized.append((key, sequence, value))
            keys.add(key)
        staged = {}
        key_list = list(keys)
        for start in range(0, len(key_list), 800):
            chunk = key_list[start:start + 800]
            marks = ",".join("?" for _ in chunk)
            staged.update(
                (key, (sequence, None if value is None else bytes(value)))
                for key, sequence, value in db.execute(
                    f"SELECT key, sequence, value FROM journal WHERE key IN ({marks})",
                    chunk,
                )
            )
        changed = set()
        for key, sequence, value in normalized:
            old = staged.get(key)
            if old is None or _wins(sequence, old[0]):
                staged[key] = (sequence, value)
                changed.add(key)
        db.executemany(
            "INSERT INTO journal(key, sequence, value) VALUES (?, ?, ?) "
            "ON CONFLICT(key) DO UPDATE SET "
            "sequence = excluded.sequence, value = excluded.value",
            ((key, staged[key][0], staged[key][1]) for key in changed),
        )
    except BaseException:
        db.execute("ROLLBACK TO journal_apply")
        db.execute("RELEASE journal_apply")
        raise
    db.execute("RELEASE journal_apply")


def snapshot(db: sqlite3.Connection) -> list[tuple[str, bytes]]:
    return [
        (key, bytes(value))
        for key, value in db.execute(
            "SELECT key, value FROM journal "
            "WHERE value IS NOT NULL ORDER BY key"
        )
    ]
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "journal" / "r2_sqlite.py").write_text(ORACLE, encoding="utf-8")


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
