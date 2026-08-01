"""Revision 1 over the binary transport."""
from __future__ import annotations


Field = tuple[bytes, bytes | None]
_MAX_U32 = (1 << 32) - 1


def _name(value: bytes) -> bytes:
    normalized = b" ".join(value.split()).lower()
    if not normalized:
        raise ValueError("empty field name")
    return normalized


def _varint(value: int) -> bytes:
    if not 0 <= value <= _MAX_U32:
        raise ValueError("length out of range")
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
            raise ValueError("truncated length")
        byte = frame[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            if value > _MAX_U32 or frame[start:offset] != _varint(value):
                raise ValueError("non-canonical length")
            return value, offset
        shift += 7
    raise ValueError("length out of range")


def pack(fields: list[Field]) -> bytes:
    records = [(_name(name), value) for name, value in fields if value is not None]
    records.sort()
    chunks = [b"B1"]
    for name, value in records:
        assert value is not None
        chunks.extend((_varint(len(name)), name, _varint(len(value)), value))
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Field]:
    if not frame.startswith(b"B1"):
        raise ValueError("wrong revision")
    offset = 2
    result: list[Field] = []
    while offset < len(frame):
        name_size, offset = _take_varint(frame, offset)
        end = offset + name_size
        if end > len(frame):
            raise ValueError("truncated name")
        name = frame[offset:end]
        offset = end
        value_size, offset = _take_varint(frame, offset)
        end = offset + value_size
        if end > len(frame):
            raise ValueError("truncated value")
        value = frame[offset:end]
        offset = end
        if _name(name) != name:
            raise ValueError("non-canonical name")
        result.append((name, value))
    if result != sorted(result):
        raise ValueError("non-canonical order")
    return result
