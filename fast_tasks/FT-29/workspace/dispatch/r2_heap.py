"""Revision 2 dispatcher using a priority heap."""
from __future__ import annotations

import heapq
import itertools

from .r1_linear import _key, _priority
from .r2_linear import Job, _generation


class Dispatcher:
    def __init__(self) -> None:
        self._heap: list[tuple[int, str, int, int, bytes]] = []
        self._latest: dict[str, int] = {}
        self._order = itertools.count()

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
        old = self._latest.get(key)
        if old is not None and generation <= old:
            return
        self._latest[key] = generation
        if payload is not None:
            heapq.heappush(
                self._heap,
                (-priority, key, next(self._order), generation, payload),
            )

    def take(self) -> Job | None:
        while self._heap:
            priority, key, _, generation, payload = heapq.heappop(self._heap)
            if self._latest.get(key) == generation:
                del self._latest[key]
                return key, generation, -priority, payload
        return None
