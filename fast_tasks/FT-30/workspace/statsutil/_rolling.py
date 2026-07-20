"""Internal rolling-window helpers (stdlib only)."""


def rolling_mean(values: list[float], window: int) -> list[float]:
    """Return the simple moving average for each full window.

    For input length n and window w, the result has length n - w + 1 when
    n >= w. Each output element is the arithmetic mean of w consecutive
    values. Empty input or window larger than n yields [].
    """
    assert isinstance(values, list)
    assert isinstance(window, int)
    assert window >= 1
    n = len(values)
    if n < window:
        return []
    out: list[float] = []
    running = sum(values[:window])
    out.append(running / window)
    for i in range(window, n):
        running += values[i] - values[i - window]
        out.append(running / window)
    return out
