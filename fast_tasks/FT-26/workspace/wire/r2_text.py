"""Revision 2 over the text transport."""
from __future__ import annotations

from .r1_text import Posting, _check, _take_decimal

Entry = tuple[Posting, bool]


def pack(entries: list[Entry]) -> str:
    latest: dict[Posting, bool] = {}
    for identifier, live in entries:
        latest[_check(identifier)] = bool(live)
    ordered = sorted(latest.items())
    parts = []
    previous = None
    for identifier, live in ordered:
        gap = identifier if previous is None else identifier - previous
        parts.append(f"{gap}{'+' if live else '-'}")
        previous = identifier
    return f"P2|{len(ordered)}|" + ",".join(parts)


def unpack(frame: str) -> list[Entry]:
    if not frame.startswith("P2|"):
        raise ValueError("wrong revision")
    count_end = frame.find("|", 3)
    if count_end < 0:
        raise ValueError("missing count")
    count = _take_decimal(frame[3:count_end])
    body = frame[count_end + 1:]
    tokens = [] if body == "" else body.split(",")
    if len(tokens) != count:
        raise ValueError("count mismatch")
    result: list[Entry] = []
    previous = None
    for token in tokens:
        if not token:
            raise ValueError("empty entry")
        flag = token[-1]
        if flag not in "+-":
            raise ValueError("bad liveness flag")
        gap = _take_decimal(token[:-1])
        if previous is None:
            value = gap
        else:
            if gap < 1:
                raise ValueError("non-increasing posting")
            value = previous + gap
        result.append((value, flag == "+"))
        previous = value
    return result
