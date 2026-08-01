"""Revision 2 over the binary transport."""
from __future__ import annotations

from .r1_binary import Field, _name, _take_varint, _varint


def _nullable_size(value: bytes | None) -> bytes:
    """Zero is null; a present value is encoded as its byte length plus one."""
    return _varint(0 if value is None else len(value) + 1)


def pack(fields: list[Field]) -> bytes:
    """Pack an R2 binary frame."""
    records = [(_name(name), value) for name, value in fields if value is not None]
    records.sort()
    chunks = [b"B2", _varint(len(records))]
    for name, value in records:
        chunks.extend((_varint(len(name)), name, _nullable_size(value)))
        if value:
            chunks.append(value)
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Field]:
    """Unpack and validate a canonical R2 binary frame."""
    if not frame.startswith(b"B2"):
        raise ValueError("wrong revision")
    count, offset = _take_varint(frame, 2)
    result: list[Field] = []
    for _ in range(count):
        name_size, offset = _take_varint(frame, offset)
        end = offset + name_size
        if end > len(frame):
            raise ValueError("truncated name")
        name = frame[offset:end]
        offset = end
        value_size, offset = _take_varint(frame, offset)
        if value_size == 0:
            value = None
        else:
            end = offset + value_size - 1
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
