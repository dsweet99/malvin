"""Revision 2 over the binary transport."""
from __future__ import annotations

from .r1_binary import Posting, _check, _take_varint, _varint

Entry = tuple[Posting, bool]

_LIVE = b"\x01"
_TOMB = b"\x00"


def _flag(live: bool) -> bytes:
    """Encode the liveness flag as a single trailing byte."""
    return _LIVE if live else _TOMB


def pack(entries: list[Entry]) -> bytes:
    """Pack an R2 binary frame."""
    ordered = sorted({_check(identifier) for identifier, live in entries if live})
    chunks = [b"B2", _varint(len(ordered))]
    previous = None
    for identifier in ordered:
        gap = identifier if previous is None else identifier - previous
        chunks.extend((_varint(gap), _flag(True)))
        previous = identifier
    return b"".join(chunks)


def unpack(frame: bytes) -> list[Entry]:
    """Unpack an R2 binary frame."""
    if not frame.startswith(b"B2"):
        raise ValueError("wrong revision")
    count, offset = _take_varint(frame, 2)
    result: list[Entry] = []
    previous = None
    for _ in range(count):
        gap, offset = _take_varint(frame, offset)
        flag = frame[offset:offset + 1]
        offset += 1
        if previous is None:
            value = gap
        else:
            value = previous + gap
        result.append((value, flag == _LIVE))
        previous = value
    return result
