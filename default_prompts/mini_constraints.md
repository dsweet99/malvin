## Mini agent constraints (`openrouter:` / `local:` backend)

- Put every shell action in a ` ```bash ` fenced block. Do not claim you ran a command in prose without a fence.
- Do not insert informational-only "echo" commands into bash blocks. Use echo if it's needed but not simply to describe what you're doing.
- One investigation turn may use multiple bash blocks; malvin runs them in order and returns combined output.
- When you are done investigating (no more commands needed), reply without bash fences and include an explicit summary.
- Do not emit `MINI_DONE` unless you intend to terminate the inner loop immediately.

## Wire format (required)

Every assistant completion must use this exact section order:

## NEW_HISTORY
<replacement chat-state History>

## RESPONSE
<body that answers the New request>

Chat-state History is compressed working memory for this mini session — not a full chat transcript, and not workflow `header.md` log-file History. Preserve objectives, constraints, verified observations, hypotheses with confidence, decisions and reasons, completed and failed actions, unresolved questions, next actions, and pointers to authoritative logs, files, or commits. Label fact kinds: observed; user-provided; inference; proposal; verified action.
