"""Revision 1 dispatcher using a linear ready list."""
from __future__ import annotations


Job = tuple[str, int, bytes]


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    result = " ".join(value.split()).casefold()
    if not result:
        raise ValueError("empty key")
    return result


def _priority(value: int) -> int:
    if type(value) is not int or not -(1 << 31) <= value < (1 << 31):
        raise ValueError("priority out of range")
    return value


class Dispatcher:
    def __init__(self) -> None:
        self._ready: list[Job] = []

    def put(self, key: str, priority: int, payload: bytes) -> None:
        key = _key(key)
        priority = _priority(priority)
        if type(payload) is not bytes:
            raise TypeError("payload must be bytes")
        self._ready.append((key, priority, payload))

    def take(self) -> Job | None:
        if not self._ready:
            return None
        index = min(
            range(len(self._ready)),
            key=lambda item: (-self._ready[item][1], self._ready[item][0], item),
        )
        return self._ready.pop(index)
