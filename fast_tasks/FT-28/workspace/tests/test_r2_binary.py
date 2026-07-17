from timeline.r2_binary import pack, unpack


def test_ordinary_round_trip() -> None:
    entries = [
        ("alpha", 10, b"one"),
        ("beta", 11, None),
        ("gamma", 12, b""),
    ]
    frame = pack(entries)
    assert frame.startswith(b"B2")
    assert unpack(frame) == entries
