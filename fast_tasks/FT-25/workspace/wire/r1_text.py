"""Revision 1 over the text transport."""
from __future__ import annotations


Field = tuple[str, str | None]


def _name(value: str) -> str:
    normalized = " ".join(value.split()).casefold()
    if not normalized:
        raise ValueError("empty field name")
    return normalized


def _take_decimal(frame: str, offset: int) -> tuple[int, int]:
    end = frame.find(":", offset)
    if end < 0:
        raise ValueError("truncated length")
    token = frame[offset:end]
    if not token.isascii() or not token.isdecimal():
        raise ValueError("invalid length")
    if len(token) > 1 and token[0] == "0":
        raise ValueError("non-canonical length")
    return int(token), end + 1


def pack(fields: list[Field]) -> str:
    records = [(_name(name), value) for name, value in fields if value is not None]
    records.sort()
    body = "".join(f"{len(name)}:{name}{len(value)}:{value}" for name, value in records)
    return "T1|" + body


def unpack(frame: str) -> list[Field]:
    if not frame.startswith("T1|"):
        raise ValueError("wrong revision")
    offset = 3
    result: list[Field] = []
    while offset < len(frame):
        name_size, offset = _take_decimal(frame, offset)
        end = offset + name_size
        if end > len(frame):
            raise ValueError("truncated name")
        name = frame[offset:end]
        offset = end
        value_size, offset = _take_decimal(frame, offset)
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
