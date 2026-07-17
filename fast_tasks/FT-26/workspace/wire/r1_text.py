"""Revision 1 over the text transport."""
from __future__ import annotations


Posting = int


def _check(value: int) -> int:
    if type(value) is not int or value < 0:
        raise ValueError("posting id must be a non-negative int")
    return value


def _take_decimal(token: str) -> int:
    if not token.isascii() or not token.isdecimal():
        raise ValueError("invalid number")
    if len(token) > 1 and token[0] == "0":
        raise ValueError("non-canonical number")
    return int(token)


def pack(ids: list[Posting]) -> str:
    ordered = sorted({_check(value) for value in ids})
    gaps = []
    previous = None
    for value in ordered:
        gaps.append(value if previous is None else value - previous)
        previous = value
    return "P1|" + ",".join(str(gap) for gap in gaps)


def unpack(frame: str) -> list[Posting]:
    if not frame.startswith("P1|"):
        raise ValueError("wrong revision")
    body = frame[3:]
    if body == "":
        return []
    ids: list[Posting] = []
    previous = None
    for token in body.split(","):
        gap = _take_decimal(token)
        if previous is None:
            value = gap
        else:
            if gap < 1:
                raise ValueError("non-increasing posting")
            value = previous + gap
        ids.append(value)
        previous = value
    return ids
