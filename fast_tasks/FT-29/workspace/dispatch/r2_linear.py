"""Revision 2 dispatcher using a linear ready list."""
from __future__ import annotations

import unicodedata

from .r1_linear import _priority


Event = tuple[str, int, int, bytes | None]
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
        self._state: dict[str, tuple[int, int, bytes | None]] = {}

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
            self._state[key] = (generation, priority, payload)

    def take(self) -> Job | None:
        candidates = [
            (key, generation, priority, payload)
            for key, (generation, priority, payload) in self._state.items()
            if payload is not None
        ]
        if not candidates:
            return None
        key, generation, priority, payload = min(
            candidates,
            key=lambda item: (-item[2], item[0]),
        )
        self._state[key] = (generation, priority, None)
        return key, generation, priority, payload
