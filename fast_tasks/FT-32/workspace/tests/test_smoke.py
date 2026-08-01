"""Public smoke tests — deliberately small; hidden grader is the real bar."""

from __future__ import annotations

from pathlib import Path

from kvstore import DiskMap


def test_put_get_delete_roundtrip(tmp_path: Path) -> None:
    store = DiskMap(tmp_path / "db", mem_budget_bytes=1 << 20)
    store.put(b"a", b"1")
    store.put(b"b", b"2")
    assert store.get(b"a") == b"1"
    store.delete(b"a")
    assert store.get(b"a") is None
    assert store.range(b"a", b"z") == [(b"b", b"2")]
    store.close()


def test_drop_cache_must_preserve_flushed_state(tmp_path: Path) -> None:
    store = DiskMap(tmp_path / "db2", mem_budget_bytes=1 << 20)
    store.put(b"k", b"v")
    store.flush()
    store.drop_cache()
    assert store.get(b"k") == b"v"
    store.close()
