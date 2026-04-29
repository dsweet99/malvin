{{ coding_rules }}

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
TRIGGER: 1-4 keywords or phrases, where phrases are 1-3 words
ADVICE: A briefly-stated action to take, like "Read <file>" or "Use <tool/approach/method, with instructions>" or "Check for <problem/benefit>" etc.
CONFIDENCE: 0
- Keep `./.malvin_memory/index.md` < 100 lines; focus on highest-priority rules and index entries.
- Don't lose information: Consolidate into broader rules and/or move detail into `./.malvin_memory/<topic>.md`.
- `./.malvin_memory/<topic>.md` files should also use the  TRIGGER: / ADVICE: / CONFIDENCE: format.
- Keep commands and hard constraints exact.
