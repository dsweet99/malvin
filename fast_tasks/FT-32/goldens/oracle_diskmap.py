"""Reference LSM-style DiskMap used only by the grader self-test oracle."""

from __future__ import annotations

import json
import struct
from pathlib import Path

_TOMBSTONE_LEN = 0xFFFFFFFF
_U32 = struct.Struct("<I")
_SPARSE_EVERY = 64


def _write_record(fh, key: bytes, value: bytes | None) -> None:
    fh.write(_U32.pack(len(key)))
    fh.write(key)
    if value is None:
        fh.write(_U32.pack(_TOMBSTONE_LEN))
    else:
        fh.write(_U32.pack(len(value)))
        fh.write(value)


class _Segment:
    """On-disk sorted segment with sparse index; reads via seek (not full slurp)."""

    def __init__(self, path: Path) -> None:
        self.path = path
        self.index: list[tuple[bytes, int]] = []
        idx_path = path.with_suffix(".idx")
        if idx_path.is_file():
            raw = idx_path.read_bytes()
            off = 0
            while off < len(raw):
                klen = _U32.unpack_from(raw, off)[0]
                off += 4
                key = raw[off : off + klen]
                off += klen
                pos = _U32.unpack_from(raw, off)[0]
                off += 4
                self.index.append((key, pos))

    def _seek_start(self, key: bytes) -> int:
        lo, hi = 0, len(self.index)
        start = 0
        while lo < hi:
            mid = (lo + hi) // 2
            if self.index[mid][0] <= key:
                start = self.index[mid][1]
                lo = mid + 1
            else:
                hi = mid
        return start

    def get(self, key: bytes) -> tuple[bool, bytes | None]:
        with self.path.open("rb") as fh:
            fh.seek(self._seek_start(key))
            while True:
                hdr = fh.read(4)
                if len(hdr) < 4:
                    return False, None
                klen = _U32.unpack(hdr)[0]
                k = fh.read(klen)
                if len(k) < klen:
                    return False, None
                vhdr = fh.read(4)
                if len(vhdr) < 4:
                    return False, None
                vlen = _U32.unpack(vhdr)[0]
                if vlen == _TOMBSTONE_LEN:
                    value = None
                else:
                    value = fh.read(vlen)
                    if len(value) < vlen:
                        return False, None
                if k == key:
                    return True, value
                if k > key:
                    return False, None

    def iter_from(self, lo: bytes, hi: bytes):
        with self.path.open("rb") as fh:
            fh.seek(self._seek_start(lo))
            while True:
                hdr = fh.read(4)
                if len(hdr) < 4:
                    return
                klen = _U32.unpack(hdr)[0]
                k = fh.read(klen)
                if len(k) < klen:
                    return
                vhdr = fh.read(4)
                if len(vhdr) < 4:
                    return
                vlen = _U32.unpack(vhdr)[0]
                if vlen == _TOMBSTONE_LEN:
                    value = None
                else:
                    value = fh.read(vlen)
                    if len(value) < vlen:
                        return
                if k < lo:
                    continue
                if k >= hi:
                    return
                yield k, value


class DiskMap:
    def __init__(self, root: str | Path, *, mem_budget_bytes: int) -> None:
        assert isinstance(mem_budget_bytes, int) and mem_budget_bytes >= 1024
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.mem_budget_bytes = mem_budget_bytes
        self._mem: dict[bytes, bytes | None] = {}
        self._mem_bytes = 0
        self._segs: list[_Segment] = []
        self._next_id = 1
        self._load_meta()

    def _meta_path(self) -> Path:
        return self.root / "meta.json"

    def _load_meta(self) -> None:
        meta = self._meta_path()
        if not meta.is_file():
            return
        info = json.loads(meta.read_text(encoding="utf-8"))
        self._next_id = int(info["next_id"])
        for name in info.get("segments", []):
            path = self.root / name
            if path.is_file():
                self._segs.append(_Segment(path))

    def _save_meta(self) -> None:
        names = [seg.path.name for seg in self._segs]
        payload = {"next_id": self._next_id, "segments": names}
        self._meta_path().write_text(json.dumps(payload), encoding="utf-8")

    def _charge(self, key: bytes, value: bytes | None) -> int:
        return len(key) + (0 if value is None else len(value)) + 16

    def put(self, key: bytes, value: bytes) -> None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        assert isinstance(value, (bytes, bytearray)) and len(value) > 0
        self._set(bytes(key), bytes(value))

    def delete(self, key: bytes) -> None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        self._set(bytes(key), None)

    def _set(self, key: bytes, value: bytes | None) -> None:
        if key in self._mem:
            self._mem_bytes -= self._charge(key, self._mem[key])
        self._mem[key] = value
        self._mem_bytes += self._charge(key, value)
        if self._mem_bytes >= max(1024, self.mem_budget_bytes // 2):
            self.flush()

    def flush(self) -> None:
        if not self._mem:
            self._save_meta()
            return
        items = sorted(self._mem.items(), key=lambda kv: kv[0])
        seg_name = f"seg_{self._next_id:06d}.sst"
        self._next_id += 1
        seg_path = self.root / seg_name
        idx = bytearray()
        with seg_path.open("wb") as fh:
            for i, (key, value) in enumerate(items):
                if i % _SPARSE_EVERY == 0:
                    idx.extend(_U32.pack(len(key)))
                    idx.extend(key)
                    idx.extend(_U32.pack(fh.tell()))
                _write_record(fh, key, value)
        seg_path.with_suffix(".idx").write_bytes(bytes(idx))
        self._segs.insert(0, _Segment(seg_path))
        self._mem.clear()
        self._mem_bytes = 0
        self._save_meta()

    def drop_cache(self) -> None:
        self.flush()
        names = [seg.path.name for seg in self._segs]
        self._segs = [_Segment(self.root / name) for name in names if (self.root / name).is_file()]
        self._mem.clear()
        self._mem_bytes = 0

    def get(self, key: bytes) -> bytes | None:
        assert isinstance(key, (bytes, bytearray)) and len(key) > 0
        key_b = bytes(key)
        if key_b in self._mem:
            return self._mem[key_b]
        for seg in self._segs:
            found, value = seg.get(key_b)
            if found:
                return value
        return None

    def range(self, lo: bytes, hi: bytes) -> list[tuple[bytes, bytes]]:
        assert isinstance(lo, (bytes, bytearray)) and isinstance(hi, (bytes, bytearray))
        lo_b, hi_b = bytes(lo), bytes(hi)
        latest: dict[bytes, bytes | None] = {}
        for seg in reversed(self._segs):
            for key, value in seg.iter_from(lo_b, hi_b):
                latest[key] = value
        for key, value in self._mem.items():
            if lo_b <= key < hi_b:
                latest[key] = value
        return [(k, v) for k, v in sorted(latest.items()) if v is not None]

    def close(self) -> None:
        self.flush()
        self._mem.clear()
        self._mem_bytes = 0
        self._segs.clear()
