"""Revision 2 journal backed by SQLite."""
from __future__ import annotations

import sqlite3
from collections.abc import Iterable

from .r1_memory import Event, _key, _sequence


def open_db(path: str = ":memory:") -> sqlite3.Connection:
    db = sqlite3.connect(path)
    db.execute(
        "CREATE TABLE IF NOT EXISTS journal "
        "(key TEXT PRIMARY KEY, sequence INTEGER NOT NULL, value BLOB)"
    )
    return db


def apply(db: sqlite3.Connection, events: Iterable[Event]) -> None:
    with db:
        for raw_key, raw_sequence, value in events:
            key = _key(raw_key)
            sequence = _sequence(raw_sequence)
            row = db.execute(
                "SELECT sequence FROM journal WHERE key = ?", (key,)
            ).fetchone()
            if row is not None and sequence <= row[0]:
                continue
            if value is None:
                db.execute("DELETE FROM journal WHERE key = ?", (key,))
            else:
                if type(value) is not bytes:
                    raise TypeError("value must be bytes or None")
                db.execute(
                    "INSERT INTO journal(key, sequence, value) VALUES (?, ?, ?) "
                    "ON CONFLICT(key) DO UPDATE SET "
                    "sequence = excluded.sequence, value = excluded.value",
                    (key, sequence, value),
                )


def snapshot(db: sqlite3.Connection) -> list[tuple[str, bytes]]:
    return [
        (key, bytes(value))
        for key, value in db.execute(
            "SELECT key, value FROM journal WHERE value IS NOT NULL ORDER BY key"
        )
    ]
