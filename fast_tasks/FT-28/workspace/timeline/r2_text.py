"""Revision 2 timeline codec over the text transport."""
from __future__ import annotations

import base64
import json
import unicodedata
import zlib

from .r1_binary import Entry, _version


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    result = unicodedata.normalize("NFKC", " ".join(value.split()).casefold())
    if not result:
        raise ValueError("empty key")
    return result


def _wins(candidate: int, current: int) -> bool:
    distance = (candidate - current) & 0xFFFF
    return distance == 0 or distance < 0x8000


def _canonical(entries: list[Entry]) -> list[Entry]:
    latest: dict[str, tuple[int, bytes | None]] = {}
    for raw_key, raw_version, value in entries:
        key = _key(raw_key)
        version = _version(raw_version)
        if value is not None and type(value) is not bytes:
            raise TypeError("value must be bytes or None")
        previous = latest.get(key)
        if previous is None or _wins(version, previous[0]):
            latest[key] = (version, value)
    return [(key, *latest[key]) for key in sorted(latest)]


def _records_bytes(records: list[list[object]]) -> bytes:
    return json.dumps(
        records, ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")


def _inflate(encoded: str) -> bytes:
    try:
        compressed = base64.b64decode(encoded, validate=True)
        inflater = zlib.decompressobj()
        value = inflater.decompress(compressed)
        value += inflater.flush()
    except (ValueError, zlib.error) as error:
        raise ValueError("bad payload") from error
    if not inflater.eof or inflater.unused_data or inflater.unconsumed_tail:
        raise ValueError("bad payload")
    return value


def pack(entries: list[Entry]) -> bytes:
    records = []
    for key, version, value in _canonical(entries):
        encoded = (
            None
            if value is None
            else base64.b64encode(zlib.compress(value, level=9)).decode("ascii")
        )
        records.append([key, version.to_bytes(2, "little").hex(), encoded])
    return json.dumps(
        {
            "check": f"{zlib.adler32(_records_bytes(records)):08x}",
            "records": records,
            "revision": 2,
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
        or document["revision"] != 2
        or type(document["records"]) is not list
        or document["check"]
        != f"{zlib.adler32(_records_bytes(document['records'])):08x}"
    ):
        raise ValueError("bad text envelope")
    result = []
    previous = None
    for raw in document["records"]:
        if type(raw) is not list or len(raw) != 3:
            raise ValueError("bad record")
        key, encoded_version, encoded_value = raw
        if (
            type(key) is not str
            or key != _key(key)
            or previous is not None and key <= previous
            or type(encoded_version) is not str
            or len(encoded_version) != 4
            or encoded_version != encoded_version.lower()
        ):
            raise ValueError("non-canonical record")
        try:
            version = int.from_bytes(bytes.fromhex(encoded_version), "little")
        except ValueError as error:
            raise ValueError("bad version") from error
        if encoded_value is None:
            value = None
        elif type(encoded_value) is str:
            value = _inflate(encoded_value)
        else:
            raise ValueError("bad payload")
        result.append((key, version, value))
        previous = key
    return result
