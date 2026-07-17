"""Revision 1 timeline codec over the text transport."""
from __future__ import annotations

import base64
import json
import zlib

from .r1_binary import Entry, _checked


def _records_bytes(records: list[list[object]]) -> bytes:
    return json.dumps(
        records, ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")


def pack(entries: list[Entry]) -> bytes:
    records = []
    for entry in entries:
        key, version, value = _checked(entry)
        encoded = None if value is None else base64.b64encode(value).decode("ascii")
        records.append([key, version.to_bytes(2, "big").hex(), encoded])
    return json.dumps(
        {
            "check": f"{zlib.crc32(_records_bytes(records)):08x}",
            "revision": 1,
            "records": records,
        },
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def unpack(frame: bytes) -> list[Entry]:
    if type(frame) is not bytes:
        raise TypeError("frame must be bytes")
    try:
        document = json.loads(frame)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("bad text frame") from error
    if (
        type(document) is not dict
        or set(document) != {"check", "revision", "records"}
        or document["revision"] != 1
        or type(document["records"]) is not list
        or document["check"]
        != f"{zlib.crc32(_records_bytes(document['records'])):08x}"
    ):
        raise ValueError("bad text envelope")
    result = []
    for raw in document["records"]:
        if type(raw) is not list or len(raw) != 3:
            raise ValueError("bad record")
        key, encoded_version, encoded = raw
        if (
            type(encoded_version) is not str
            or len(encoded_version) != 4
            or encoded_version != encoded_version.lower()
        ):
            raise ValueError("bad version")
        try:
            version = int.from_bytes(bytes.fromhex(encoded_version), "big")
        except ValueError as error:
            raise ValueError("bad version") from error
        if encoded is None:
            value = None
        elif type(encoded) is str:
            try:
                value = base64.b64decode(encoded, validate=True)
            except ValueError as error:
                raise ValueError("bad payload") from error
        else:
            raise ValueError("bad payload")
        checked = _checked((key, version, value))
        if checked[0] != key:
            raise ValueError("non-canonical key")
        result.append(checked)
    return result
