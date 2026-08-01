from dispatch.r2_heap import Dispatcher


def test_r2_heap_orders_ordinary_jobs() -> None:
    queue = Dispatcher()
    queue.put("alpha", 1, 3, b"a")
    queue.put("beta", 1, 9, b"b")

    assert queue.take() == ("beta", 1, 9, b"b")
    assert queue.take() == ("alpha", 1, 3, b"a")
    assert queue.take() is None
