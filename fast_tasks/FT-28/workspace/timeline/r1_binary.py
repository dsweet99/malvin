"""Revision 1 timeline codec over the binary transport."""
from __future__ import annotations

import zlib

Entry = tuple[str, int, bytes | None]


def _key(value: str) -> str:
    if type(value) is not str:
        raise TypeError("key must be text")
    result = " ".join(value.split()).casefold()
    if not result:
        raise ValueError("empty key")
    return result


def _version(value: int) -> int:
    if type(value) is not int or not 0 <= value <= 0xFFFF:
        raise ValueError("version out of range")
    return value


def _varint(value: int) -> bytes:
    result = bytearray()
    while value >= 0x80:
        result.append((value & 0x7F) | 0x80)
        value >>= 7
    result.append(value)
    return bytes(result)


def _take_varint(data: bytes, offset: int) -> tuple[int, int]:
    start = offset
    value = 0
    shift = 0
    while offset < len(data) and shift <= 28:
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            if data[start:offset] != _varint(value):
                raise ValueError("non-canonical varint")
            return value, offset
        shift += 7
    raise ValueError("bad varint")


def _checked(entry: Entry) -> Entry:
    raw_key, raw_version, value = entry
    key = _key(raw_key)
    version = _version(raw_version)
    if value is not None and type(value) is not bytes:
        raise TypeError("value must be bytes or None")
    return key, version, value


def pack(entries: list[Entry]) -> bytes:
    records = [_checked(entry) for entry in entries]
    chunks = [b"B1", _varint(len(records))]
    for key, version, value in records:
        encoded = key.encode("utf-8")
        chunks.extend((_varint(len(encoded)), encoded, version.to_bytes(2, "big")))
        if value is None:
            chunks.append(b"\x00")
        else:
            chunks.extend((b"\x01", _varint(len(value)), value))
    body = b"".join(chunks)
    return body + zlib.crc32(body).to_bytes(4, "big")


def unpack(frame: bytes) -> list[Entry]:
    if type(frame) is not bytes:
        raise TypeError("frame must be bytes")
    if len(frame) < 7 or frame[:2] != b"B1":
        raise ValueError("wrong revision")
    body, checksum = frame[:-4], frame[-4:]
    if zlib.crc32(body).to_bytes(4, "big") != checksum:
        raise ValueError("bad checksum")
    count, offset = _take_varint(body, 2)
    result: list[Entry] = []
    for _ in range(count):
        size, offset = _take_varint(body, offset)
        end = offset + size
        if end + 3 > len(body):
            raise ValueError("truncated record")
        try:
            key = body[offset:end].decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError("bad key encoding") from error
        if key != _key(key):
            raise ValueError("non-canonical key")
        version = int.from_bytes(body[end:end + 2], "big")
        marker = body[end + 2]
        offset = end + 3
        if marker == 0:
            value = None
        elif marker == 1:
            size, offset = _take_varint(body, offset)
            end = offset + size
            if end > len(body):
                raise ValueError("truncated payload")
            value = body[offset:end]
            offset = end
        else:
            raise ValueError("bad payload marker")
        result.append((key, version, value))
    if offset != len(body):
        raise ValueError("trailing data")
    return result
