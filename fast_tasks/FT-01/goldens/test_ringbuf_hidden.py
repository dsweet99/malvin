from src.ringbuf import RingBuffer


def test_wrap_after_full_drain():
    n = 5
    b = RingBuffer(n)
    for i in range(n):
        b.push(i)
    for i in range(n):
        assert b.pop() == i
    # Critical: next push must succeed (expose off-by-one head wrap)
    b.push(99)
    assert b.pop() == 99


def test_interleaved():
    b = RingBuffer(2)
    b.push("a")
    b.push("b")
    assert b.pop() == "a"
    b.push("c")
    assert b.pop() == "b"
    assert b.pop() == "c"
