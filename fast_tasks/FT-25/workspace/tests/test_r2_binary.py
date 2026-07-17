from wire.r2_binary import pack, unpack


def test_ordinary_round_trip() -> None:
    fields = [(b" Host ", b"example.test"), (b"accept", b"text/plain")]
    frame = pack(fields)
    assert frame == (
        b"B2\x02"
        b"\x06accept\x0btext/plain"
        b"\x04host\x0dexample.test"
    )
    assert unpack(frame) == [
        (b"accept", b"text/plain"),
        (b"host", b"example.test"),
    ]
