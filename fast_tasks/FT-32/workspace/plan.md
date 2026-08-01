# FT-32: Durable hybrid memtable + sorted-segment map

Work only in this workspace.

## Incident

Callers need a durable ordered key-value map that keeps a bounded in-memory
write buffer and spills immutable sorted segments to disk. The public API lives
in `kvstore.diskmap.DiskMap`. The starter is intentionally incomplete: it keeps
everything in a Python dict and loses data when the cache is dropped.

You must implement a real hybrid structure. A correct solution looks like a
small LSM / sorted-string-table store: a memtable, flushes to sorted segment
files, sparse indexes (or equivalent) for point lookups, tombstones for
deletes, and merge logic for range scans. Loading the whole dataset into RAM,
replaying a flat log on every read, or scanning every byte of every segment on
every `get` will fail the hidden grade workload under the resource caps below.

## Public API (keep signatures)

```python
class DiskMap:
    def __init__(self, root: str | Path, *, mem_budget_bytes: int) -> None: ...
    def put(self, key: bytes, value: bytes) -> None: ...
    def delete(self, key: bytes) -> None: ...
    def get(self, key: bytes) -> bytes | None: ...
    def range(self, lo: bytes, hi: bytes) -> list[tuple[bytes, bytes]]: ...
    def flush(self) -> None: ...
    def drop_cache(self) -> None: ...
    def close(self) -> None: ...
```

Semantics:

- Keys and values are non-empty `bytes`. `delete` installs a tombstone; later
  `get` returns `None` until a newer `put`.
- `range(lo, hi)` returns live `(key, value)` pairs with `lo <= key < hi`,
  sorted by key ascending.
- `flush` must persist the memtable so a crash after `flush` does not lose
  buffered writes. `close` implies a final flush.
- `drop_cache` must flush, then discard all in-memory caches and memtable
  contents. Subsequent `get` / `range` must still be correct using only on-disk
  state. The grader calls `drop_cache` mid-workload.
- After `close`, reopening `DiskMap` on the same `root` must observe the same
  logical map.

## Complexity, wall-time, and memory (graded)

Let `N` be the number of live keys after the grade workload (order 1e5–2e5,
with multi-hundred-byte values).

- Amortized cost: `put` / `delete` / `get` must behave like `O(log N)` I/O with
  a bounded memtable (not `O(N)` full-segment scans per operation once many
  segments exist). `range` over `K` live hits must be `O(K + log N)`, not
  `O(N)` per call when `K` is small.
- Memory: with `mem_budget_bytes` set near 8 MiB, process peak RSS during the
  hidden workload must stay under the grader’s hard cap (tens of MiB above a
  clean interpreter baseline). An all-in-RAM dict of the full working set fails.
- Wall time: the full hidden workload (writes, updates, deletes, forced
  `drop_cache`, many point reads, several ranges, close/reopen) must finish
  within the grader’s wall-time budget on this host class.

## Constraints

- Edit package code under `kvstore/` only. Do not edit `tests/`, `plan.md`, or
  `pytest.ini`.
- Use only the Python standard library.
- Do **not** import or use `sqlite3`, `dbm`, `shelve`, or third-party stores.
  Build the on-disk layout yourself.
- Do not special-case the public smoke-test keys.
- Do not read parent directories (grader / goldens).

## Done when

`python -m pytest -q` passes, and the hidden grader accepts correctness under
`drop_cache`, durability across reopen, plus the wall-time and memory caps.

Stay inside this workspace directory. Do not read or edit parent directories
(including any grader or golden files outside this folder).
