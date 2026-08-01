"""Revision 1 over the binary transport."""
from __future__ import annotations


Posting = int
_MAX_U32 = (1 << 32) - 1


def _check(value: int) -> int:
    if type(value) is not int or value < 0:
        raise ValueError("posting id must be a non-negative int")
    return value


def _varint(value: int) -> bytes:
    if not 0 <= value <= _MAX_U32:
        raise ValueError("value out of range")
    encoded = bytearray()
    while value >= 0x80:
        encoded.append((value & 0x7F) | 0x80)
        value >>= 7
    encoded.append(value)
    return bytes(encoded)


def _take_varint(frame: bytes, offset: int) -> tuple[int, int]:
    start = offset
    value = 0
    shift = 0
    for _ in range(5):
        if offset >= len(frame):
            raise ValueError("truncated value")
        byte = frame[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            if value > _MAX_U32 or frame[start:offset] != _varint(value):
                raise ValueError("non-canonical value")
            return value, offset
        shift += 7
    raise ValueError("value out of range")


def pack(ids: list[Posting]) -> bytes:
    ordered = sorted({_check(value) for value in ids})
    chunks = [b"B1"]
    previous = None
    for value in ordered:
        chunks.append(_varint(value if previous is None else value - previous))
        previous = value
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Posting]:
    if not frame.startswith(b"B1"):
        raise ValueError("wrong revision")
    offset = 2
    ids: list[Posting] = []
    previous = None
    while offset < len(frame):
        gap, offset = _take_varint(frame, offset)
        if previous is None:
            value = gap
        else:
            if gap < 1:
                raise ValueError("non-increasing posting")
            value = previous + gap
        ids.append(value)
        previous = value
    return ids
