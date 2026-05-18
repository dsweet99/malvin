
## Observation

1) What *new*, interesting/surprising things did you learn in this chat about?:
- Codebase structure (where things live)
- Algorithms/methods + notable results
- Tooling/workflow constraints (commands, env gotchas, quality gates)
- The user's preferences (about coding or communication)
- Agent pacing (latency vs thoroughness)
- Code quality
- The subject matter being address by this coding project
Newness should be decided relative to the existing `./.malvin_memory/*.md` files.

2) Summarize them in a list.

3) Edit an `.malvin_memory/*.md` file to incorporate each list item.

- Store as TRIGGER: / ADVICE: pairs, in this format
```
TRIGGER: 1-4 keywords or phrases, where phrases are 1-3 words
ADVICE: A briefly-stated action to take, like "Read <file>" or "Use <tool/approach/method, with instructions>" or "Check for <problem/benefit>" etc.
CONFIDENCE: 0
```
in one of `./.malvin_memory/*.md`. If no file name (subject area) seems appropriate, create a new file with an appropriate file name (subject area).

TRIGGER keywords should be words that the agent was searching for -- or might search for -- when it had the question but didn't yet have the answer. ADVICE should be the final, discovered, correct or useful action that the agent took. Our hope is that a future agent will grep for keywords, find a relevant TRIGGER, and be very glad it found the associated ADVICE.

## Falsification

Each TRIGGER: / ADVICE: / CONFIDENCE: triple is implicitly hypothesizing that it will be "good advice".

Forgetting: Can you find one piece of ADVICE: that was followed during this session that was *bad* advice? For example, did it lead to a bad outcome, repeat a stock malvin prompt, or just waste time producing nothing useful? If so, consider that TRIGGER: / ADVICE: / CONFIDENCE: triple falsified and remove it from ./.malvin_memory/*.md.

Increasing confidence: Pick one piece of ADVICE: that wasn't bad and had CONFIDENCE: < 3 -- if there was one. Find it in ./.malvin_memory/*.md and increment its CONFIDENCE: value by 1, but limit CONFIDENCE: to a maximum value of 3.
