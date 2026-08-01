class RingBuffer:
    def __init__(self, capacity: int) -> None:
        if capacity < 1:
            raise ValueError("capacity must be >= 1")
        self.capacity = capacity
        self._buf = [None] * capacity
        self._head = 0  # next pop index
        self._tail = 0  # next push index
        self._size = 0

    def __len__(self) -> int:
        return self._size

    def push(self, item) -> None:
        if self._size >= self.capacity:
            raise IndexError("buffer full")
        self._buf[self._tail] = item
        self._tail = (self._tail + 1) % self.capacity
        self._size += 1

    def pop(self):
        if self._size == 0:
            raise IndexError("buffer empty")
        item = self._buf[self._head]
        self._buf[self._head] = None
        # BUG: off-by-one wrap — should advance then mod
        self._head = self._head + 1
        if self._head > self.capacity:  # should be >= capacity / use %
            self._head = 0
        self._size -= 1
        return item
