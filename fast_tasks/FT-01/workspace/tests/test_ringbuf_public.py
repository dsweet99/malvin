from src.ringbuf import RingBuffer


def test_basic_push_pop():
    b = RingBuffer(3)
    b.push(1)
    b.push(2)
    assert b.pop() == 1
    assert b.pop() == 2
