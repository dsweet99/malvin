"""Revision 1 dispatcher using a priority heap."""
from __future__ import annotations

import heapq
import itertools

from .r1_linear import Job, _key, _priority


class Dispatcher:
    def __init__(self) -> None:
        self._heap: list[tuple[int, str, int, bytes]] = []
        self._order = itertools.count()

    def put(self, key: str, priority: int, payload: bytes) -> None:
        key = _key(key)
        priority = _priority(priority)
        if type(payload) is not bytes:
            raise TypeError("payload must be bytes")
        heapq.heappush(
            self._heap,
            (-priority, key, next(self._order), payload),
        )

    def take(self) -> Job | None:
        if not self._heap:
            return None
        priority, key, _, payload = heapq.heappop(self._heap)
        return key, -priority, payload
