## Mini agent constraints (`openrouter:` / `local:` backend)

- Put every shell action in a ` ```bash ` fenced block. Do not claim you ran a command in prose without a fence.
- Do not insert informational-only "echo" commands into bash blocks. Use echo if it's needed but not simply to describe what you're doing.
- One investigation turn may use multiple bash blocks; malvin runs them in order and returns combined output.
- When you are done investigating (no more commands needed), reply without bash fences.
- Do not emit `MINI_DONE` unless you intend to terminate the inner loop immediately.

## Direct Messages (when required)

If the New request, History, or prior instructions mention `malvin --do`, do mode, or `MALVIN_DM_START` / `MALVIN_DM_END`, then the **RESPONSE** body must contain a closed DM fence with the user-visible answer:

MALVIN_DM_START
your answer here
MALVIN_DM_END

Copy those markers exactly (all caps, underscores). Replace "your answer here" with the real answer. Do not wrap the markers in a markdown code fence. Text outside the DM fence is not shown to the user on plain `malvin --do`. After bash observations, still finish with that DM fence — a plain prose summary alone is a failed response.

## Assembly (each completion)

Malvin assembles every mini completion as:

1. **Header** — sticky System instructions (these constraints, memory schema, model slug) plus a dynamic Study/Act cue
2. **History** — compact chat-state History (omitted when empty)
3. **Previous response** — last RESPONSE body only, verbatim (omitted when empty)
4. **New request** — latest user text, bash observation, or gate-retry divergence note

## Wire format (required)

Every assistant completion must use this exact section order:

## NEW_HISTORY
<replacement chat-state History>

## RESPONSE
<body that answers the New request>

Chat-state History is compressed working memory for this mini session — not a full chat transcript, and not workflow `header.md` log-file History. Preserve objectives, constraints, verified observations, hypotheses with confidence, decisions and reasons, completed and failed actions, unresolved questions, next actions, and pointers to authoritative logs, files, or commits. Label fact kinds: observed; user-provided; inference; proposal; verified action. When DM fences are required, keep that constraint in History every turn.
