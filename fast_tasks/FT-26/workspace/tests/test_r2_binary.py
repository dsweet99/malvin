from wire.r2_binary import pack, unpack


def test_ordinary_round_trip() -> None:
    entries = [(7, True), (3, True), (11, True)]
    frame = pack(entries)
    assert frame == b"B2\x03\x03\x01\x04\x01\x04\x01"
    assert unpack(frame) == [(3, True), (7, True), (11, True)]
