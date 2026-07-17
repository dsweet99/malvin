"""Revision 1 journal backed by a dictionary."""
from __future__ import annotations

from collections.abc import Iterable


Event = tuple[str, int, bytes | None]
State = dict[str, tuple[int, bytes]]


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    normalized = " ".join(value.split()).casefold()
    if not normalized:
        raise ValueError("empty key")
    return normalized


def _sequence(value: int) -> int:
    if type(value) is not int or not 0 <= value <= 0xFFFF:
        raise ValueError("sequence out of range")
    return value


def apply(state: State, events: Iterable[Event]) -> None:
    staged = state.copy()
    for raw_key, raw_sequence, value in events:
        key = _key(raw_key)
        sequence = _sequence(raw_sequence)
        old = staged.get(key)
        if old is not None and sequence < old[0]:
            continue
        if value is None:
            staged.pop(key, None)
        else:
            if type(value) is not bytes:
                raise TypeError("value must be bytes or None")
            staged[key] = (sequence, value)
    state.clear()
    state.update(staged)


def snapshot(state: State) -> list[tuple[str, bytes]]:
    return [(key, value) for key, (_, value) in sorted(state.items())]
