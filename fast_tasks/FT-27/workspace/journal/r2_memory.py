"""Revision 2 journal backed by a dictionary."""
from __future__ import annotations

import unicodedata
from collections.abc import Iterable

from .r1_memory import Event, _sequence


State = dict[str, tuple[int, bytes | None]]


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


def apply(state: State, events: Iterable[Event]) -> None:
    staged = state.copy()
    for raw_key, raw_sequence, value in events:
        key = _key(raw_key)
        sequence = _sequence(raw_sequence)
        if value is not None and type(value) is not bytes:
            raise TypeError("value must be bytes or None")
        old = staged.get(key)
        if old is None or _wins(sequence, old[0]):
            staged[key] = (sequence, value)
    state.clear()
    state.update(staged)


def snapshot(state: State) -> list[tuple[str, bytes]]:
    return [
        (key, value)
        for key, (_, value) in sorted(state.items())
        if value is not None
    ]
