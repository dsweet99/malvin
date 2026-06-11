## Mini agent constraints (`--mini`)

- Put every shell action in a ` ```bash ` fenced block. Do not claim you ran a command in prose without a fence.
- One investigation turn may use multiple bash blocks; malvin runs them in order and returns combined output.
- When you are done investigating (no more commands needed), reply without bash fences and include an explicit summary.
- Do not emit `MINI_DONE` unless you intend to terminate the inner loop immediately.
