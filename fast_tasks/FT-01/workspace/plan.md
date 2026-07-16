# FT-01: Ring buffer off-by-one

You are editing files in this workspace directory only.

## Task
The ring buffer in `src/ringbuf.py` has a bug in `RingBuffer.pop()`. After `N` pushes then `N` pops on capacity `N`, the next push must succeed without raising.

## Rules
- Fix only `src/ringbuf.py`.
- Do not edit `tests/`.
- Do not use the network. Stdlib only.
- When fixed, `python -m pytest -q tests/test_ringbuf_public.py` should pass. Hidden tests will also run at grade time.

## Done when
`RingBuffer` correctly wraps after a full drain so pushes succeed again.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
