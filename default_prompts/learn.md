{{ coding_rules }}

## Learning

1) What *new*, interesting/surprising things did you learn in this chat about?:
- Codebase structure (where things live)
- Algorithms/methods + notable results
- Tooling/workflow constraints (commands, env gotchas, quality gates)
- The user's preferences (about coding or communication)
- Agent pacing (latency vs thoroughness)
- Code quality

2) Summarize them as a short bullet list.

3) Edit `./.malvin_memory/index.md` to incorporate the new info.
- Store as TRIGGER: / ADVICE: / CONFIDENCE: triples, in this format
```
TRIGGER: 1-4 keywords or phrases, where phrases are 1-3 words
ADVICE: A briefly-stated action to take, like "Read <file>" or "Use <tool/approach/method, with instructions>" or "Check for <problem/benefit>" etc.
CONFIDENCE: 0
```
in one of `./.malvin_memory/*.md`. If no file name (subject area) seems appropriate, create a new file with an appropriate file name (subject area).

## Forgetting

Each TRIGGER: / ADVICE: / CONFIDENCE: triple is implicitly hypothesizing that it will be "good advice".

Can you find a piece of ADVICE: that was followed during this session that was not good advice? For example, did it lead to a bad outcome or just waste time producing nothing useful? If so, consider that TRIGGER: / ADVICE: / CONFIDENCE: triple falsified and remove it from ./.malvin_memory/*.md.
