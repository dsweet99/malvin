"""Revision 2 timeline codec over the binary transport."""
from __future__ import annotations

import zlib

from .r1_binary import Entry, _checked, _take_varint, _varint


def pack(entries: list[Entry]) -> bytes:
    records = [_checked(entry) for entry in entries]
    chunks = [b"B2", _varint(len(records))]
    for key, version, value in records:
        encoded = key.encode("utf-8")
        chunks.extend((_varint(len(encoded)), encoded, version.to_bytes(2, "big")))
        if value is None:
            chunks.append(b"\x00")
        else:
            chunks.extend((b"\x01", _varint(len(value)), value))
    body = b"".join(chunks)
    return body + zlib.crc32(body).to_bytes(4, "big")


def unpack(frame: bytes) -> list[Entry]:
    if type(frame) is not bytes:
        raise TypeError("frame must be bytes")
    if len(frame) < 7 or frame[:2] != b"B2":
        raise ValueError("wrong revision")
    body, checksum = frame[:-4], frame[-4:]
    if zlib.crc32(body).to_bytes(4, "big") != checksum:
        raise ValueError("bad checksum")
    count, offset = _take_varint(body, 2)
    result: list[Entry] = []
    for _ in range(count):
        size, offset = _take_varint(body, offset)
        end = offset + size
        if end + 3 > len(body):
            raise ValueError("truncated record")
        key = body[offset:end].decode("utf-8")
        version = int.from_bytes(body[end:end + 2], "big")
        marker = body[end + 2]
        offset = end + 3
        if marker == 0:
            value = None
        elif marker == 1:
            size, offset = _take_varint(body, offset)
            end = offset + size
            if end > len(body):
                raise ValueError("truncated payload")
            value = body[offset:end]
            offset = end
        else:
            raise ValueError("bad payload marker")
        result.append((key, version, value))
    if offset != len(body):
        raise ValueError("trailing data")
    return result
