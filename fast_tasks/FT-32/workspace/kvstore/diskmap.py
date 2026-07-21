"""Starter DiskMap: in-RAM only. Fails durability, drop_cache, and resource caps."""

from __future__ import annotations

from pathlib import Path


class DiskMap:
    def __init__(self, root: str | Path, *, mem_budget_bytes: int) -> None:
        assert isinstance(mem_budget_bytes, int) and mem_budget_bytes >= 1024
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.mem_budget_bytes = mem_budget_bytes
        self._data: dict[bytes, bytes | None] = {}

    def put(self, key: bytes, value: bytes) -> None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        assert isinstance(value, (bytes, bytearray)) and len(value) > 0
        self._data[bytes(key)] = bytes(value)

    def delete(self, key: bytes) -> None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        self._data[bytes(key)] = None

    def get(self, key: bytes) -> bytes | None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        value = self._data.get(bytes(key), None)
        return None if value is None else value

    def range(self, lo: bytes, hi: bytes) -> list[tuple[bytes, bytes]]:
        assert isinstance(lo, (bytes, bytearray)) and isinstance(hi, (bytes, bytearray))
        lo_b, hi_b = bytes(lo), bytes(hi)
        out: list[tuple[bytes, bytes]] = []
        for key in sorted(self._data):
            if key < lo_b:
                continue
            if key >= hi_b:
                break
            value = self._data[key]
            if value is not None:
                out.append((key, value))
        return out

    def flush(self) -> None:
        # Starter never writes a recoverable on-disk image.
        return None

    def drop_cache(self) -> None:
        # Clears RAM without a durable image — data loss (intentional starter bug).
        self._data.clear()

    def close(self) -> None:
        self.flush()
        self._data.clear()
