"""Revision 2 over the text transport."""
from __future__ import annotations

import unicodedata

from .r1_text import Field, _take_decimal


def _name(value: str) -> str:
    normalized = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not normalized:
        raise ValueError("empty field name")
    return normalized


def pack(fields: list[Field]) -> str:
    latest: dict[str, str | None] = {}
    for name, value in fields:
        latest[_name(name)] = value
    records = sorted(latest.items())
    chunks = [f"T2|{len(records)}|"]
    for name, value in records:
        chunks.append(f"{len(name)}:{name}")
        chunks.append("-:" if value is None else f"{len(value)}:{value}")
    return "".join(chunks)


def unpack(frame: str) -> list[Field]:
    if not frame.startswith("T2|"):
        raise ValueError("wrong revision")
    count_end = frame.find("|", 3)
    if count_end < 0:
        raise ValueError("missing count")
    count_token = frame[3:count_end]
    if (
        not count_token.isascii()
        or not count_token.isdecimal()
        or (len(count_token) > 1 and count_token[0] == "0")
    ):
        raise ValueError("invalid count")
    offset = count_end + 1
    result: list[Field] = []
    for _ in range(int(count_token)):
        name_size, offset = _take_decimal(frame, offset)
        end = offset + name_size
        if end > len(frame):
            raise ValueError("truncated name")
        name = frame[offset:end]
        offset = end
        if frame.startswith("-:", offset):
            value = None
            offset += 2
        else:
            value_size, offset = _take_decimal(frame, offset)
            end = offset + value_size
            if end > len(frame):
                raise ValueError("truncated value")
            value = frame[offset:end]
            offset = end
        if _name(name) != name:
            raise ValueError("non-canonical name")
        result.append((name, value))
    if offset != len(frame):
        raise ValueError("trailing data")
    names = [name for name, _ in result]
    if names != sorted(set(names)):
        raise ValueError("non-canonical order")
    return result
